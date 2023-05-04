pub mod errors;
pub mod settings;
pub mod tracing;

use errors::*;
use settings::Settings;

pub fn setup(settings: &Settings) -> Result<()> {
    errors::setup().context("setup errors")?;
    tracing::setup(&settings.logging.engine_log_filter.filter).context("setup tracing")?;
    Ok(())
}
