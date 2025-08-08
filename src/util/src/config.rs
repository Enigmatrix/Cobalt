use std::io::Result;
use std::path::PathBuf;
use std::time::Duration;

pub use dirs::*;
use serde::{Deserialize, Serialize};

/// [Config] of the Engine
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    engine_log_filter: String,
    ui_log_filter: String,

    track_incognito: Option<bool>,
    default_focus_streak_settings: FocusStreakSettings,
    default_distractive_streak_settings: DistractiveStreakSettings,

    max_idle_duration: Duration,
    poll_duration: Duration,
    alert_duration: Duration,
}

/// Settings for focus streaks.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FocusStreakSettings {
    /// The minimum score of a focused app.
    pub min_focus_score: f64,
    /// The minimum duration of a focused usage.
    pub min_focus_usage_dur: Duration,
    /// The maximum gap between two focused streaks.
    pub max_focus_gap: Duration,
}

impl Default for FocusStreakSettings {
    fn default() -> Self {
        Self {
            min_focus_score: 0.0,
            // harder to be focused
            min_focus_usage_dur: Duration::from_secs(60),
            max_focus_gap: Duration::from_secs(10),
        }
    }
}

/// Settings for distractive streaks.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DistractiveStreakSettings {
    /// The maximum score of a distractive app.
    pub max_distractive_score: f64,
    /// The minimum duration of a distractive usage.
    pub min_distractive_usage_dur: Duration,
    /// The maximum gap between two distractive streaks.
    pub max_distractive_gap: Duration,
}

impl Default for DistractiveStreakSettings {
    fn default() -> Self {
        Self {
            max_distractive_score: 0.0,
            // easier to be distracted
            min_distractive_usage_dur: Duration::from_secs(10),
            max_distractive_gap: Duration::from_secs(60),
        }
    }
}

/// The name of the config file during tests
#[cfg(test)]
pub const CONFIG_FILE: &str = "test.appsettings.json";

/// The name of the config file
#[cfg(not(test))]
pub const CONFIG_FILE: &str = "appsettings.json";

impl Default for Config {
    fn default() -> Self {
        Self {
            engine_log_filter: "Info".to_string(),
            ui_log_filter: "Info".to_string(),
            track_incognito: Some(false),
            default_focus_streak_settings: FocusStreakSettings::default(),
            default_distractive_streak_settings: DistractiveStreakSettings::default(),
            max_idle_duration: Duration::from_secs(5),
            poll_duration: Duration::from_secs(1),
            alert_duration: Duration::from_secs(1),
        }
    }
}

impl Config {
    /// Get the config path from dir
    pub fn config_path(segment: &str) -> Result<PathBuf> {
        #[cfg(debug_assertions)]
        {
            use crate::{TARGET, Target};

            let target = TARGET.lock().unwrap().clone();
            match target {
                Target::Engine => Ok(PathBuf::from(segment)),
                Target::Ui => Ok(PathBuf::from("../../../").join(segment)),
                Target::Tool { .. } => Ok(PathBuf::from(segment)),
            }
        }

        // The dirs crate is what tauri uses to get the data directory.
        // me.cobalt.enigmatrix is the bundle identifier for the app.
        #[cfg(not(debug_assertions))]
        {
            use std::io::{Error, ErrorKind};
            let parent = dirs::data_local_dir()
                .ok_or(Error::new(ErrorKind::NotFound, "data local dir"))?
                .join("me.enigmatrix.cobalt");

            if !parent.try_exists()? {
                std::fs::create_dir(&parent)?;
            }

            Ok(parent.join(segment))
        }
    }

    /// Validate the config and replace any missing values with defaults
    pub fn validate_or_replace(&mut self) -> Result<()> {
        // don't need to validate log filters

        // validate durations
        let mut replace = false;
        if self.poll_duration < Duration::from_secs(1) {
            replace = true;
        }
        if self.poll_duration >= Duration::from_secs(10) {
            replace = true;
        }

        if self.max_idle_duration < Duration::from_secs(1) {
            replace = true;
        }
        if self.max_idle_duration >= Duration::from_secs(10) {
            replace = true;
        }

        if self.alert_duration < Duration::from_secs(1) {
            replace = true;
        }
        if self.alert_duration >= Duration::from_secs(10) {
            replace = true;
        }

        if self.default_focus_streak_settings.min_focus_score
            < self
                .default_distractive_streak_settings
                .max_distractive_score
        {
            replace = true;
        }
        if self.default_focus_streak_settings.min_focus_score < -100.0
            || self.default_focus_streak_settings.min_focus_score > 100.0
        {
            replace = true;
        }
        if self
            .default_distractive_streak_settings
            .max_distractive_score
            < -100.0
            || self
                .default_distractive_streak_settings
                .max_distractive_score
                > 100.0
        {
            replace = true;
        }

        // Streak durations have no real limits since it doesn't affect the engine.

        if replace {
            Self::replace_with_default()?;
            *self = Default::default();
        }
        Ok(())
    }

