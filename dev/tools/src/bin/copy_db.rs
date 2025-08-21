//! Copy database from source to target

use std::path::Path;

use clap::Parser;
use dialoguer::Confirm;
use tools::db::Source;
use util::error::{Context, Result, bail};
use util::tracing::warn;
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

    remove_icon_files(&target_path)?;
    remove_db_files(&target_path)?;

    check_and_copy_dir(&source_path, &target_path, "icons")?;
    check_and_copy_file(&source_path, &target_path, "main.db")?;
    check_and_copy_file(&source_path, &target_path, "main.db-journal")?;
    check_and_copy_file(&source_path, &target_path, "main.db-shm")?;
    check_and_copy_file(&source_path, &target_path, "main.db-wal")?;

    Ok(())
}

fn check_and_remove_file(source_dir: &Path, file: &str) -> util::error::Result<()> {
    let file = source_dir.join(file);
    if std::fs::metadata(&file)
        .map(|f| f.is_file())
        .unwrap_or(false)
    {
        std::fs::remove_file(&file).context(format!("remove {:?}", &file))?;
    } else {
        warn!("file {:?} not found", file);
    }
    Ok(())
}

fn check_and_remove_dir(source_dir: &Path, dir: &str) -> util::error::Result<()> {
    let dir = source_dir.join(dir);
    if std::fs::metadata(&dir).map(|f| f.is_dir()).unwrap_or(false) {
        std::fs::remove_dir_all(&dir).context(format!("remove {:?}", &dir))?;
    } else {
        warn!("dir {:?} not found", dir);
    }
    Ok(())
}

fn check_and_copy_file(
    source_dir: &Path,
    target_dir: &Path,
    file: &str,
) -> util::error::Result<()> {
    let from_file = source_dir.join(file);
    let to_file = target_dir.join(file);
    if std::fs::metadata(&from_file)
        .map(|f| f.is_file())
        .unwrap_or(false)
    {
        std::fs::copy(from_file, to_file).context(format!("copy {file}"))?;
    } else {
        warn!("file {file} not found");
    }
    Ok(())
}

fn check_and_copy_dir(source_dir: &Path, target_dir: &Path, file: &str) -> util::error::Result<()> {
    let from_dir = source_dir.join(file);
    let to_dir = target_dir.join(file);
    if std::fs::metadata(&from_dir)
        .map(|f| f.is_dir())
        .unwrap_or(false)
    {
        // Create destination directory if it doesn't exist
        if !to_dir.exists() {
            std::fs::create_dir_all(&to_dir).context(format!("create dir {}", to_dir.display()))?;
        }

        // Copy all files from source directory to destination directory
        for entry in
            std::fs::read_dir(&from_dir).context(format!("read dir {}", from_dir.display()))?
        {
            let entry = entry.context("read dir entry")?;
            let entry_path = entry.path();
            let file_name = entry_path.file_name().unwrap();
            let dest_path = to_dir.join(file_name);

            if entry_path.is_file() {
                std::fs::copy(&entry_path, &dest_path).context(format!(
                    "copy file {} to {}",
                    entry_path.display(),
                    dest_path.display()
                ))?;
            } else if entry_path.is_dir() {
                // Recursively copy subdirectories
                check_and_copy_dir(&from_dir, &to_dir, file_name.to_str().unwrap())?;
            }
        }
    } else {
        warn!("dir {} not found", file);
    }
    Ok(())
}

fn remove_db_files(source_dir: &Path) -> util::error::Result<()> {
    // remove previous files (especially the non-main.db files)
    check_and_remove_file(source_dir, "main.db")?;
    check_and_remove_file(source_dir, "main.db-journal")?;
    check_and_remove_file(source_dir, "main.db-shm")?;
    check_and_remove_file(source_dir, "main.db-wal")?;
    Ok(())
}

fn remove_icon_files(source_dir: &Path) -> util::error::Result<()> {
    check_and_remove_dir(source_dir, "icons")?;
    Ok(())
}
