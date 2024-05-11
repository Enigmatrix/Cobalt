use util::error::Result;

mod engine;
mod resolver;
mod sentry;

fn main() -> Result<()> {
    let config = util::config::get_config()?;
    util::setup(&config)?;

    println!("Hello, world!");
    Ok(())
}
