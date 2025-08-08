use std::mem::swap;
use std::path::Path;

use rand::seq::IndexedMutRandom;
use util::error::{Context, Result};
use util::tracing::ResultTraceExt;
use windows::ApplicationModel::{AppDisplayInfo, AppInfo as UWPAppInfo};
use windows::Foundation::Size;
use windows::Storage::FileProperties::ThumbnailMode;
use windows::Storage::StorageFile;
use windows::Storage::Streams::DataReader;
use windows::core::AgileReference;

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
    /// Icon
    pub icon: Option<Icon>,
}

/// Icon data
pub struct Icon {
    /// Icon as bytes
    pub data: Vec<u8>,
    /// File extension
    pub ext: Option<String>,
    /// Content type (mime type)
    pub mime: Option<String>,
}

// https://developer.mozilla.org/en-US/docs/Web/Media/Guides/Formats/Image_types#common_image_file_types
const IMAGE_EXTS: &[&str] = &[
    "png", "apng", // PNG variants
    "avif", // AVIF
    "gif",  // GIF variants
    "jpg", "jpeg", "jfif", "pjpeg", "pjp",  // JPEG variants
    "svg",  // SVG
    "webp", // WebP
    "bmp",  // BMP
    "ico", "cur", // ICO variants
    "tiff", "tif", // TIFF variants
];

impl Icon {
    /// Deduce the extension of the icon from the mime type or file magic bytes
    pub fn deduce_ext(&self) -> Option<String> {
        // First try explicit extension if available
        if let Some(ext) = &self.ext {
            return Some(ext.clone());
        }

        // Then try MIME type if available
        if let Some(mime) = &self.mime
            && let Some(ext) = mime2ext::mime2ext(mime)
        {
            return Self::to_valid_image_ext(ext);
        }

        // Finally try magic bytes detection
        if let Some(kind) = infer::get(&self.data) {
            return Self::to_valid_image_ext(kind.extension());
        }

        None
    }

    fn to_valid_image_ext(ext: &str) -> Option<String> {
        if IMAGE_EXTS.contains(&ext) {
            Some(ext.to_string())
        } else {
            None
        }
    }
}

/// Image size for Win32 apps
pub const WIN32_IMAGE_SIZE: u32 = 64;
/// Image size for UWP apps
pub const UWP_IMAGE_SIZE: f32 = 256.0;

impl AppInfo {
    /// Create a default [AppInfo] from a Win32 path
    pub fn default_from_win32_path(path: &str) -> Self {
        let path = Path::new(path);
        let file = path.file_name().expect("file name").to_string_lossy();
        Self {
            name: file.to_string(),
            description: file.to_string(),
            company: "".to_string(),
            color: random_color(),
            icon: None,
        }
    }

    /// Create a default [AppInfo] from a UWP AUMID
    pub fn default_from_uwp(aumid: &str) -> Self {
        let name = aumid; // TODO: get name from company, AUMID
        Self {
            name: name.to_string(),
            description: name.to_string(),
            company: "".to_string(),
            color: random_color(),
            icon: None,
        }
    }

    /// Create a new [AppInfo] of a Win32 program from its path
    pub async fn from_win32(path: &str) -> Result<Self> {
        let file = AgileReference::new(&StorageFile::GetFileFromPathAsync(&path.into())?.await?)?;

        let mut fv = FileVersionInfo::new(path)?;

        let icon = Self::win32_icon(&file)
            .await
            .map(Some)
            .with_context(|| format!("get win32 icon for {path:?}"))
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
            icon,
        })
    }

    /// Create a new [AppInfo] of a given UWP app from its AUMID
    pub async fn from_uwp(aumid: &str) -> Result<Self> {
        let app_info = UWPAppInfo::GetFromAppUserModelId(&aumid.into())?;
        let display_info = app_info.DisplayInfo()?;
        let package = app_info.Package()?;
        let icon = Self::uwp_icon(&display_info)
            .await
            .map(Some)
            .with_context(|| format!("get uwp icon for {aumid:?}"))
            .warn();

        Ok(AppInfo {
            name: display_info.DisplayName()?.to_string_lossy(),
            description: display_info.Description()?.to_string_lossy(),
            company: package.PublisherDisplayName()?.to_string_lossy(),
            color: random_color(),
            icon,
        })
    }

    async fn win32_icon(file: &AgileReference<StorageFile>) -> Result<Icon> {
        let icon = file
            .resolve()?
            .GetThumbnailAsyncOverloadDefaultOptions(ThumbnailMode::SingleItem, WIN32_IMAGE_SIZE)?;
        let (content_type, size, reader) = {
            let icon = icon.await?;
            let content_type = icon.ContentType()?.to_string_lossy();
            let size = icon.Size()? as usize;
            let reader = DataReader::CreateDataReader(&icon)?;
            (content_type, size, reader)
        };
        reader.LoadAsync(size as u32)?.await?;
        let mut data = vec![0u8; size];
        reader.ReadBytes(&mut data)?;
        Ok(Icon {
            data,
            ext: None,
            mime: Some(content_type.to_string()),
        })
    }

    async fn uwp_icon(display_info: &AppDisplayInfo) -> Result<Icon> {
        let (content_type, size, reader) = {
            let icon = display_info
                .GetLogo(Size {
                    Width: UWP_IMAGE_SIZE,
                    Height: UWP_IMAGE_SIZE,
                })?
                .OpenReadAsync()?
                .await?;
            let content_type = icon.ContentType()?.to_string_lossy();
            let size = icon.Size()? as usize;
            let reader = DataReader::CreateDataReader(&icon)?;
            (content_type, size, reader)
        };
        reader.LoadAsync(size as u32)?.await?;
        let mut data = vec![0u8; size];
        reader.ReadBytes(&mut data)?;
        Ok(Icon {
            data,
            ext: None,
            mime: Some(content_type.to_string()),
        })
    }
}

/// Generate a random color
pub fn random_color() -> String {
    // ref: https://github.com/catppuccin/catppuccin
    // Mocha colors
    let mut rng = rand::rng();

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
        assert_ne!(0, app_info.icon.unwrap().data.len());
        Ok(())
    }

    #[tokio::test]
    async fn app_info_from_uwp_store() -> Result<()> {
        let aumid = "Microsoft.Windows.NarratorQuickStart_8wekyb3d8bbwe!App";
        let app_info = AppInfo::from_uwp(aumid).await?;
        assert_eq!("Narrator", app_info.name);
        assert_eq!("Narrator Home", app_info.description);
        assert_eq!("Microsoft", app_info.company);
        assert_ne!(0, app_info.icon.unwrap().data.len());
        Ok(())
    }
}
