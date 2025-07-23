use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("Serde error: {0}")]
    Serde(#[from] serde_json::Error),
}

#[derive(Deserialize, Debug)]
pub struct DiscordTokenResponse {
    pub access_token: String,
}

#[derive(Deserialize, Debug)]
pub struct DiscordUser {
    pub id: String,
    pub username: String,
}

#[async_trait]
pub trait AuthService: Send + Sync {
    fn get_discord_auth_url(&self) -> String;
    async fn exchange_code_for_token(&self, code: &str) -> Result<String, AuthError>;
    async fn get_discord_user(&self, access_token: &str) -> Result<DiscordUser, AuthError>;
}

pub struct AuthServiceImpl {
    discord_client_id: String,
    discord_client_secret: String,
    discord_redirect_uri: String,
    http_client: Client,
}

impl AuthServiceImpl {
    pub fn new(
        discord_client_id: String,
        discord_client_secret: String,
        discord_redirect_uri: String,
    ) -> Self {
        Self {
            discord_client_id,
            discord_client_secret,
            discord_redirect_uri,
            http_client: Client::new(),
        }
    }
}

#[async_trait]
impl AuthService for AuthServiceImpl {
    fn get_discord_auth_url(&self) -> String {
        format!(
            "https://discord.com/api/oauth2/authorize?client_id={}&redirect_uri={}&response_type=code&scope=identify",
            self.discord_client_id, self.discord_redirect_uri
        )
    }

    async fn exchange_code_for_token(&self, code: &str) -> Result<String, AuthError> {
        let params = [
            ("client_id", &self.discord_client_id),
            ("client_secret", &self.discord_client_secret),
            ("grant_type", &"authorization_code".to_string()),
            ("code", &code.to_string()),
            ("redirect_uri", &self.discord_redirect_uri),
        ];

        let response = self
            .http_client
            .post("https://discord.com/api/oauth2/token")
            .form(&params)
            .send()
            .await?
            .json::<DiscordTokenResponse>()
            .await?;

        Ok(response.access_token)
    }

    async fn get_discord_user(&self, access_token: &str) -> Result<DiscordUser, AuthError> {
        let response = self
            .http_client
            .get("https://discord.com/api/users/@me")
            .bearer_auth(access_token)
            .send()
            .await?
            .json::<DiscordUser>()
            .await?;

        Ok(response)
    }
}
