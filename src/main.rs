mod aws;
mod config;
mod data;

use crate::config::Config;
use axum::{routing::get, Router};
use log::info;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    env_logger::init();
    let config = match Config::from_env() {
        Ok(config) => config,
        Err(e) => panic!("Error loading config: {}", e),
    };

    info!("Starting server on {}:{}", config.host, config.port);
    let app = Router::new().route("/health", get(health));
    let listner = TcpListener::bind(format!("{}:{}", config.host, config.port))
        .await
        .unwrap();
    axum::serve(listner, app).await.unwrap();
}

async fn health() -> &'static str {
    "OK"
}
