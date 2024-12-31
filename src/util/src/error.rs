pub use color_eyre::eyre::*;

/// Setup error handling
pub fn setup() -> color_eyre::Result<()> {
    color_eyre::install()?;
    Ok(())
}
