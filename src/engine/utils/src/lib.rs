pub mod errors;
pub mod tracing;

use color_eyre::eyre::Context;
use errors::Result;

pub fn setup() -> Result<()> {
    errors::setup().context("setup errors")?;
    tracing::setup().context("setup tracing")?;
    Ok(())
}
