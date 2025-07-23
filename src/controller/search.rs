use std::sync::Arc;

use actix_web::{
    body::BoxBody,
    get,
    web::{self, Redirect},
    HttpRequest, HttpResponse, Responder, Scope,
};
use governor::RateLimiter;

use crate::{
    model::AiSearchQuery,
    service::search::SearchService,
};

#[get("ai")]
async fn ai_search(
    req: HttpRequest,
    query: web::Query<AiSearchQuery>,
    search_service: web::Data<Arc<dyn SearchService>>,
    rate_limiter: web::Data<Option<Arc<RateLimiter>>>,
) -> impl Responder {
    if let Some(limiter) = rate_limiter.get_ref() {
        if let Err(_) = limiter.check() {
            return HttpResponse::TooManyRequests().body("Too many requests");
        }
    }
>>>>>>>
[Response interrupted by user]
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

    Redirect::to(result.url)
        .temporary()
        .respond_to(&req)
        .set_body(BoxBody::new("307 Temporary Redirect"))
}

pub fn scope() -> Scope {
    Scope::new("/search").service(ai_search)
}
