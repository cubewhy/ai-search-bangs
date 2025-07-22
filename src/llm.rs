use std::string::FromUtf8Error;

use async_trait::async_trait;
use futures_util::StreamExt;
use reqwest::Client;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LLMError {
    #[error("Error request AI")]
    Http(#[from] reqwest::Error),

    #[error("Error decoding response")]
    Decoding(#[from] FromUtf8Error),

    #[error("Failed to deserialize")]
    Serde(#[from] serde_json::Error),
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
}

impl Gemini {
    pub fn new(api: &str, api_key: &str) -> Self {
        Self {
            client: Client::new(),
            api: api.to_string(),
            api_key: api_key.to_string(),
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
                response_mime_type: "text/plain".to_string(),
                thinking_config: GeminiThinkingConfig {
                    thinking_budget: -1,
                },
            },
        };

        let mut stream = self.client.post(format!(
            "{}/v1beta/models/{}:streamGenerateContent?key={}",
            self.api,
            model_id,
            self.api_key
        )).json(&request_body)
        .send()
        .await?
        .bytes_stream();

        let mut response_buf = String::new();

        // receive content from stream
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            response_buf.push_str(&String::from_utf8(chunk.to_vec())?);
        }

        // parse content_buf
        let response: serde_json::Value = serde_json::from_str(&response_buf)?;

        let mut content_buf = String::new();

        for part in response.as_array().unwrap().into_iter() {
            // TODO: replace unwrap
            for part in part
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
    #[serde(rename = "thinkingConfig")]
    thinking_config: GeminiThinkingConfig,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
struct GeminiThinkingConfig {
    #[serde(rename = "thinkingBudget")]
    thinking_budget: i32,
}
