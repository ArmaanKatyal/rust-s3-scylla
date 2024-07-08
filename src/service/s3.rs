use aws_config::{meta::region::RegionProviderChain, BehaviorVersion, Region};
use aws_sdk_s3::{operation::get_object::GetObjectOutput, Client};
use axum::async_trait;
use log::info;

use crate::data::source_model::Logs;

use super::Ingestor;

const DEFAULT_REGION: &str = "us-west-2";

#[async_trait]
pub trait S3ClientTrait {
    async fn init(region: String) -> Self
    where
        Self: Sized;
    async fn get_object(&self, bucket: &str, key: &str) -> Result<GetObjectOutput, anyhow::Error>;
}

#[derive(Clone)]
pub struct S3Client {
    client: Client,
}

#[async_trait]
impl S3ClientTrait for S3Client {
    async fn init(region: String) -> Self {
        let region_provider = RegionProviderChain::first_try(Region::new(region))
            .or_default_provider()
            .or_else(Region::new(DEFAULT_REGION));
        let config = aws_config::defaults(BehaviorVersion::latest())
            .region(region_provider)
            .load()
            .await;
        let client = Client::new(&config);
        info!("initialized S3 client");
        Self { client }
    }

    async fn get_object(&self, bucket: &str, key: &str) -> Result<GetObjectOutput, anyhow::Error> {
        match self
            .client
            .get_object()
            .bucket(bucket)
            .key(key)
            .send()
            .await
        {
            Ok(res) => Ok(res),
            Err(e) => Err(anyhow::Error::msg(format!("Failed to get object: {}", e))),
        }
    }
}

pub struct S3Service<C>
where
    C: S3ClientTrait,
{
    pub client: C,
}

impl<C: S3ClientTrait> S3Service<C> {
    pub async fn init(region: String) -> Self {
        let client = C::init(region).await;
        Self { client }
    }
}

#[async_trait]
impl<C: S3ClientTrait + Send + Sync> Ingestor for S3Service<C> {
    async fn read_file(&self, bucket: &str, key: &str) -> Result<Logs, anyhow::Error> {
        let mut object = match self.client.get_object(bucket, key).await {
            Ok(res) => res,
            Err(e) => return Err(e),
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

#[cfg(test)]
mod tests {
    use aws_sdk_s3::primitives::ByteStream;
    use mockall::{mock, predicate::*};

    use crate::data::source_model::Log;

    use super::*;

    mock! {
        S3ClientTraitImpl {}

        #[async_trait]
        impl S3ClientTrait for S3ClientTraitImpl {
            async fn init(region: String) -> Self;
            async fn get_object(&self, bucket: &str, key: &str) -> Result<GetObjectOutput, anyhow::Error>;
        }
    }

    #[tokio::test]
    async fn test_s3_read_file_success() {
        let bucket = "test-bucket";
        let key = "test-key";
        let mut logs = Logs::new();
        logs.push(Log {
            event_type: Some("test_event".to_string()),
            log_id: Some(1),
            timestamp: Some("2021-01-01T00:00:00Z".to_string()),
            user_id: Some(1),
            page_url: Some("http://example.com".to_string()),
            ip_address: Some("1.1.1.1".to_string()),
            device_type: Some("desktop".to_string()),
            browser: Some("Chrome".to_string()),
            os: Some("Windows".to_string()),
            response_time: Some(0.1),
        });

        let logs_json = serde_json::to_vec(&logs).unwrap();

        let mut mock_s3_client = MockS3ClientTraitImpl::new();
        mock_s3_client
            .expect_get_object()
            .withf(move |b, k| b == bucket && k == key)
            .returning(move |_, _| {
                let byte_stream = ByteStream::from(logs_json.clone());
                let output = GetObjectOutput::builder().body(byte_stream).build();
                Ok(output)
            });

        let service = S3Service {
            client: mock_s3_client,
        };
        let result = service.read_file(bucket, key).await;
        assert!(result.is_ok());
        let logs = result.unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(
            logs[0].event_type.as_ref().expect("event_type missing"),
            "test_event"
        );
    }
}