    /// Replace the config with the default values
    pub fn replace_with_default() -> Result<()> {
        #[cfg(not(debug_assertions))]
        let config = Config::default();
        #[cfg(debug_assertions)]
        let config: Config = serde_json::from_str(&std::fs::read_to_string(Self::config_path(
            "dev/appsettings.Debug.json",
        )?)?)?;

        config.write()?;
        Ok(())
    }

    /// Returns the connection string to the query context database
    pub fn connection_string(&self) -> Result<PathBuf> {
        Self::config_path("main.db")
    }

    /// Returns the path to the logs directory
    pub fn logs_dir(&self) -> Result<PathBuf> {
        Self::config_path("logs")
    }

    /// Returns the path to the icons directory
    pub fn icons_dir() -> Result<PathBuf> {
        let path = Self::config_path("icons")?;
        static ICON_CREATE_GUARD: std::sync::Mutex<()> = std::sync::Mutex::new(());
        let _icon_create_guard = ICON_CREATE_GUARD
            .lock()
            .expect("failed to lock icon create guard");
        if !path.exists() {
            std::fs::create_dir(&path)?;
        }
        Ok(path)
    }

    /// Engine Log filter (tracing)
    pub fn engine_log_filter(&self) -> &str {
        &self.engine_log_filter
    }

    /// Whether to track incognito usage
    pub fn track_incognito(&self) -> bool {
        self.track_incognito.unwrap_or(false)
    }

    /// Set the track incognito setting
    pub fn set_track_incognito(&mut self, value: bool) -> Result<()> {
        self.track_incognito = Some(value);
        self.write()?;
        Ok(())
    }

    /// Set default focus streak settings
    pub fn set_default_focus_streak_settings(&mut self, value: FocusStreakSettings) -> Result<()> {
        self.default_focus_streak_settings = value;
        self.write()?;
        Ok(())
    }

    /// Set default distractive streak settings
    pub fn set_default_distractive_streak_settings(
        &mut self,
        value: DistractiveStreakSettings,
    ) -> Result<()> {
        self.default_distractive_streak_settings = value;
        self.write()?;
        Ok(())
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

    /// Default focus streak settings
    pub fn default_focus_streak_settings(&self) -> &FocusStreakSettings {
        &self.default_focus_streak_settings
    }

    /// Default distractive streak settings
    pub fn default_distractive_streak_settings(&self) -> &DistractiveStreakSettings {
        &self.default_distractive_streak_settings
    }

    /// Write the config to the appsettings.json file
    pub fn write(&self) -> Result<()> {
        let path = &Self::config_path(CONFIG_FILE)?;
        std::fs::write(path, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }
}

/// Get the configuration from the appsettings.json file
pub fn get_config() -> Result<Config> {
    let path = Config::config_path(CONFIG_FILE)?;
    let mut config: Config = loop {
        match std::fs::read_to_string(&path)
            .map_err(|e| e.to_string())
            .and_then(|s| serde_json::from_str(&s).map_err(|e| e.to_string()))
        {
            Ok(config) => break config,
            Err(e) => {
                // can't log this since logger isn't setup yet.
                eprintln!("Error loading config: {e:?}");
                Config::replace_with_default().expect("failed to replace with default config");
                // eprintln!("Replaced with default config: {:?}", e);
            }
        }
    };
    config
        .validate_or_replace()
        .expect("failed to validate or replace config");
    Ok(config)
}

#[test]
fn combined_extract_fields() -> Result<()> {
    std::env::set_current_dir("../../")?;
    let config = get_config()?;
    assert_eq!(
        "main.db",
        config.connection_string()?.to_string_lossy().to_string()
    );
    std::fs::remove_file(Config::config_path(CONFIG_FILE)?).expect("failed to remove file");
    std::env::set_current_dir("src/util")?;

    std::env::set_current_dir("../../")?;
    let config = get_config()?;
    assert_eq!(
        "Debug,selectors=info,html5ever=info",
        config.engine_log_filter()
    );
    std::fs::remove_file(Config::config_path(CONFIG_FILE)?).expect("failed to remove file");
    std::env::set_current_dir("src/util")?;
    Ok(())
}
