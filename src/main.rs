mod aws;
mod config;
mod data;
mod db;
mod ingest;

use crate::ingest::ingest_handler;
use crate::{aws::s3::S3Service, config::Config};

use axum::{
    routing::{get, post},
    Router,
};
use db::scylladb::ScyllaDbService;
use log::info;
use tokio::net::TcpListener;

#[derive(Clone)]
#[allow(dead_code)]
struct AppState {
    s3: S3Service,
    db_svc: ScyllaDbService,
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let config = match Config::from_env() {
        Ok(config) => config,
        Err(e) => panic!("Error loading config: {}", e),
    };

    info!("Starting server on {}:{}", config.host, config.port);
    let app = Router::new().route("/health", get(health)).route(
        "/ingest",
        post(ingest_handler).with_state(AppState {
            s3: S3Service::init().await,
            db_svc: ScyllaDbService::new(
                config.db_dc,
                config.db_url,
                config.db_parallelism,
                config.schema_file,
            )
            .await,
        }),
    );
    let listner = TcpListener::bind(format!("{}:{}", config.host, config.port))
        .await
        .unwrap();
    axum::serve(listner, app).await.unwrap();
}

async fn health() -> &'static str {
    "OK"
}
