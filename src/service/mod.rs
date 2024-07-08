use axum::async_trait;
use local::LocalService;
use s3::{S3Client, S3Service};

use crate::data::source_model::Logs;

mod local;
mod s3;

#[async_trait]
pub trait Ingestor: Send + Sync {
    async fn read_file(&self, bucket: &str, key: &str) -> Result<Logs, anyhow::Error>;
}

pub async fn get_s3_service(region: String) -> S3Service<S3Client> {
    let service = S3Service::<S3Client>::init(region).await;
    return service;
}

pub fn get_local_service() -> LocalService {
    return LocalService::init();
}
