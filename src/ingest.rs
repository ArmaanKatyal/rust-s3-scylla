use axum::{
    debug_handler,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use log::info;
use serde_json::json;

use crate::AppState;

#[derive(Debug)]
#[allow(dead_code)]
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

#[debug_handler]
pub async fn ingest_handler(
    State(s): State<AppState>,
    Json(payload): Json<IngestRequest>,
) -> Result<Json<IngestResponse>, AppError> {
    let logs =
        s.s3.read_file(payload.bucket, payload.files[0].clone())
            .await
            .unwrap();
    info!("logs: {:?}", logs);

    Ok(Json(IngestResponse::new(
        "OK".to_string(),
        "Ingested".to_string(),
    )))
}
