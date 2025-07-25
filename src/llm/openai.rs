use async_trait::async_trait;
use openai::{
    Credentials,
    chat::{ChatCompletion, ChatCompletionMessage, ChatCompletionMessageRole},
};

use crate::llm::{LLMError, LLMPrompt, LargeLanguageModel};

pub struct OpenAI {
    credentials: Credentials,
}

impl OpenAI {
    pub fn new(api_base: &str, token: &str) -> Self {
        let credentials = Credentials::new(token, api_base);
        Self { credentials }
    }
}

#[async_trait]
impl LargeLanguageModel for OpenAI {
    async fn query(&self, model: &str, contents: &Vec<LLMPrompt>) -> Result<String, LLMError> {
        let messages: Vec<_> = contents
            .iter()
            .enumerate()
            .map(|(i, content)| {
                let role = if i == 0 {
                    ChatCompletionMessageRole::Developer
                } else {
                    match content.role.as_str() {
                        "assistant" => ChatCompletionMessageRole::Assistant,
                        "user" => ChatCompletionMessageRole::User,
                        role => return Err(LLMError::BadRole(role.to_string())),
                    }
                };
                Ok(ChatCompletionMessage {
                    role,
                    function_call: None,
                    name: None,
                    content: Some(content.content.to_owned()),
                    ..Default::default()
                })
            })
            .filter(|item| item.is_ok())
            .map(|item| item.unwrap())
            .collect();

        let chat_completion = ChatCompletion::builder(model, messages.clone())
            .credentials(self.credentials.clone())
            .create()
            .await?;

        let returned_message = chat_completion.choices.first().unwrap().message.clone();
        returned_message
            .content
            .map(|s| s.trim().to_string())
            .ok_or(LLMError::EmptyResponse)
    }
}
