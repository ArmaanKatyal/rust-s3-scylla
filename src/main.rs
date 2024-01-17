mod aws;
mod config;
mod data;
mod db;
mod ingest;

use std::sync::Arc;

use crate::db::scylladb::ScyllaDbService;
use crate::ingest::ingest_handler;
use crate::{aws::s3::S3Service, config::Config};

use axum::{
    routing::{get, post},
    Router,
};
use log::info;
use tokio::net::TcpListener;
use tokio::sync::Semaphore;

#[derive(Clone)]
struct AppState {
    semaphore: Arc<Semaphore>,
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
            semaphore: Arc::new(Semaphore::new(config.parallel_files)),
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
