use std::mem::swap;

use rand::rngs::ThreadRng;
use rand::seq::IndexedMutRandom;
use util::error::{Context, Result};
use util::tracing::ResultTraceExt;
use windows::core::AgileReference;
use windows::ApplicationModel::{AppDisplayInfo, AppInfo as UWPAppInfo};
use windows::Foundation::Size;
use windows::Storage::FileProperties::ThumbnailMode;
use windows::Storage::StorageFile;
use windows::Storage::Streams::DataReader;

use crate::objects::FileVersionInfo;

/// Information about an App
pub struct AppInfo {
    /// Name
    pub name: String,
    /// Description
    pub description: String,
    /// Company
    pub company: String,
    /// Color
    pub color: String,
    /// Logo as bytes
    pub logo: Option<Vec<u8>>,
}

/// Image size for Win32 apps
pub const WIN32_IMAGE_SIZE: u32 = 64;
/// Image size for UWP apps
pub const UWP_IMAGE_SIZE: f32 = 256.0;

impl AppInfo {
    /// Create a new [AppInfo] of a Win32 program from its path
    pub async fn from_win32(path: &str) -> Result<Self> {
        let file = AgileReference::new(&StorageFile::GetFileFromPathAsync(&path.into())?.await?)?;

        let mut fv = FileVersionInfo::new(path)?;

        let logo = Self::win32_logo(&file)
            .await
            .map(Some)
            .with_context(|| format!("get win32 logo for {path:?}"))
            .warn();

        // yes, this is swapper, this is surprisingly more accurate.
        let mut name = fv.query_value("FileDescription").warn();
        let mut description = fv.query_value("ProductName").warn();
        // exceptions
        if description.ends_with(".exe") {
            swap(&mut name, &mut description);
        }

        Ok(AppInfo {
            // not sure why FileDescription is the actual name of the app...
            name,
            description,
            company: fv.query_value("CompanyName").warn(),
            color: random_color(),
            logo,
        })
    }

    /// Create a new [AppInfo] of a given UWP app from its AUMID
    pub async fn from_uwp(aumid: &str) -> Result<Self> {
        let app_info = UWPAppInfo::GetFromAppUserModelId(&aumid.into())?;
        let display_info = app_info.DisplayInfo()?;
        let package = app_info.Package()?;
        let logo = Self::uwp_logo(&display_info)
            .await
            .map(Some)
            .with_context(|| format!("get uwp logo for {aumid:?}"))
            .warn();

        Ok(AppInfo {
            name: display_info.DisplayName()?.to_string_lossy(),
            description: display_info.Description()?.to_string_lossy(),
            company: package.PublisherDisplayName()?.to_string_lossy(),
            color: random_color(),
            logo,
        })
    }

    async fn win32_logo(file: &AgileReference<StorageFile>) -> Result<Vec<u8>> {
        let logo = file
            .resolve()?
            .GetThumbnailAsyncOverloadDefaultOptions(ThumbnailMode::SingleItem, WIN32_IMAGE_SIZE)?;
        let (size, reader) = {
            let logo = logo.await?;
            let size = logo.Size()? as usize;
            let reader = DataReader::CreateDataReader(&logo)?;
            (size, reader)
        };
        reader.LoadAsync(size as u32)?.await?;
        let mut logo = vec![0u8; size];
        reader.ReadBytes(&mut logo)?;
        Ok(logo)
    }

    async fn uwp_logo(display_info: &AppDisplayInfo) -> Result<Vec<u8>> {
        let (size, reader) = {
            let logo = display_info
                .GetLogo(Size {
                    Width: UWP_IMAGE_SIZE,
                    Height: UWP_IMAGE_SIZE,
                })?
                .OpenReadAsync()?
                .await?;
            let size = logo.Size()? as usize;
            let reader = DataReader::CreateDataReader(&logo)?;
            (size, reader)
        };

        reader.LoadAsync(size as u32)?.await?;
        let mut logo = vec![0u8; size];
        reader.ReadBytes(&mut logo)?;
        Ok(logo)
    }
}

/// Generate a random color
pub fn random_color() -> String {
    // ref: https://github.com/catppuccin/catppuccin
    // Mocha colors
    let mut rng = rand::thread_rng();

    [
        "#f5e0dc", "#f2cdcd", "#f5c2e7", "#cba6f7", "#f38ba8", "#eba0ac", "#fab387", "#f9e2af",
        "#a6e3a1", "#94e2d5", "#89dceb", "#74c7ec", "#89b4fa", "#b4befe", "#cdd6f4", // Text
    ]
    .choose_mut(&mut rng)
    .unwrap()
    .to_string()
}

#[cfg(test)]
mod tests {

    use util::future as tokio;

    use super::*;

    #[tokio::test]
    async fn app_info_from_win32_notepad() -> Result<()> {
        let notepad = r#"C:\Windows\System32\notepad.exe"#;
        let app_info = AppInfo::from_win32(notepad).await?;
        assert_eq!("Notepad", app_info.name);
        assert_eq!("Microsoft® Windows® Operating System", app_info.description);
        assert_eq!("Microsoft Corporation", app_info.company);
        assert_ne!(0, app_info.logo.unwrap().len());
        Ok(())
    }

    #[tokio::test]
    async fn app_info_from_uwp_store() -> Result<()> {
        let aumid = "Microsoft.Windows.NarratorQuickStart_8wekyb3d8bbwe!App";
        let app_info = AppInfo::from_uwp(aumid).await?;
        assert_eq!("Narrator", app_info.name);
        assert_eq!("Narrator Home", app_info.description);
        assert_eq!("Microsoft", app_info.company);
        assert_ne!(0, app_info.logo.unwrap().len());
        Ok(())
    }
}
