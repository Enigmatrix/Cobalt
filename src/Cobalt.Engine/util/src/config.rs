use std::time::Duration;

use figment::providers::{Format, Json};
use figment::Figment;
use serde::Deserialize;

use crate::error::Result;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Config {
    connection_strings: ConnectionStrings,
    engine_log_filter: String,
    max_idle_duration: Duration,
    poll_duration: Duration,
    alert_duration: Duration,
}

impl Config {
    pub fn connection_string(&self) -> &str {
        self.connection_strings
            .query_context
            .split_once("=")
            .expect("format of the connection string is Data Source=<path>")
            .1
    }

    pub fn engine_log_filter(&self) -> &str {
        &self.engine_log_filter
    }

    pub fn max_idle_duration(&self) -> Duration {
        self.max_idle_duration
    }

    pub fn poll_duration(&self) -> Duration {
        self.poll_duration
    }

    pub fn alert_duration(&self) -> Duration {
        self.alert_duration
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ConnectionStrings {
    query_context: String,
}

pub fn get_config() -> Result<Config> {
    Ok(Figment::new()
        .merge(Json::file("appsettings.json"))
        .extract()?)
}

#[test]
fn extract_query_content_connection_string() -> Result<()> {
    let config = get_config()?;
    assert_eq!("main.db", config.connection_string());
    Ok(())
}

#[test]
fn extract_engine_log_filter() -> Result<()> {
    let config = get_config()?;
    assert_eq!("Debug", config.engine_log_filter());
    Ok(())
}
