use crate::error::Result;
use figment::{
    providers::{Format, Json},
    Figment,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Config {
    connection_strings: ConnectionStrings,
}

impl Config {
    pub fn connection_string(&self) -> &str {
        &self
            .connection_strings
            .query_context
            .split_once("=")
            .expect("format of the connection string is Data Source=<path>")
            .1
    }
}

#[derive(Debug, Deserialize)]
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
    assert_eq!(
        "Data Source=main.db",
        config.connection_strings.query_context
    );
    Ok(())
}
