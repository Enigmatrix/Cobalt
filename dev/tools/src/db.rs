use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;

use util::error::{ContextCompat, Result, bail};

#[derive(Debug, Clone, Eq, PartialEq)]
/// Source of the database
pub enum Source {
    // /// Seed database
    // Seed,
    /// Install location
    Install,
    /// Current directory
    Current,
    /// Custom path
    Custom(PathBuf),
}

impl FromStr for Source {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            // "seed" => Ok(Source::Seed),
            "install" => Ok(Source::Install),
            "." => Ok(Source::Current),
            _ => Ok(Source::Custom(PathBuf::from(s))),
        }
    }
}

impl Source {
    /// Convert the source to a PathBuf
    pub fn dir_path(&self) -> Result<PathBuf> {
        let path = match self {
            // Source::Seed => Ok(PathBuf::from("./dev/seed.db")),
            Source::Install => util::config::data_local_dir()
                .context("data local dir")?
                .join("me.enigmatrix.cobalt"),
            Source::Current => PathBuf::from("."),
            Source::Custom(path) => PathBuf::from(path),
        };
        if !path.is_dir() {
            bail!("{} is not a directory", path.display());
        }
        Ok(path)
    }
}

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Source::Install => write!(f, "install"),
            Source::Current => write!(f, "."),
            Source::Custom(path) => write!(f, "{}", path.display()),
        }
    }
}
