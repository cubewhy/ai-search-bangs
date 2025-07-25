use std::{env, num::NonZeroU32, sync::Arc};

use actix_files as fs;
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::{cookie::Key, web, App, HttpServer};
use governor::{clock::DefaultClock, Quota, RateLimiter};
use llm::Gemini;
use log::info;
use service::{auth::AuthServiceImpl, search::SearchServiceImpl, turnstile::TurnstileService};
use sqlx::SqlitePool;

mod controller;
pub mod llm;
pub mod model;
pub mod service;

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv()?;
    env_logger::init();

    let host: String = env::var("SEARCH_BANGS_HOST").unwrap_or("0.0.0.0".to_string());
    let port: u16 = env::var("SEARCH_BANGS_PORT")
        .unwrap_or("8080".to_string())
        .parse()
        .expect("Failed to parse port");

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = SqlitePool::connect(&database_url).await?;

    let session_secret_key = env::var("SESSION_SECRET_KEY").expect("SESSION_SECRET_KEY must be set");
    let session_key = Key::from(session_secret_key.as_bytes());

    let gemini_key = env::var("GEMINI_KEY").expect("Gemini key is not set");
    let gemini_api =
        env::var("GEMINI_API").unwrap_or("https://generativelanguage.googleapis.com".to_string());
    let temperature: f32 = env::var("TEMPERATURE")
        .unwrap_or("0".to_string())
        .parse()
        .expect("Failed to parse temperature");
    let prompt_file = env::var("PROMPT_FILE").unwrap_or("prompt.md".to_string());
    let requests_per_minute: i32 = env::var("REQUESTS_PER_MINUTE")
        .unwrap_or("-1".to_string())
        .parse()
        .expect("Failed to parse requests per minute");

    let rate_limiter: Option<Arc<RateLimiter<governor::state::direct::NotKeyed, governor::state::InMemoryState, DefaultClock>>> = if requests_per_minute > 0 {
        let quota = Quota::per_minute(NonZeroU32::new(requests_per_minute as u32).unwrap());
        Some(Arc::new(RateLimiter::direct(quota)))
    } else {
        None
    };

    let llm_model = env::var("LLM_MODEL").unwrap_or("gemini-1.5-flash".to_string());

    let llm = Gemini::new(gemini_api, gemini_key, temperature);
    let search_service: Arc<dyn service::search::SearchService> = Arc::new(SearchServiceImpl::new(
        Box::new(llm),
        llm_model,
        prompt_file,
        pool.clone(),
    ));

    let discord_client_id = env::var("DISCORD_CLIENT_ID").expect("DISCORD_CLIENT_ID must be set");
    let discord_client_secret =
        env::var("DISCORD_CLIENT_SECRET").expect("DISCORD_CLIENT_SECRET must be set");
    let discord_redirect_uri =
        env::var("DISCORD_REDIRECT_URI").expect("DISCORD_REDIRECT_URI must be set");
    let auth_service: Arc<dyn service::auth::AuthService> = Arc::new(AuthServiceImpl::new(
        discord_client_id,
        discord_client_secret,
        discord_redirect_uri,
    ));

    let turnstile_secret_key =
        env::var("CLOUDFLARE_TURNSTILE_SECRET_KEY").expect("CLOUDFLARE_TURNSTILE_SECRET_KEY must be set");
    let turnstile_service = Arc::new(TurnstileService::new(turnstile_secret_key));

    info!("Start AI search bangs service at {host}:{port}");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(search_service.clone()))
            .app_data(web::Data::new(auth_service.clone()))
            .app_data(web::Data::new(turnstile_service.clone()))
            .app_data(web::Data::new(rate_limiter.clone()))
            .wrap(SessionMiddleware::new(
                CookieSessionStore::default(),
                session_key.clone(),
            ))
            .service(controller::auth::service())
            .service(controller::search::service())
            .service(fs::Files::new("/", "./static").index_file("index.html"))
    })
    .bind((host, port))?
    .run()
    .await?;
    Ok(())
}
