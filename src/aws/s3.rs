use aws_config::{meta::region::RegionProviderChain, BehaviorVersion, Region};
use aws_sdk_s3::Client;
use log::info;

use crate::data::source_model::Logs;

#[derive(Clone)]
pub struct S3Service {
    pub client: Client,
}

impl S3Service {
    pub async fn init() -> Self {
        let region_provider =
            RegionProviderChain::default_provider().or_else(Region::new("us-west-2"));
        let config = aws_config::defaults(BehaviorVersion::latest())
            .region(region_provider)
            .load()
            .await;
        let client = Client::new(&config);
        info!("initialized s3 client; region: us-west-2");
        Self { client }
    }

    pub async fn read_file(&self, bucket: String, key: String) -> Result<Logs, anyhow::Error> {
        let mut object = self
            .client
            .get_object()
            .bucket(bucket)
            .key(key)
            .send()
            .await?;
        let mut bytes = Vec::new();
        while let Some(chunk) = object.body.try_next().await? {
            bytes.extend_from_slice(&chunk);
        }
        let data: Logs = serde_json::from_slice(&bytes)?;
        Ok(data)
    }
}
