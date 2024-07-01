#[derive(Debug, serde::Deserialize, Clone)]
pub struct Config {
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

impl Config {
    pub fn from_env() -> Result<Self, config::ConfigError> {
        dotenv::dotenv().ok();
        let mut c = config::Config::new();
        c.merge(config::Environment::new())?;
        c.merge(config::File::new("config.json", config::FileFormat::Json))?;
        c.try_into()
    }
}
