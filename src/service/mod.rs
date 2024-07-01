use axum::async_trait;

use crate::data::source_model::Logs;

pub mod local;
pub mod s3;

#[async_trait]
pub trait Ingestor: Send + Sync {
    async fn read_file(&self, bucket: String, key: String) -> Result<Logs, anyhow::Error>;
}
