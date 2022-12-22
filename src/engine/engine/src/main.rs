use utils::errors::eyre::Context;
use utils::errors::Result;
use utils::tracing::info;

fn main() -> Result<()> {
    utils::setup().context("setup utils")?;
    info!("🚀 engine started");
    info!("🛑 engine exiting");
    Ok(())
}
