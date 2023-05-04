pub use color_eyre::eyre::*;

pub fn setup() -> color_eyre::Result<()> {
    color_eyre::install()?;
    Ok(())
}