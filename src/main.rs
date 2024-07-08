mod app_config;
mod data;
mod db;
mod ingest;
mod service;

use std::{process::exit, sync::Arc};

use crate::app_config::AppConfig;
use crate::db::scylladb::ScyllaDbService;
use crate::ingest::ingest_handler;

use axum::{
    routing::{get, post},
    Router,
};
use log::{debug, error, info};
use service::Ingestor;
use tokio::net::TcpListener;
use tokio::sync::Semaphore;

#[derive(Clone)]
struct AppState {
    semaphore: Arc<Semaphore>,
    ingest: Arc<dyn Ingestor>,
    db_svc: ScyllaDbService,
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let config = AppConfig::init()
        .load_from_env()
        .load_from_file("config.toml", config::FileFormat::Toml)
        .parse();
    let ingest: Arc<dyn Ingestor> = if config.use_s3 {
        info!("Using S3 ingestor");
        Arc::new(service::get_s3_service(config.region).await)
    } else {
        info!("Using local ingestor");
        Arc::new(service::get_local_service())
    };
    info!("Starting server on {}:{}", config.host, config.port);
    let app = Router::new().route("/health", get(health)).route(
        "/ingest",
        post(ingest_handler).with_state(AppState {
            semaphore: Arc::new(Semaphore::new(config.parallel_files)),
            ingest,
            db_svc: ScyllaDbService::new(
                config.db_dc.as_str(),
                config.db_url.as_str(),
                config.db_parallelism,
                config.schema_file.as_str(),
            )
            .await,
        }),
    );
    let listener = match TcpListener::bind(format!("{}:{}", config.host, config.port)).await {
        Ok(l) => l,
        Err(e) => {
            error!("Failed to bind listner on port {:?} {:?}", config.port, e);
            exit(1)
        }
    };
    match axum::serve(listener, app).await {
        Ok(_) => debug!("Server started on port: {:?}", config.port),
        Err(e) => {
            error!("Server failed to start: {:?}", e);
            exit(1)
        }
    };
}

async fn health() -> &'static str {
    "OK"
}
