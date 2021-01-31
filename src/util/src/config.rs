use crate::error::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::{lazy::SyncLazy, time::Duration};
use tracing::Level;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl From<LogLevel> for Level {
    fn from(l: LogLevel) -> Self {
        match l {
            LogLevel::Trace => Level::TRACE,
            LogLevel::Debug => Level::DEBUG,
            LogLevel::Info => Level::INFO,
            LogLevel::Warn => Level::WARN,
            LogLevel::Error => Level::ERROR,
        }
    }
}

impl From<Level> for LogLevel {
    fn from(l: Level) -> Self {
        match l {
            Level::TRACE => LogLevel::Trace,
            Level::DEBUG => LogLevel::Debug,
            Level::INFO => LogLevel::Info,
            Level::WARN => LogLevel::Warn,
            Level::ERROR => LogLevel::Error,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Log {
    pub level: LogLevel,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Service {
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Idle {
    pub timeout: Duration,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct General {
    pub idle: Idle,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub general: General,
    pub service: Service,
    pub log: Log,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: General {
                idle: Idle {
                    timeout: Duration::from_secs(5),
                },
            },
            service: Service { port: 10691 },
            log: Log {
                level: LogLevel::Trace,
            },
        }
    }
}

// assume that INSTANCE is accessed infrequently and in non-performance critical code
static INSTANCE: SyncLazy<Config> = SyncLazy::new(|| Config::load().unwrap_or_exit());

impl Config {
    pub fn instance() -> &'static Config {
        &*INSTANCE
    }

    pub fn data_dir() -> Result<PathBuf> {
        let data_dir =
            dirs::data_local_dir().with_context(|| "Unable to get LocalAppData directory")?;
        let config_dir = data_dir.join("Cobalt");
        if !config_dir.is_dir() {
            fs::create_dir(config_dir.clone()).with_context(|| "Create config directory")?;
        }
        Ok(config_dir)
    }

    pub fn get_file() -> Result<PathBuf> {
        let config_dir = Config::data_dir().with_context(|| "Get Cobalt data directory")?;
        let config_file = config_dir.join("config.toml");
        if !config_file.is_file() {
            let config = toml::to_string(&Config::default())
                .with_context(|| "Serialize default Config to TOML")?;
            fs::write(config_file.clone(), config)
                .with_context(|| "Write Config to config file")?;
        }
        Ok(config_file)
    }

    pub fn load() -> Result<Self> {
        let config_file = Config::get_file().with_context(|| "Get config file path")?;

        let mut config = config::Config::new();
        config
            .merge(config::File::from(config_file))
            .with_context(|| "Merge settings from config file")?;

        config
            .try_into()
            .with_context(|| "Convert config file format into Config")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn windows_paths() {
        let local = dirs::data_local_dir()
            .with_context(|| "Unable to get LocalAppData directory")
            .unwrap()
            .join("Cobalt")
            .join("config.toml");
        assert_ne!(local, PathBuf::default());
    }
}
