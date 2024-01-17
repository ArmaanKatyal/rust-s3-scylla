use std::time::Instant;

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use log::{debug, info};
use serde_json::json;
use tokio::{
    sync::{AcquireError, OwnedSemaphorePermit},
    task::{self, JoinHandle},
};
use uuid::Uuid;

use crate::{
    data::source_model::{LogEntries, LogEntry, Logs},
    AppState,
};

#[derive(Debug)]
pub enum AppError {
    InternalServerError,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::InternalServerError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
        };
        let body = Json(json!({ "error": error_message }));
        (status, body).into_response()
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct IngestRequest {
    pub ingestion_id: String,
    pub bucket: String,
    pub files: Vec<String>,
}

#[derive(serde::Serialize)]
pub struct IngestResponse {
    status: String,
    message: String,
}

impl IngestResponse {
    pub fn new(status: String, message: String) -> Self {
        Self { status, message }
    }
}

pub async fn ingest_handler(
    State(s): State<AppState>,
    Json(payload): Json<IngestRequest>,
) -> Result<Json<IngestResponse>, AppError> {
    let now = Instant::now();
    let mut handlers: Vec<JoinHandle<_>> = Vec::new();

    for file in payload.files.iter() {
        let permit = s.semaphore.clone().acquire_owned().await;
        handlers.push(task::spawn(process_file(
            payload.ingestion_id.clone(),
            s.clone(),
            payload.bucket.to_string(),
            file.to_string(),
            permit,
        )));
    }

    debug!("Waiting for files to be processed");
    for thread in handlers {
        match thread.await {
            Err(_) => return Err(AppError::InternalServerError),
            Ok(r) => {
                debug!("Thread finished: {:?}", r);
            }
        }
    }
    let elapsed = now.elapsed();
    info!(
        "ingestion_id: {}; ingestion_time: {:.2?}",
        payload.ingestion_id, elapsed
    );

    Ok(Json(IngestResponse::new(
        "OK".to_string(),
        "Ingested".to_string(),
    )))
}

async fn process_file(
    ingestion_id: String,
    state: AppState,
    bucket: String,
    file: String,
    permit: Result<OwnedSemaphorePermit, AcquireError>,
) -> Result<(), anyhow::Error> {
    info!("Processing file {file} for provider {ingestion_id}. Reading file...");
    let now = Instant::now();
    let logs = state.s3.read_file(bucket, file.clone()).await.unwrap();
    info!("Logs processed, logs size: {}. Persisting...", logs.len());
    state
        .db_svc
        .insert(transform_logs(ingestion_id, logs))
        .await?;
    info!("logs persisted!");
    let elapsed = now.elapsed();
    info!("File {} processed in {:.2?}", file, elapsed);
    let _permit = permit;
    Ok(())
}

fn transform_logs(ingest_id: String, logs: Logs) -> LogEntries {
    let mut entries = Vec::new();
    for log in logs {
        let entry = LogEntry {
            id: Uuid::new_v4().to_string(),
            ingestion_id: ingest_id.clone(),
            timestamp: log.timestamp.unwrap_or("".to_string()),
            user_id: log.user_id.unwrap_or(0),
            event_type: log.event_type.unwrap_or("".to_string()),
            page_url: log.page_url.unwrap_or("".to_string()),
            ip_address: log.ip_address.unwrap_or("".to_string()),
            device_type: log.device_type.unwrap_or("".to_string()),
            browser: log.browser.unwrap_or("".to_string()),
            os: log.os.unwrap_or("".to_string()),
            response_time: log.response_time.unwrap_or(0.0),
        };
        entries.push(entry);
    }
    entries
}
