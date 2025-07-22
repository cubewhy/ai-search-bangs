use async_trait::async_trait;
use thiserror::Error;

use crate::{
    llm::{LLMPrompt, LargeLanguageModel},
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
    Ai,
    #[error("Unknown search engine {0}")]
    UnknownEngine(String),

    #[error("Failed to deserialize response")]
    Deserialize(#[from] serde_json::Error),
}

#[async_trait]
pub trait SearchService {
    async fn generate_query(
        &self,
        query: &str,
        search_engine: &str,
    ) -> Result<GenerateQueryResult, SearchError>;
}

pub struct SearchServiceImpl {
    llm: Box<dyn LargeLanguageModel>,
    llm_model: String,
}

impl SearchServiceImpl {
    pub fn new(llm: Box<dyn LargeLanguageModel>, llm_model: String) -> Self {
        Self { llm, llm_model }
    }
}

#[async_trait]
impl SearchService for SearchServiceImpl {
    async fn generate_query(
        &self,
        query_prompt: &str,
        search_engine: &str,
    ) -> Result<GenerateQueryResult, SearchError> {
        // get search engine name and address
        let search_engine: Box<dyn SearchEngine> = match search_engine.to_lowercase().as_str() {
            "ddg" | "duckduckgo" => Box::new(Duckduckgo::default()),
            "ddg-lite" | "duckduckgo-lite" => Box::new(DuckduckgoLite::default()),
            "ddg-html" | "duckduckgo-html" => Box::new(DuckduckgoHtml::default()),
            "google" => Box::new(Google::default()),
            "baidu" => Box::new(Baidu::default()),
            "bing" => Box::new(Bing::default()),
            "sogou" => Box::new(Sogou::default()),
            _ => return Err(SearchError::UnknownEngine(search_engine.to_string())),
        };

        // TODO: move this into factory function
        let user_query_request = UserQueryRequest {
            engine: search_engine.name(),
            prompt: query_prompt.to_string(),
        };

        let contents = vec![
            LLMPrompt::new(
                "user",
                "# Role & Goal\nYou are a professional Search Query Optimizer. Your core task is to receive a JSON-formatted user input and convert it into an efficient, optimized search query string for a specific search engine. Your output must be precise, concise, and leverage the target search engine's features to the fullest.\n\n# Input Format\nThe input you receive will be a JSON object with two keys:\n- `engine`: A string specifying the target search engine (e.g., \"google\", \"bing\", \"duckduckgo\", \"baidu\").\n- `prompt`: A string containing the user's search intent in natural language.\n\nFor example:\n```json\n{\n  \"engine\": \"google\",\n  \"prompt\": \"Search on stackoverflow for how to quit vim, in English\"\n}\n```\n\n# Output Format\nYour output must be a JSON object containing only a single key, `query`.\n- `query`: A string representing the final, optimized search query.\n**Rule:** Strictly adhere to this format. Do not output any explanations, comments, or Markdown syntax.\n\nFor example:\n```json\n{\n  \"query\": \"site:stackoverflow.com how to quit vim\"\n}\n```\n\n# Processing Logic & Instructions\nYou must follow these steps to construct the final query:\n\n1.  **Identify Core Intent**: Analyze the natural language in the `prompt` to identify the core search topic, key entities, and the user's true objective.\n\n2.  **Extract Advanced Search Operators**: Identify and convert specific instruction words from the `prompt` into search engine operators.\n    - **Site-Specific Search**:\n        - Recognize: \"search on...\", \"from the website...\", \"site:...\" and similar patterns.\n        - Convert to: `site:domain.com`.\n        - Common Alias Mapping: \"stackoverflow\" -> `stackoverflow.com`, \"GitHub\" -> `github.com`.\n    - **Keyword Exclusion**:\n        - Recognize: \"don't...\", \"exclude...\", \"except for...\", \"not including...\".\n        - Convert to: Add a `-` prefix to the word to be excluded (e.g., `-pro`).\n    - **Exact Match**:\n        - Recognize: \"exact search for...\", \"the exact phrase is...\", \"verbatim...\", or when the user uses quotes.\n        - Convert to: Enclose the phrase in English double quotes `\"\"`.\n    - **File Type Specification**:\n        - Recognize: \"...as a PDF\", \"...PPT\", \"filetype:...\".\n        - Convert to: `filetype:pdf`, `filetype:ppt`, etc.\n    - **Language Instruction**:\n        - Recognize: \"in English\", \"in Chinese\", etc.\n        - Convert to: Translate the core search topic into the target language.\n        - **Note**: If the core content of the `prompt` is already in the target language, use it directly without re-translation.\n\n3.  **Preserve Critical Information**: If the `prompt` contains code, error messages (like `TypeError: 'NoneType' is not iterable`), specific IDs, or proper nouns, you must preserve them completely in the query. It is often recommended to use an exact match (quotes) for long error messages.\n\n4.  **Adapt to Target Engine**: Based on the `engine` value, use its specific syntax.\n    - **Google/Bing/DuckDuckGo**: Support `site:`, `-`, `\"\"`, `filetype:`, etc.\n    - **DuckDuckGo**: Also consider using `!` bang syntax (e.g., `!g` for a Google search, `!w` for Wikipedia). If the user's intent clearly points to a major site, this can be used.\n\n5.  **Combine and Optimize**: Combine the processed keywords, phrases, and operators into a logically structured query string. Keep the query concise, **preferably under 32 words in total**.\n\n# Examples\n\n### Example 1: Basic Site and Language Instruction (Google)\n- **Input**:\n```json\n{\n  \"engine\": \"google\",\n  \"prompt\": \"Search on stackoverflow for how to quit vim, in English\"\n}\n```\n- **Output**:\n```json\n{\n  \"query\": \"site:stackoverflow.com how to quit vim\"\n}\n```\n\n### Example 2: Keyword Exclusion and Exact Match (Bing)\n- **Input**:\n```json\n{\n  \"engine\": \"bing\",\n  \"prompt\": \"I want reviews for the \\\"macbook air M2\\\", but not the pro model.\"\n}\n```\n- **Output**:\n```json\n{\n  \"query\": \"\\\"macbook air M2\\\" review -pro\"\n}\n```\n\n### Example 3: File Type and Language Instruction (Google)\n- **Input**:\n```json\n{\n  \"engine\": \"google\",\n  \"prompt\": \"latest PDF reports on deep learning, must be in English\"\n}\n```\n- **Output**:\n```json\n{\n  \"query\": \"deep learning report filetype:pdf\"\n}\n```\n\n### Example 4: Preserving Code Error Messages (Google)\n- **Input**:\n```json\n{\n  \"engine\": \"google\",\n  \"prompt\": \"How to fix the python error 'IndentationError: unexpected indent'\"\n}\n```\n- **Output**:\n```json\n{\n  \"query\": \"python fix \\\"IndentationError: unexpected indent\\\"\"\n}\n```\n\n### Example 5: DuckDuckGo Specific Syntax\n- **Input**:\n```json\n{\n  \"engine\": \"duckduckgo\",\n  \"prompt\": \"Look up 'history of artificial intelligence' on Wikipedia\"\n}\n```\n- **Output**:\n```json\n{\n  \"query\": \"!w history of artificial intelligence\"\n}\n```\n",
            ),
            LLMPrompt::new(
                "model",
                "```json\n{\n  \"query\": \"!w history of artificial intelligence\"\n}\n```",
            ),
            // the user's query
            LLMPrompt::new("user", &serde_json::to_string_pretty(&user_query_request)?),
        ];

        // send prompt to ai
        let ai_response = self.llm.query(&self.llm_model, &contents).await;

        let Ok(mut content) = ai_response else {
            return Err(SearchError::Ai);
        };
        // parse response
        if content.starts_with("```json") {
            content = content
                .trim_start_matches("```json")
                .trim_end_matches("```")
                .to_string();
        }

        // parse json
        let response: GenerateSearchQueryResponse = serde_json::from_str(&content)?;

        let encoded_query = urlencoding::encode(&response.query);
        let url = search_engine.generate_url(&encoded_query.to_string());

        Ok(GenerateQueryResult { url })
    }
}
