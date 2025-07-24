use async_trait::async_trait;
use log::info;
use sqlx::SqlitePool;
use thiserror::Error;

use crate::{
    llm::{LLMError, LLMPrompt, LargeLanguageModel},
    model::{
        search::{
            Baidu, Bing, Duckduckgo, DuckduckgoHtml, DuckduckgoLite, DuckduckgoNoAi, Google, SearchEngine, Sogou,
        },
        GenerateQueryResult, UserQueryRequest,
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
    #[error("Database error")]
    Sqlx(#[from] sqlx::Error),
}

#[async_trait]
pub trait SearchService: Send + Sync {
    async fn generate_query(
        &self,
        query: &str,
        search_engine: &str,
        language: &str,
    ) -> Result<GenerateQueryResult, SearchError>;
}

pub struct SearchServiceImpl {
    llm: Box<dyn LargeLanguageModel>,
    llm_model: String,
    prompt_template: String,
    pool: SqlitePool,
}

impl SearchServiceImpl {
    pub fn new(
        llm: Box<dyn LargeLanguageModel>,
        llm_model: String,
        prompt_file: String,
        pool: SqlitePool,
    ) -> Self {
        let prompt_template = std::fs::read_to_string(prompt_file).expect("Failed to read prompt file");
        Self {
            llm,
            llm_model,
            prompt_template,
            pool,
        }
    }
}

#[async_trait]
impl SearchService for SearchServiceImpl {
    async fn generate_query(
        &self,
        query_prompt: &str,
        search_engine: &str,
        language: &str,
    ) -> Result<GenerateQueryResult, SearchError> {
        let cached_result = sqlx::query!(
            "SELECT url FROM cache WHERE query_prompt = ? AND search_engine = ? AND language = ?",
            query_prompt,
            search_engine,
            language
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(record) = cached_result {
            info!("Cache hit for query: {}", query_prompt);
            return Ok(GenerateQueryResult { url: record.url });
        }

        let search_engine_instance: Box<dyn SearchEngine> =
            match search_engine.to_lowercase().as_str() {
                "ddg" | "duckduckgo" => Box::new(Duckduckgo::default()),
                "ddg-lite" | "duckduckgo-lite" => Box::new(DuckduckgoLite::default()),
                "ddg-html" | "duckduckgo-html" => Box::new(DuckduckgoHtml::default()),
                "ddg-noai" | "duckduckgo-noai" => Box::new(DuckduckgoNoAi::default()),
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
            language
        );

        let user_query_request = UserQueryRequest {
            engine: search_engine_instance.name(),
            prompt: query_prompt.to_string(),
            language: language.to_string(),
        };

        let contents = vec![
            LLMPrompt::new("user", &self.prompt_template),
            LLMPrompt::new(
                "model",
                "```json\n{\n  \"query\": \"!w history of artificial intelligence\"\n}\n```",
            ),
            LLMPrompt::new(
                "user",
                &serde_json::to_string_pretty(&user_query_request)?,
            ),
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

        sqlx::query!(
            "INSERT INTO cache (query_prompt, search_engine, language, url) VALUES (?, ?, ?, ?)",
            query_prompt,
            search_engine,
            language,
            url
        )
        .execute(&self.pool)
        .await?;

        Ok(GenerateQueryResult { url })
    }
}
