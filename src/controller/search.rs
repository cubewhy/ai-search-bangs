use std::env;

use actix_web::{
    HttpRequest, HttpResponse, Responder, Scope,
    body::BoxBody,
    get,
    web::{self, Redirect},
};

use crate::{
    llm::Gemini,
    model::AiSearchQuery,
    service::search::{SearchService, SearchServiceImpl},
};

struct Context {
    pub search_service: Box<dyn SearchService>,
}

#[get("ai")]
async fn ai_search(
    req: HttpRequest,
    query: web::Query<AiSearchQuery>,
    context: web::Data<Context>,
) -> impl Responder {
    let request = query.into_inner();
    // do query
    let Some(query) = request.q else {
        return HttpResponse::BadRequest().body("no search content provided");
    };
    let Some(search_engine) = request.service else {
        return HttpResponse::BadRequest().body("no search engine provided");
    };

    let result = context
        .search_service
        .generate_query(&query, &search_engine)
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
    let api = env::var("GEMINI_API").unwrap_or("https://generativelanguage.googleapis.com".to_string());
    let api_key = env::var("GEMINI_KEY").expect("No api key provided");
    let llm = Gemini::new(&api, &api_key);
    let llm_model = "gemini-2.5-flash-lite-preview-06-17";

    let context = Context {
        search_service: Box::new(SearchServiceImpl::new(Box::new(llm), llm_model.to_string())),
    };
    Scope::new("/search")
        .app_data(web::Data::new(context))
        .service(ai_search)
}
