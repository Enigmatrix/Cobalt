use std::fs;
use std::path::Path;

use crate::error::{Context, Result};
use crate::tracing::warn;

/// Check if file exists then remove it
pub fn check_and_remove_file(source_dir: &Path, file: &str) -> Result<()> {
    let file = source_dir.join(file);
    if fs::metadata(&file).map(|f| f.is_file()).unwrap_or(false) {
        fs::remove_file(&file).context(format!("remove {:?}", &file))?;
    } else {
        warn!("file {:?} not found", file);
    }
    Ok(())
}

/// Check if dir exists then remove it
pub fn check_and_remove_dir(source_dir: &Path, dir: &str) -> Result<()> {
    let dir = source_dir.join(dir);
    if fs::metadata(&dir).map(|f| f.is_dir()).unwrap_or(false) {
        fs::remove_dir_all(&dir).context(format!("remove {dir:?}"))?;
    } else {
        warn!("dir {:?} not found", dir);
    }
    Ok(())
}

/// Check if source file exists then copy it
pub fn check_and_copy_file(source_dir: &Path, target_dir: &Path, file: &str) -> Result<()> {
    let from_file = source_dir.join(file);
    let to_file = target_dir.join(file);
    if fs::metadata(&from_file)
        .map(|f| f.is_file())
        .unwrap_or(false)
    {
        fs::copy(from_file, to_file).context(format!("copy {file}"))?;
    } else {
        warn!("file {file} not found");
    }
    Ok(())
}

/// Check if source dir exists then copy it
pub fn check_and_copy_dir(
    source_dir: &Path,
    target_dir: &Path,
    file: impl AsRef<Path>,
) -> Result<()> {
    let file = file.as_ref().file_name().expect("file name is ..");
    let from_dir = source_dir.join(file);
    let to_dir = target_dir.join(file);
    if fs::metadata(&from_dir).map(|f| f.is_dir()).unwrap_or(false) {
        // Create destination directory if it doesn't exist
        if !to_dir.exists() {
            fs::create_dir_all(&to_dir).context(format!("create dir {}", to_dir.display()))?;
        }

        // Copy all files from source directory to destination directory
        for entry in fs::read_dir(&from_dir).context(format!("read dir {}", from_dir.display()))? {
            let entry = entry.context("read dir entry")?;
            let entry_path = entry.path();
            let dest_path = to_dir.join(entry_path.file_name().expect("file name is .."));

            if entry_path.is_file() {
                fs::copy(&entry_path, &dest_path).context(format!(
                    "copy file {} to {}",
                    entry_path.display(),
                    dest_path.display()
                ))?;
            } else if entry_path.is_dir() {
                warn!("dir {} is not a file", entry_path.display());
            }
        }
    } else {
        warn!("dir {} not found", file.display());
    }
    Ok(())
}

/// Remove db files
pub fn remove_db_files(source_dir: &Path) -> Result<()> {
    // remove previous files (especially the non-main.db files)
    check_and_remove_file(source_dir, "main.db")?;
    check_and_remove_file(source_dir, "main.db-journal")?;
    check_and_remove_file(source_dir, "main.db-shm")?;
    check_and_remove_file(source_dir, "main.db-wal")?;
    Ok(())
}

/// Copy db files
pub fn copy_db_files(source_dir: &Path, target_dir: &Path) -> Result<()> {
    check_and_copy_file(source_dir, target_dir, "main.db")?;
    check_and_copy_file(source_dir, target_dir, "main.db-journal")?;
    check_and_copy_file(source_dir, target_dir, "main.db-shm")?;
    check_and_copy_file(source_dir, target_dir, "main.db-wal")?;
    Ok(())
}
