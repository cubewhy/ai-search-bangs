use std::string::FromUtf8Error;

use async_trait::async_trait;
use ::openai::OpenAiError;
use reqwest::Client;
use thiserror::Error;

pub mod openai;

#[derive(Error, Debug)]
pub enum LLMError {
    #[error("Error request AI")]
    Http(#[from] reqwest::Error),

    #[error("Error decoding response")]
    Decoding(#[from] FromUtf8Error),

    #[error("Failed to deserialize")]
    Serde(#[from] serde_json::Error),

    #[error("Bad role {0}")]
    BadRole(String),

    #[error("OpenAI error")]
    OpenAI(#[from] OpenAiError),

    #[error("Empty response from ai provider")]
    EmptyResponse,
}

#[derive(Clone, Debug)]
pub struct LLMPrompt {
    pub role: String,
    pub content: String,
}

impl LLMPrompt {
    pub fn new(role: &str, content: &str) -> Self {
        Self {
            role: role.to_string(),
            content: content.to_string(),
        }
    }
}

#[async_trait]
pub trait LargeLanguageModel: Send + Sync {
    async fn query(&self, model: &str, contents: &Vec<LLMPrompt>) -> Result<String, LLMError>;
}

#[derive(Clone, Debug)]
pub struct Gemini {
    client: Client,
    api: String,
    api_key: String,
    temperature: f32,
}

impl Gemini {
    pub fn new(api: String, api_key: String, temperature: f32) -> Self {
        Self {
            client: Client::new(),
            api,
            api_key,
            temperature,
        }
    }
}

#[async_trait]
impl LargeLanguageModel for Gemini {
    async fn query(&self, model_id: &str, contents: &Vec<LLMPrompt>) -> Result<String, LLMError> {
        let request_body = GeminiGenerateRequest {
            contents: contents
                .iter()
                .map(|prompt| GeminiPrompt::from(prompt))
                .collect(),
            generation_config: GeminiGenerationConfig {
                response_mime_type: "application/json".to_string(),
                temperature: self.temperature,
            },
        };

        let response_text = self
            .client
            .post(format!(
                "{}/v1beta/models/{}:generateContent",
                self.api, model_id
            ))
            .header("x-goog-api-key", &self.api_key)
            .json(&request_body)
            .send()
            .await?
            .text()
            .await?;

        // parse content_buf
        let response: serde_json::Value = serde_json::from_str(&response_text)?;

        let mut content_buf = String::new();

        // TODO: replace unwrap
        for part in response
            .as_object()
            .unwrap()
            .get("candidates")
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
        {
            let candidate = part.as_object().unwrap();
            let content = candidate.get("content").unwrap().as_object().unwrap();
            let result: Vec<String> = content
                .get("parts")
                .unwrap()
                .as_array()
                .unwrap()
                .iter()
                .map(|part| {
                    part.as_object()
                        .unwrap()
                        .get("text")
                        .unwrap()
                        .as_str()
                        .unwrap()
                        .to_string()
                })
                .collect();
            result.iter().for_each(|item| content_buf.push_str(item));
        }

        Ok(content_buf)
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
struct GeminiGenerateRequest {
    pub contents: Vec<GeminiPrompt>,
    #[serde(rename = "generationConfig")]
    pub generation_config: GeminiGenerationConfig,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
struct GeminiPrompt {
    pub role: String,
    pub parts: Vec<GeminiPromptPart>,
}

impl From<&LLMPrompt> for GeminiPrompt {
    fn from(value: &LLMPrompt) -> Self {
        Self {
            role: value.role.to_owned(),
            parts: vec![GeminiPromptPart::new(&value.content)],
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
struct GeminiPromptPart {
    text: String,
}

impl GeminiPromptPart {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
struct GeminiGenerationConfig {
    #[serde(rename = "responseMimeType")]
    response_mime_type: String,
    temperature: f32,
}

mod gemini_response {
    use serde::{Deserialize, Serialize};

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Root {
        pub candidates: Vec<Candidate>,
        pub usage_metadata: UsageMetadata,
        pub model_version: String,
        pub response_id: String,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Candidate {
        pub content: Content,
        pub finish_reason: String,
        pub index: i64,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Content {
        pub parts: Vec<Part>,
        pub role: String,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Part {
        pub text: String,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct UsageMetadata {
        pub prompt_token_count: i64,
        pub candidates_token_count: i64,
        pub total_token_count: i64,
        pub prompt_tokens_details: Vec<PromptTokensDetail>,
        pub thoughts_token_count: i64,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct PromptTokensDetail {
        pub modality: String,
        pub token_count: i64,
    }
}
