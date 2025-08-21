//! Copy database from source to target

use clap::Parser;
use dialoguer::Confirm;
use tools::db::Source;
use util::error::{Context, Result, bail};
use util::{Target, config, future as tokio};

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Database source ('seed', 'install', or custom path)
    #[arg(short, long)]
    source: Source,
    /// Database target ('seed', 'install', or custom path)
    #[arg(short, long)]
    target: Source,
}

#[tokio::main]
async fn main() -> Result<()> {
    util::set_target(Target::Tool {
        name: "copy_db".to_string(),
    });
    let config = config::get_config()?;
    util::setup(&config)?;
    platform::setup()?;

    let args = Args::parse();
    let source_path = args
        .source
        .dir_path()
        .context("failed to resolve source path")?;
    let target_path = args
        .target
        .dir_path()
        .context("failed to resolve target path")?;

    if source_path.canonicalize()? == target_path.canonicalize()? {
        bail!("source and target are the same");
    }

    let confirmed = Confirm::new()
        .with_prompt(format!(
            "Are you sure you want to copy from {:?} to {:?}?",
            args.source, args.target
        ))
        .interact()
        .context("failed to confirm")?;
    if !confirmed {
        return Ok(());
    }

    util::fs::remove_icon_files(&target_path)?;
    util::fs::remove_db_files(&target_path)?;
    util::fs::copy_icon_files(&source_path, &target_path)?;
    util::fs::copy_db_files(&source_path, &target_path)?;

    Ok(())
}
