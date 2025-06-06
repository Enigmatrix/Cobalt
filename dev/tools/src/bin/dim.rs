//! Dim a window to a given opacity

use clap::Parser;
use util::error::Result;
use util::tracing::info;
use util::{config, Target};

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Window opacity (0.0 to 1.0)
    #[arg(short, long, value_parser = |s: &str| -> Result<f64, String> {
        let val = s.parse::<f64>().map_err(|_| "Invalid number".to_string())?;
        if val >= 0.0 && val <= 1.0 {
            Ok(val)
        } else {
            Err("Opacity must be between 0.0 and 1.0".to_string())
        }
    })]
    opacity: f64,

    /// Window handle in hexadecimal
    #[arg(long, group = "window_identifier", required = true)]
    handle: Option<String>,

    /// Application name to filter by
    #[arg(long, group = "window_identifier", required = true)]
    app: Option<String>,

    /// Window title text to filter by
    #[arg(group = "window_identifier", required = true)]
    title: Option<String>,
}

/// Filter for a window
#[derive(Debug, Clone)]
pub struct WindowFilter {
    /// Window handle in hexadecimal
    pub handle: Option<String>,

    /// Application name to filter by
    pub app: Option<String>,

    /// Window title text to filter by
    pub title: Option<String>,
}

fn main() -> Result<()> {
    util::set_target(Target::Engine);
    let config = config::get_config()?;
    util::setup(&config)?;
    info!("starting tool dim");
    platform::setup()?;

    let args = Args::parse();
    dbg!(&args);

    // Validate opacity
    Ok(())
}
