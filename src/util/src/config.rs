use std::time::Duration;

use tracing::Level;

pub struct Config {
    pub idle_timeout: Duration,
    pub db_connection: String,
    pub log_level: Level,
}

impl Config {
    pub fn instance() -> Self {
        Self {
            idle_timeout: Duration::from_secs(5),
            db_connection: "C:\\Users\\enigm\\NANI.db".to_string(),
            log_level: Level::INFO
        }
    }
}