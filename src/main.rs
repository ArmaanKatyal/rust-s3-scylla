mod config;
mod data;
mod db;
mod ingest;
mod service;

use std::sync::Arc;

use crate::config::Config;
use crate::db::scylladb::ScyllaDbService;
use crate::ingest::ingest_handler;

use axum::{
    routing::{get, post},
    Router,
};
use log::info;
use service::{local::LocalService, s3::S3Service, Ingestor};
use tokio::net::TcpListener;
use tokio::sync::Semaphore;

#[derive(Clone)]
struct AppState {
    semaphore: Arc<Semaphore>,
    ingestor: Arc<dyn Ingestor>,
    db_svc: ScyllaDbService,
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let config = match Config::from_env() {
        Ok(config) => config,
        Err(e) => panic!("Error loading config: {}", e),
    };
    let ingestor: Arc<dyn Ingestor> = if config.use_s3 {
        info!("Using S3 ingestor");
        Arc::new(S3Service::init().await)
    } else {
        info!("Using local ingestor");
        Arc::new(LocalService::init())
    };
    info!("Starting server on {}:{}", config.host, config.port);
    let app = Router::new().route("/health", get(health)).route(
        "/ingest",
        post(ingest_handler).with_state(AppState {
            semaphore: Arc::new(Semaphore::new(config.parallel_files)),
            ingestor,
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
