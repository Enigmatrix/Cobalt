//! List out browser information

use clap::Parser;
use util::error::Result;
use util::{config, Target};

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Browser name
    #[arg(long)]
    app: Option<String>,

    /// Browser window title
    #[arg(long)]
    title: Option<String>,
}

fn main() -> Result<()> {
    util::set_target(Target::Engine);
    let config = config::get_config()?;
    util::setup(&config)?;
    platform::setup()?;

    let args = Args::parse();
    dbg!(&args);

    Ok(())
}
