use std::path::{Path, PathBuf};

use semver::Version;
use util::config::data_local_dir;
use util::error::{ContextCompat, Result, bail};

/// Squirrel executable e.g. %LOCALAPPDATA%\{identifier}\app-{version}\{file}
/// ref: https://github.com/Squirrel/Squirrel.Windows/blob/51f5e2cb01add79280a53d51e8d0cfa20f8c9f9f/docs/getting-started/4-installing.md
/// for guarentees on format, including ref: https://github.com/Squirrel/Squirrel.Windows/issues/1002 for guarentees on %LOCALAPPDATA%.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SquirrelExe {
    /// Base directory e.g. %LOCALAPPDATA%\{identifier}
    pub base_dir: SquirrelBaseDir,
    /// Semver version
    pub version: Version,
    /// Executable file name (including .exe)
    pub file: String,
}

impl SquirrelExe {
    /// Check if this path is potentially a Squirrel executable. Assumes that the path is a valid path and exists.
    pub fn new<P: AsRef<Path>>(path: P) -> Option<SquirrelExe> {
        let path = path.as_ref();

        let local_app_data = data_local_dir()?;

        // Check if path starts with %LOCALAPPDATA% (resolve this env var)
        // Get the relative path from LOCALAPPDATA which also checks if the path starts with it.
        let relative_path = path.strip_prefix(&local_app_data).ok()?;

        let mut components = relative_path.components();

        // Extract identifier (first segment)
        let identifier = components.next()?.as_os_str().to_string_lossy().to_string();

        // Extract version (second segment, must start with "app-")
        let version_segment = components.next()?.as_os_str().to_string_lossy();
        let version_string = version_segment.strip_prefix("app-")?;
        let version = Version::parse(version_string).ok()?;

        // Extract file (third segment, must be last and end with .exe)
        let file_segment = components.next()?.as_os_str().to_string_lossy();
        if components.next().is_some() || !file_segment.ends_with(".exe") {
            return None;
        }
        let file = file_segment.to_string();

        // Check if ../Update.exe relative to the original path exists
        // ref: https://github.com/Squirrel/Squirrel.Windows/blob/51f5e2cb01add79280a53d51e8d0cfa20f8c9f9f/docs/faq.md
        // and the linked issue mention this is the way to check if the app is a squirrel app.
        let update_exe_path = path.parent()?.parent()?.join("Update.exe");
        if !update_exe_path.exists() {
            return None;
        }

        Some(SquirrelExe {
            base_dir: SquirrelBaseDir { identifier },
            version,
            file,
        })
    }

    /// Get as path
    pub fn path(&self) -> Result<PathBuf> {
        let local_app_data = data_local_dir().context("data local dir")?;
        Ok(local_app_data
            .join(&self.base_dir.identifier)
            .join(format!("app-{}", self.version))
            .join(&self.file))
    }
}

/// Squirrel base directory e.g. %LOCALAPPDATA%\{identifier}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SquirrelBaseDir {
    /// Identifier e.g. software name / company
    pub identifier: String,
}

impl SquirrelBaseDir {
    /// Create a [SquirrelBaseDir] from a identifier. Assumes that the identifier results in a valid path.
    pub fn new(identifier: String) -> Result<Self> {
        Ok(Self { identifier })
    }

    /// Get the base directory as path e.g. %LOCALAPPDATA%\{identifier}
    pub fn directory(&self) -> Result<PathBuf> {
        let local_app_data = data_local_dir().context("data local dir")?;
        let path = local_app_data.join(&self.identifier);

        // this also checks if the dir exists
        if !path.is_dir() {
            bail!("squirrel base dir {} is not a directory", path.display());
        }
        Ok(path)
    }

    /// Get latest version of the squirrel installation present in the base directory.
    pub fn latest_version(&self) -> Result<Version> {
        // Algorithm in https://github.com/Squirrel/Squirrel.Windows/blob/51f5e2cb01add79280a53d51e8d0cfa20f8c9f9f/src/StubExecutable/StubExecutable.cpp#L48
        // Note that they skip directories that have a .not-finished file inside but we don't really care :)
        let path = self.directory()?;
        let versions = path.read_dir()?.filter_map(|entry| {
            // all Errs are filtered out and not logged!
            let path = entry.ok()?.path();

            if !path.is_dir() {
                return None;
            }

            path.file_name()?
                .to_string_lossy()
                .strip_prefix("app-")
                .and_then(|s| Version::parse(s).ok())
        });
        versions.max().context("no versions found")
    }

    /// Get latest version of the squirrel installation's executable.
    pub fn latest_exe(&self, file: &str) -> Result<SquirrelExe> {
        let version = self.latest_version()?;
        Ok(SquirrelExe {
            base_dir: self.clone(),
            version,
            file: file.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_squirrel_exe_parsing() {
        // This test would require creating actual directories and files
        // For now, we'll just test that the function signature works
        let path = PathBuf::from("C:\\Users\\test\\AppData\\Local\\TestApp\\app-1.2.3\\test.exe");
        let result = SquirrelExe::new(&path);
        // This should return None because the path doesn't exist and doesn't match the expected structure
        assert!(result.is_none());
    }

    // #[test]
    // fn test_squirrel_exe_check() {
    //     let path = PathBuf::from("C:\\Users\\enigm\\AppData\\Local\\Discord\\app-1.2.3\\test.exe");
    //     let result = SquirrelExe::new(&path);
    //     assert!(result.is_some());

    //     let path = PathBuf::from("C:\\Users\\enigm\\AppData\\Local\\Figma\\app-1.2.3\\test.exe");
    //     let result = SquirrelExe::new(&path);
    //     assert!(result.is_some());

    //     let path = PathBuf::from("C:\\Users\\enigm\\AppData\\Local\\slack\\app-1.2.3\\test.exe");
    //     let result = SquirrelExe::new(&path);
    //     assert!(result.is_some());
    // }

    // #[test]
    // fn test_squirrel_latest() -> Result<()> {
    //     let path = PathBuf::from("C:\\Users\\enigm\\AppData\\Local\\Discord\\app-1.2.3\\test.exe");
    //     let result = SquirrelExe::new(&path);
    //     assert!(result.is_some());

    //     let exe = result.context("squirrel exe")?;
    //     let latest_exe = exe.base_dir.latest_exe("Discord.exe")?;
    //     let path = latest_exe.path()?;
    //     dbg!(&path);
    //     assert!(path.exists());
    //     Ok(())
    // }

    #[test]
    fn test_invalid_paths() {
        // Test with various invalid paths
        assert!(SquirrelExe::new("not_a_squirrel_path").is_none());
        assert!(SquirrelExe::new("C:\\Windows\\System32\\notepad.exe").is_none());
        assert!(
            SquirrelExe::new("C:\\Users\\test\\AppData\\Local\\TestApp\\wrong-format\\test.exe")
                .is_none()
        );
    }
}
