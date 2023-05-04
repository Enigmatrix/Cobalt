pub mod errors;
pub mod tracing;
pub mod settings;

use settings::Settings;
use errors::*;

pub fn setup(settings: &Settings) -> Result<()> {
    errors::setup().context("setup errors")?;
    tracing::setup(&settings.logging.engine_log_filter.filter).context("setup tracing")?;
    Ok(())
}