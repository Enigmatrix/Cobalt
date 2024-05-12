use crate::error::Result;
use figment::{
    providers::{Format, Json},
    Figment,
};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Config {
    connection_strings: ConnectionStrings,
    engine_log_filter: String,
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
