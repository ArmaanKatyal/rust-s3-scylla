use scylla::SerializeRow;
use serde::Deserialize;
use serde::Serialize;

pub type Logs = Vec<Log>;
pub type LogEntries = Vec<LogEntry>;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Log {
    pub log_id: Option<i64>,
    pub timestamp: Option<String>,
    pub user_id: Option<i64>,
    pub event_type: Option<String>,
    pub page_url: Option<String>,
    pub ip_address: Option<String>,
    pub device_type: Option<String>,
    pub browser: Option<String>,
    pub os: Option<String>,
    pub response_time: Option<f64>,
}

#[derive(Default, Debug, Clone, SerializeRow)]
pub struct LogEntry {
    pub id: String,
    pub ingestion_id: String,
    pub timestamp: String,
    pub user_id: i64,
    pub event_type: String,
    pub page_url: String,
    pub ip_address: String,
    pub device_type: String,
    pub browser: String,
    pub os: String,
    pub response_time: f64,
}
