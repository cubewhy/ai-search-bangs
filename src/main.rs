use std::env;

use actix_web::{App, HttpServer};
use log::info;

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

    info!("Start AI search bangs service at {host}:{port}");

    HttpServer::new(|| App::new().service(controller::search::scope()))
        .bind((host, port))?
        .run()
        .await?;
    Ok(())
}
