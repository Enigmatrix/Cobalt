use common::settings::Settings;
use common::errors::*;

fn main() -> Result<()> {
    let settings = Settings::from_file("appsettings.json").context("fetch settings")?;
    common::setup(&settings).context("setup common")?;

    Ok(())
}
