use aws_config::{meta::region::RegionProviderChain, BehaviorVersion, Region};
use aws_sdk_s3::Client;
use axum::async_trait;
use log::info;

use crate::data::source_model::Logs;

use super::Ingestor;

const DEFAULT_REGION: &str = "us-west-2";

#[derive(Clone)]
pub struct S3Service {
    pub client: Client,
}

impl S3Service {
    pub async fn init(region: String) -> Self {
        let region_provider = RegionProviderChain::first_try(Region::new(region))
            .or_else(DEFAULT_REGION)
            .or_default_provider();
        let config = aws_config::defaults(BehaviorVersion::latest())
            .region(region_provider)
            .load()
            .await;
        let client = Client::new(&config);
        info!("initialized s3 client; region: us-west-2");
        Self { client }
    }
}

#[async_trait]
impl Ingestor for S3Service {
    async fn read_file(&self, bucket: &str, key: &str) -> Result<Logs, anyhow::Error> {
        let mut object = match self
            .client
            .get_object()
            .bucket(bucket)
            .key(key)
            .send()
            .await
        {
            Ok(res) => res,
            Err(e) => return Err(anyhow::Error::msg(format!("Failed to get object: {}", e))),
        };
        let mut bytes = Vec::new();
        while let Some(chunk) = match object.body.try_next().await {
            Ok(chunk) => chunk,
            Err(e) => {
                return Err(anyhow::Error::msg(format!(
                    "Failed to consume ByteStream: {}",
                    e
                )))
            }
        } {
            bytes.extend_from_slice(&chunk)
        }
        let data: Logs = match serde_json::from_slice(&bytes) {
            Ok(data) => data,
            Err(e) => return Err(anyhow::Error::msg(format!("Failed to parse logs: {}", e))),
        };
        Ok(data)
    }
}
