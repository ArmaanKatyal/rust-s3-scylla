use std::path::Path;

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
        let file_path = Path::new(bucket).join(key);
        let contents = match std::fs::read_to_string(file_path) {
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

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::*;

    fn create_test_file(dir: &Path, filename: &str, contents: &str) -> std::io::Result<()> {
        let file_path = dir.join(filename);
        fs::write(file_path, contents)
    }

    #[tokio::test]
    async fn test_read_file_sucess() {
        let temp_dir = TempDir::new().unwrap();
        let bucket = temp_dir.path().to_str().unwrap();
        let key = "testfile.json";
        let contents = r#"[{"event_type":"test_event"}]"#;

        create_test_file(temp_dir.path(), key, contents).unwrap();

        let local_service = LocalService::init();
        let result = local_service.read_file(bucket, key).await;
        assert!(result.is_ok());
        let logs = result.unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(
            logs[0].event_type.as_ref().expect("event_type missing"),
            "test_event"
        )
    }

    #[tokio::test]
    async fn test_read_file_json_err() {
        let temp_dir = TempDir::new().unwrap();
        let bucket = temp_dir.path().to_str().unwrap();
        let key = "testfile.json";
        let contents = "file content is not json";

        create_test_file(temp_dir.path(), key, contents).unwrap();

        let local_service = LocalService::init();
        let result = local_service.read_file(bucket, key).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Error parsing log file"))
    }

    #[tokio::test]
    async fn test_read_file_not_found() {
        let local_service = LocalService::init();
        let temp_dir = TempDir::new().unwrap();
        let bucket = temp_dir.path().to_str().unwrap();
        let result = local_service.read_file(bucket, "testfile").await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Error reading file"))
    }
}
