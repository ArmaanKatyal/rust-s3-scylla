use axum::async_trait;

use crate::data::source_model::Logs;

use super::Ingestor;

pub struct LocalService {}

impl LocalService {
    pub fn init() -> Self {
        Self {}
    }
}

#[async_trait]
impl Ingestor for LocalService {
    async fn read_file(&self, bucket: &str, key: &str) -> Result<Logs, anyhow::Error> {
        // bucket is mapped to a local folder and key is the file name
        let contents = match std::fs::read_to_string(format!("./{}/{}", bucket, key)) {
            Ok(contents) => contents,
            Err(e) => return Err(anyhow::Error::msg(format!("Error reading file: {}", e))),
        };
        let data: Logs = match serde_json::from_str(&contents) {
            Ok(data) => data,
            Err(e) => return Err(anyhow::Error::msg(format!("Error parsing log file: {}", e))),
        };
        Ok(data)
    }
}
