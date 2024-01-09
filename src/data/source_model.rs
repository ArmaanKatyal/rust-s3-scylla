// SOURCE SCHEMA
use serde::Deserialize;
use serde::Serialize;

pub type Logs = Vec<Log>;

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
