pub use color_eyre::*;

pub fn setup() -> color_eyre::Result<()> {
    color_eyre::install()?;
    Ok(())
}
