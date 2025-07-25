use std::{env, sync::Arc};

use actix_session::Session;
use actix_web::{
    body::BoxBody,
    get,
    web::{self, Redirect},
    HttpRequest, HttpResponse, Responder, Scope,
};
use governor::{clock::DefaultClock, RateLimiter, state::direct::NotKeyed, state::InMemoryState};

use crate::{
    model::AiSearchQuery,
    service::search::SearchService,
};

#[get("ai")]
async fn ai_search(
    req: HttpRequest,
    query: web::Query<AiSearchQuery>,
    search_service: web::Data<Arc<dyn SearchService>>,
    // rate_limiter: web::Data<Option<Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>>>,
    // pool: web::Data<SqlitePool>,
    // session: Session,
) -> impl Responder {
    // let user_id = match session.get::<i64>("user_id") {
    //     Ok(Some(id)) => id,
    //     _ => {
    //         return HttpResponse::Found()
    //             .append_header(("Location", "/login.html"))
    //             .finish();
    //     }
    // };

    // if let Some(limiter) = rate_limiter.as_ref() {
    //     if let Err(_) = limiter.check() {
    //         return HttpResponse::TooManyRequests().body("Too many requests");
    //     }
    // }

    // let daily_limit: i32 = env::var("DAILY_REQUEST_LIMIT")
    //     .unwrap_or("50".to_string())
    //     .parse()
    //     .unwrap_or(50);

    // let mut user = match sqlx::query!("SELECT * FROM users WHERE id = ?", user_id)
    //     .fetch_one(pool.get_ref())
    //     .await
    // {
    //     Ok(user) => user,
    //     Err(e) => {
    //         log::error!("Failed to fetch user: {}", e);
    //         return HttpResponse::InternalServerError().finish();
    //     }
    // };

    // let today = chrono::Utc::now().date_naive();
    // if user.last_request_date != today {
    //     user.request_count = 0;
    //     user.last_request_date = today;
    // }

    // if user.request_count >= daily_limit as i64 {
    //     return HttpResponse::TooManyRequests().body("Daily request limit exceeded");
    // }

    let request = query.into_inner();
    let Some(query) = request.q else {
        return HttpResponse::BadRequest().body("no search content provided");
    };
    let search_engine = request.engine.unwrap_or("google".to_string());
    let language = request.language.unwrap_or("English".to_string());

    let result = search_service
        .generate_query(&query, &search_engine, &language)
        .await;

    let result = match result {
        Ok(result) => result,
        Err(err) => return HttpResponse::InternalServerError().body(format!("{err:?}")),
    };

    // match sqlx::query!(
    //     "UPDATE users SET request_count = request_count + 1, last_request_date = ? WHERE id = ?",
    //     today,
    //     user_id
    // )
    // .execute(pool.get_ref())
    // .await
    // {
    //     Ok(_) => (),
    //     Err(e) => {
    //         log::error!("Failed to update user request count: {}", e);
    //         // Decide if you want to fail the whole request here
    //     }
    // }

    Redirect::to(result.url)
        .temporary()
        .respond_to(&req)
        .set_body(BoxBody::new("307 Temporary Redirect"))
}

pub fn service() -> Scope {
    web::scope("/search").service(ai_search)
}
