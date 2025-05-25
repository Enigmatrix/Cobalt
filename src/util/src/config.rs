use std::time::Duration;

pub use dirs::*;
use figment::providers::{Format, Json};
use figment::Figment;
use serde::Deserialize;

use crate::error::Result;

/// [Config] of the Engine
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Config {
    engine_log_filter: String,
    ui_log_filter: String,
    max_idle_duration: Duration,
    poll_duration: Duration,
    alert_duration: Duration,
}

impl Config {
    /// Get the config path from dir
    pub fn config_path(segment: &str) -> Result<String> {
        #[cfg(debug_assertions)]
        {
            use crate::{Target, TARGET};

            let target = TARGET.lock().unwrap().clone();
            match target {
                Target::Engine => Ok(segment.to_string()),
                Target::Ui => Ok("../../../".to_string() + segment),
            }
        }

        // The dirs crate is what tauri uses to get the data directory.
        // me.cobalt.enigmatrix is the bundle identifier for the app.
        #[cfg(not(debug_assertions))]
        {
            use crate::error::{eyre, ContextCompat};
            dirs::data_local_dir()
                .context("data local dir")?
                .join("me.enigmatrix.cobalt")
                .join(segment)
                .into_os_string()
                .into_string()
                .map_err(|oss| eyre!("convert path to utf8: {:?}", oss))
        }
    }

    /// Returns the connection string to the query context database
    pub fn connection_string(&self) -> Result<String> {
        Self::config_path("main.db")
    }

    /// Returns the connection string to the query context database
    pub fn logs_dir(&self) -> Result<String> {
        Self::config_path("logs")
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

/// Get the configuration from the appsettings.json file
pub fn get_config() -> Result<Config> {
    let mut figment = Figment::new();
    figment = figment.merge(Json::file("appsettings.json"));
    #[cfg(debug_assertions)]
    {
        figment = figment.merge(Json::file("dev/appsettings.Debug.json"));
    }
    Ok(figment.extract()?)
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
    assert_eq!(
        "Debug,selectors=info,html5ever=info",
        config.engine_log_filter()
    );
    Ok(())
}
