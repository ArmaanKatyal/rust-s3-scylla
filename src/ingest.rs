use axum::{
    debug_handler,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

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
pub async fn ingest_handler() -> Result<Json<IngestResponse>, AppError> {
    Ok(Json(IngestResponse::new(
        "OK".to_string(),
        "Ingested".to_string(),
    )))
}
