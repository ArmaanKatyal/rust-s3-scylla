use config::{Config, Environment, File, FileFormat};

#[derive(Debug, serde::Deserialize, Clone)]
pub struct Conf {
    pub host: String,
    pub port: u16,
    pub region: String,
    pub db_url: String,
    pub db_dc: String,
    pub parallel_files: usize,
    pub db_parallelism: usize,
    pub schema_file: String,
    pub use_s3: bool,
}

pub struct AppConfig {
    conf: Config,
}

impl AppConfig {
    pub fn init() -> Self {
        Self {
            conf: Config::new(),
        }
    }
    pub fn from_env(mut self) -> Self {
        dotenv::dotenv().ok();
        if let Err(e) = self.conf.merge(Environment::new()) {
            panic!("Failed to load env: {:?}", e)
        }
        self
    }
    pub fn from_file(mut self, filename: &str, file_format: FileFormat) -> Self {
        if let Err(e) = self.conf.merge(File::new(filename, file_format)) {
            panic!("Failed to load file {:?}: {:?}", filename, e);
        }
        self
    }
    pub fn parse(self) -> Conf {
        match self.conf.try_into() {
            Ok(conf) => conf,
            Err(e) => {
                panic!("Failed to construct config: {:?}", e)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{env, fs::write};

    use tempfile::NamedTempFile;

    use super::*;

    const TEST_CONFIG: &str = r#"
        host = "example.com"
        port = 9090
        region = "us-west-2"
        db_url = "postgres://user:pass@db/mydb"
        db_dc = "dc1"
        parallel_files = 4
        db_parallelism = 2
        schema_file = "schema.sql"
        use_s3 = true
        "#;

    #[test]
    fn test_init() {
        let app_config = AppConfig::init();
        assert!(app_config.conf.get_str("host").is_err())
    }

    #[test]
    fn test_from_env() {
        env::set_var("HOST", "localhost");
        env::set_var("PORT", "8080");

        let app_config = AppConfig::init().from_env();

        assert_eq!(app_config.conf.get_str("host").unwrap(), "localhost");
        assert_eq!(app_config.conf.get_str("port").unwrap(), "8080");

        env::remove_var("HOST");
        env::remove_var("PORT");
    }

    #[test]
    fn test_from_file() {
        let temp_file = NamedTempFile::new().unwrap();
        let _ = write(temp_file.path(), TEST_CONFIG);

        let app_config =
            AppConfig::init().from_file(temp_file.path().to_str().unwrap(), FileFormat::Toml);

        assert_eq!(app_config.conf.get_str("host").unwrap(), "example.com");
        assert_eq!(app_config.conf.get_int("port").unwrap(), 9090);
        assert_eq!(app_config.conf.get_str("region").unwrap(), "us-west-2");
    }

    #[test]
    fn test_parse() {
        let temp_file = NamedTempFile::new().unwrap();
        let _ = write(temp_file.path(), TEST_CONFIG);

        let conf = AppConfig::init()
            .from_file(temp_file.path().to_str().unwrap(), FileFormat::Toml)
            .parse();

        assert_eq!(conf.host, "example.com");
        assert_eq!(conf.port, 9090);
        assert_eq!(conf.region, "us-west-2");
        assert_eq!(conf.db_dc, "dc1")
    }

    #[test]
    #[should_panic(expected = "Failed to construct config")]
    fn test_parse_invalid_config() {
        let config_content = r#"
        host = "example.com"
        port = "invalid_port"
        "#;

        let temp_file = NamedTempFile::new().unwrap();
        let _ = write(temp_file.path(), config_content);

        AppConfig::init()
            .from_file(temp_file.path().to_str().unwrap(), FileFormat::Toml)
            .parse();
    }
}
