use reqwest::Client;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TurnstileError {
    #[error("Reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
}

#[derive(Serialize)]
struct TurnstileRequest<'a> {
    secret: &'a str,
    response: &'a str,
}

#[derive(Deserialize)]
pub struct TurnstileResponse {
    pub success: bool,
}

pub struct TurnstileService {
    secret_key: String,
    http_client: Client,
}

impl TurnstileService {
    pub fn new(secret_key: String) -> Self {
        Self {
            secret_key,
            http_client: Client::new(),
        }
    }

    pub async fn verify(&self, token: &str) -> Result<bool, TurnstileError> {
        let request = TurnstileRequest {
            secret: &self.secret_key,
            response: token,
        };

        let response = self
            .http_client
            .post("https://challenges.cloudflare.com/turnstile/v0/siteverify")
            .form(&request)
            .send()
            .await?
            .json::<TurnstileResponse>()
            .await?;

        Ok(response.success)
    }
}
