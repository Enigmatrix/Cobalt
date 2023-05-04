use common::errors::*;
use common::settings::Settings;
use common::tracing::*;

fn main() -> Result<()> {
    let settings = Settings::from_file("appsettings.json").context("fetch settings")?;
    common::setup(&settings).context("setup common")?;
    info!("engine is running");

    Ok(())
}
