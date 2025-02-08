use std::time::Duration;

use figment::providers::{Format, Json};
use figment::Figment;
use serde::Deserialize;

use crate::error::{anyhow, Result};

/// [Config] of the Engine
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Config {
    connection_strings: ConnectionStrings,
    engine_log_filter: String,
    ui_log_filter: String,
    max_idle_duration: Duration,
    poll_duration: Duration,
    alert_duration: Duration,
}

impl Config {
    /// Returns the connection string to the query context database
    pub fn connection_string(&self) -> Result<&str> {
        Ok(self
            .connection_strings
            .query_context
            .split_once("=")
            .ok_or_else(|| anyhow!("format of the connection string is Data Source=<path>"))?
            .1)
    }

    /// Engine Log filter (tracing)
    pub fn engine_log_filter(&self) -> &str {
        &self.engine_log_filter
    }

    /// UI Log filter (tracing)
    pub fn ui_log_filter(&self) -> &str {
        &self.ui_log_filter
    }

    /// Maximum idle duration before the interaction period ends
    pub fn max_idle_duration(&self) -> Duration {
        self.max_idle_duration
    }

    /// How often the engine should poll for window switches, interactions, etc.
    pub fn poll_duration(&self) -> Duration {
        self.poll_duration
    }

    /// How often the engine should check for alerts firing
    pub fn alert_duration(&self) -> Duration {
        self.alert_duration
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ConnectionStrings {
    query_context: String,
}

/// Get the configuration from the appsettings.json file
pub fn get_config() -> Result<Config> {
    Ok(Figment::new()
        .merge(Json::file("appsettings.json"))
        .extract()?)
}

#[test]
fn extract_query_content_connection_string() -> Result<()> {
    let config = get_config()?;
    assert_eq!("main.db", config.connection_string()?);
    Ok(())
}

#[test]
fn extract_engine_log_filter() -> Result<()> {
    let config = get_config()?;
    assert_eq!("Debug", config.engine_log_filter());
    Ok(())
}
