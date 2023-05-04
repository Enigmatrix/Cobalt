use crate::errors::*;
use ::config::{File, Config};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct EngineLogFilter {
    #[serde(rename(deserialize = "Filter"))]
    pub filter: String
}

#[derive(Debug, Deserialize)]
pub struct Logging {
    #[serde(rename(deserialize = "EngineLogFilter"))]
    pub engine_log_filter: EngineLogFilter
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    #[serde(rename(deserialize = "Logging"))]
    pub logging: Logging
}

impl Settings {
    pub fn from_file(name: &str) -> Result<Self> {
        Ok(Config::builder()
            .add_source(File::with_name(name))
            .build().context("create config")?
            .try_deserialize().context("deserialize settings")?)
    }
}