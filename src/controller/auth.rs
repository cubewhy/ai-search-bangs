use std::{env, sync::Arc};

use actix_session::Session;
use actix_web::{get, web, HttpResponse, Responder, Scope};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use crate::service::{auth::AuthService, turnstile::TurnstileService};

#[derive(Serialize)]
struct ConfigResponse {
    turnstile_site_key: String,
}

#[derive(Deserialize)]
pub struct AuthQuery {
    #[serde(rename = "cf-turnstile-response")]
    turnstile_response: String,
}

#[derive(Deserialize)]
pub struct CallbackQuery {
    code: String,
}

pub fn service() -> Scope {
    web::scope("/auth")
        .service(config)
        .service(me)
        .service(discord_login)
        .service(discord_callback)
        .service(logout)
}

#[get("/config")]
async fn config() -> impl Responder {
    let site_key = env::var("CLOUDFLARE_TURNSTILE_SITE_KEY")
        .expect("CLOUDFLARE_TURNSTILE_SITE_KEY must be set");
    HttpResponse::Ok().json(ConfigResponse {
        turnstile_site_key: site_key,
    })
}

#[get("/me")]
async fn me(session: Session, pool: web::Data<SqlitePool>) -> impl Responder {
    match session.get::<i64>("user_id") {
        Ok(Some(user_id)) => {
            match sqlx::query!("SELECT username FROM users WHERE id = ?", user_id)
                .fetch_one(pool.get_ref())
                .await
            {
                Ok(user) => HttpResponse::Ok().json(serde_json::json!({ "username": user.username })),
                Err(_) => HttpResponse::InternalServerError().finish(),
            }
        }
        _ => HttpResponse::Unauthorized().finish(),
    }
}

#[get("/discord/login")]
async fn discord_login(
    query: web::Query<AuthQuery>,
    auth_service: web::Data<Arc<dyn AuthService>>,
    turnstile_service: web::Data<Arc<TurnstileService>>,
) -> impl Responder {
    let is_valid = match turnstile_service.verify(&query.turnstile_response).await {
        Ok(valid) => valid,
        Err(e) => {
            log::error!("Failed to verify turnstile token: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    if !is_valid {
        return HttpResponse::BadRequest().body("Invalid Turnstile token");
    }

    let discord_auth_url = auth_service.get_discord_auth_url();
    HttpResponse::Found()
        .append_header(("Location", discord_auth_url))
        .finish()
}

#[get("/discord/callback")]
async fn discord_callback(
    query: web::Query<CallbackQuery>,
    auth_service: web::Data<Arc<dyn AuthService>>,
    pool: web::Data<SqlitePool>,
    session: Session,
) -> impl Responder {
    let access_token = match auth_service.exchange_code_for_token(&query.code).await {
        Ok(token) => token,
        Err(e) => {
            log::error!("Failed to exchange code for token: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let discord_user = match auth_service.get_discord_user(&access_token).await {
        Ok(user) => user,
        Err(e) => {
            log::error!("Failed to get discord user: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let user_id: i64 = discord_user.id.parse().unwrap();

    let today = chrono::Utc::now().date_naive();
    match sqlx::query!(
        "INSERT INTO users (id, username, last_request_date) VALUES (?, ?, ?)
         ON CONFLICT(id) DO UPDATE SET username = excluded.username",
        user_id,
        discord_user.username,
        today
    )
    .execute(pool.get_ref())
    .await
    {
        Ok(_) => (),
        Err(e) => {
            log::error!("Failed to insert or update user: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    }

    session.insert("user_id", user_id).unwrap();

    HttpResponse::Found()
        .append_header(("Location", "/"))
        .finish()
}

#[get("/logout")]
async fn logout(session: Session) -> impl Responder {
    session.clear();
    HttpResponse::Found()
        .append_header(("Location", "/"))
        .finish()
}
