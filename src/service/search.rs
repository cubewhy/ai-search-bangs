use std::{fs, num::NonZeroUsize, sync::Arc};

use async_trait::async_trait;
use log::info;
use lru::LruCache;
use thiserror::Error;
use tokio::sync::Mutex;

use crate::{
    llm::{LLMError, LLMPrompt, LargeLanguageModel},
    model::{
        GenerateQueryResult, UserQueryRequest,
        search::{
            Baidu, Bing, Duckduckgo, DuckduckgoHtml, DuckduckgoLite, Google, SearchEngine, Sogou,
        },
    },
};

#[derive(serde::Deserialize, Clone, Debug)]
struct GenerateSearchQueryResponse {
    pub query: String,
}

#[derive(Error, Debug)]
pub enum SearchError {
    #[error("Error when querying AI")]
    Ai(#[from] LLMError),
    #[error("Unknown search engine {0}")]
    UnknownEngine(String),
    #[error("Failed to deserialize response")]
    Deserialize(#[from] serde_json::Error),
    #[error("Failed to read prompt file")]
    Io(#[from] std::io::Error),
}

#[async_trait]
pub trait SearchService: Send + Sync {
    async fn generate_query(
        &self,
        query: &str,
        search_engine: &str,
        language: Option<&str>,
    ) -> Result<GenerateQueryResult, SearchError>;
}

pub struct SearchServiceImpl {
    llm: Box<dyn LargeLanguageModel>,
    llm_model: String,
    prompt_template: String,
    cache: Arc<Mutex<LruCache<(String, String, Option<String>), String>>>,
}

impl SearchServiceImpl {
    pub fn new(llm: Box<dyn LargeLanguageModel>, llm_model: String, prompt_file: String) -> Self {
        let prompt_template = fs::read_to_string(prompt_file).expect("Failed to read prompt file");
        let cache = Arc::new(Mutex::new(LruCache::new(NonZeroUsize::new(100).unwrap())));
        Self {
            llm,
            llm_model,
            prompt_template,
            cache,
        }
    }
}

#[async_trait]
impl SearchService for SearchServiceImpl {
    async fn generate_query(
        &self,
        query_prompt: &str,
        search_engine: &str,
        language: Option<&str>,
    ) -> Result<GenerateQueryResult, SearchError> {
        let cache_key = (
            query_prompt.to_string(),
            search_engine.to_string(),
            language.map(|s| s.to_string()),
        );
        let mut cache = self.cache.lock().await;
        if let Some(cached_url) = cache.get(&cache_key) {
            info!("Cache hit for query: {}", query_prompt);
            return Ok(GenerateQueryResult {
                url: cached_url.clone(),
            });
        }
        drop(cache);

        let search_engine_instance: Box<dyn SearchEngine> =
            match search_engine.to_lowercase().as_str() {
                "ddg" | "duckduckgo" => Box::new(Duckduckgo::default()),
                "ddg-lite" | "duckduckgo-lite" => Box::new(DuckduckgoLite::default()),
                "ddg-html" | "duckduckgo-html" => Box::new(DuckduckgoHtml::default()),
                "google" => Box::new(Google::default()),
                "baidu" => Box::new(Baidu::default()),
                "bing" => Box::new(Bing::default()),
                "sogou" => Box::new(Sogou::default()),
                _ => return Err(SearchError::UnknownEngine(search_engine.to_string())),
            };

        info!(
            "Generate query using engine {} with prompt `{}` and language `{}`",
            search_engine_instance.name(),
            query_prompt,
            language.unwrap_or("<auto detect>"),
        );

        let user_query_request = UserQueryRequest {
            engine: search_engine_instance.name(),
            prompt: query_prompt.to_string(),
            language: language.map(|s| s.to_string()),
        };

        let contents = vec![
            LLMPrompt::new("user", &self.prompt_template),
            LLMPrompt::new(
                "model",
                "```json\n{\n  \"query\": \"!w history of artificial intelligence\"\n}\n```",
            ),
            LLMPrompt::new("user", &serde_json::to_string_pretty(&user_query_request)?),
        ];

        let ai_response = self.llm.query(&self.llm_model, &contents).await;

        let mut content = match ai_response {
            Ok(content) => content,
            Err(err) => return Err(SearchError::from(err)),
        };

        if content.starts_with("```json") {
            content = content
                .trim_start_matches("```json")
                .trim_end_matches("```")
                .to_string();
        }

        let response: GenerateSearchQueryResponse = serde_json::from_str(&content)?;

        let encoded_query = urlencoding::encode(&response.query);
        let url = search_engine_instance.generate_url(&encoded_query.to_string());

        info!(
            "Query successful generated for {} with prompt `{}` -> `{}`",
            search_engine_instance.name(),
            query_prompt,
            &response.query
        );

        let mut cache = self.cache.lock().await;
        cache.put(cache_key, url.clone());

        Ok(GenerateQueryResult { url })
    }
}
