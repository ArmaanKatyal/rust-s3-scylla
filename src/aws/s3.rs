use aws_config::{meta::region::RegionProviderChain, BehaviorVersion, Region};
use aws_sdk_s3::Client;

use crate::data::source_model::File;

#[derive(Clone)]
pub struct s3_service {
    pub client: Client,
}

impl s3_service {
    pub async fn init() -> Self {
        let region_provider =
            RegionProviderChain::default_provider().or_else(Region::new("us-west-2"));
        let config = aws_config::defaults(BehaviorVersion::latest())
            .region(region_provider)
            .load()
            .await;
        let client = Client::new(&config);
        Self { client }
    }

    pub async fn read_file(&self, bucket: String, key: String) -> Result<File, anyhow::Error> {
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
        let file: File = serde_json::from_slice(&bytes)?;
        Ok(file)
    }
}
