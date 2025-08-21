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
use windows::Win32::Storage::Packaging::Appx::{
    PackageNameAndPublisherIdFromFamilyName, ParseApplicationUserModelId,
};
use windows::core::{AgileReference, HSTRING, PCWSTR, PWSTR};

use crate::adapt_size2;
use crate::buf::WideBuffer;
use crate::error::IntoResult;
use crate::objects::{FileVersionInfo, SquirrelBaseDir};

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
        // Try to parse the AUMID to extract meaningful information
        if let Ok(parsed) = Aumid::parse(aumid) {
            Self {
                name: parsed.package_name.clone(),
                description: parsed.package_name.clone(),
                company: parsed.publisher_id,
                color: random_color(),
                icon: None,
            }
        } else {
            // Fallback to original behavior if parsing fails
            Self {
                name: aumid.to_string(),
                description: aumid.to_string(),
                company: "".to_string(),
                color: random_color(),
                icon: None,
            }
        }
    }

    /// Create a default [AppInfo] from a Squirrel identifier and file
    pub fn default_from_squirrel(identifier: &str, file: &str) -> Self {
        Self {
            name: file.to_string(),
            description: file.to_string(),
            company: identifier.to_string(),
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

    /// Create a new [AppInfo] of a given Squirrel app from its identifier and file
    pub async fn from_squirrel(identifier: &str, file: &str) -> Result<Self> {
        let base_dir = SquirrelBaseDir::new(identifier.to_string())?;
        let exe = base_dir.latest_exe(file)?;
        Self::from_win32(exe.path()?.to_string_lossy().as_ref())
            .await
            .with_context(|| format!("call from_win32 for squirrel exe {exe:?}"))
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

/// Parsed components of an Application User Model ID (AUMID)
pub struct Aumid {
    /// Package family name (e.g., "Microsoft.Windows.Calculator_8wekyb3d8bbwe")
    pub package_family_name: String,
    /// Package relative application ID (e.g., "App")
    pub package_relative_app_id: String,
    /// Package name (e.g., "Microsoft.Windows.Calculator")
    pub package_name: String,
    /// Publisher ID (e.g., "8wekyb3d8bbwe")
    pub publisher_id: String,
}

impl Aumid {
    /// Parse an Application User Model ID (AUMID) into its components using Windows API
    pub fn parse(aumid: &str) -> Result<Self> {
        let aumid_hstring = HSTRING::from(aumid);
        let aumid_pcwstr = PCWSTR::from_raw(aumid_hstring.as_ptr());

        // Parse the AUMID to get package family name and package relative app ID
        let (package_family_name, package_relative_app_id) = adapt_size2!(
            u16,
            package_family_name_length: 0 => 1024,
            package_family_name,
            package_relative_app_id_length: 0 => 1024,
            package_relative_app_id,
            unsafe {
                ParseApplicationUserModelId(
                    aumid_pcwstr,
                    &mut package_family_name_length,
                    Some(PWSTR(package_family_name.as_mut_ptr())),
                    &mut package_relative_app_id_length,
                    Some(PWSTR(package_relative_app_id.as_mut_ptr())),
                )
            }.into_result().map(|_| {
                (package_family_name.with_length(package_family_name_length as usize).to_string_lossy_except_null_terminator(),
                package_relative_app_id.with_length(package_relative_app_id_length as usize).to_string_lossy_except_null_terminator())
            })
        )?;

        // Parse the package family name to get package name and publisher ID
        let package_family_name_hstring = HSTRING::from(&package_family_name);
        let package_family_name_pcwstr = PCWSTR::from_raw(package_family_name_hstring.as_ptr());

        let (package_name, publisher_id) = adapt_size2!(
            u16,
            package_name_length: 0 => 1024,
            package_name,
            publisher_id_length: 0 => 1024,
            publisher_id,
            unsafe {
                PackageNameAndPublisherIdFromFamilyName(
                    package_family_name_pcwstr,
                    &mut package_name_length,
                    Some(PWSTR(package_name.as_mut_ptr())),
                    &mut publisher_id_length,
                    Some(PWSTR(publisher_id.as_mut_ptr())),
                )
            }.into_result().map(|_| {
                (package_name.with_length(package_name_length as usize).to_string_lossy_except_null_terminator(),
                publisher_id.with_length(publisher_id_length as usize).to_string_lossy_except_null_terminator())
            })
        )?;

        Ok(Aumid {
            package_family_name,
            package_relative_app_id,
            package_name,
            publisher_id,
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

    #[test]
    fn test_parse_aumid() -> Result<()> {
        let aumid = "Microsoft.Windows.NarratorQuickStart_8wekyb3d8bbwe!App";
        let parsed = Aumid::parse(aumid)?;

        assert_eq!(
            parsed.package_family_name,
            "Microsoft.Windows.NarratorQuickStart_8wekyb3d8bbwe"
        );
        assert_eq!(parsed.package_relative_app_id, "App");
        assert_eq!(parsed.package_name, "Microsoft.Windows.NarratorQuickStart");
        assert_eq!(parsed.publisher_id, "8wekyb3d8bbwe");

        Ok(())
    }

    #[test]
    fn test_default_from_uwp_with_parsing() {
        let aumid = "Microsoft.Windows.Calculator_8wekyb3d8bbwe!App";
        let app_info = AppInfo::default_from_uwp(aumid);

        // Should use parsed information
        assert_eq!(app_info.name, "Microsoft.Windows.Calculator");
        assert_eq!(app_info.description, "Microsoft.Windows.Calculator");
        assert_eq!(app_info.company, "8wekyb3d8bbwe");
    }

    #[test]
    fn test_default_from_uwp_fallback() {
        let invalid_aumid = "InvalidAUMID";
        let app_info = AppInfo::default_from_uwp(invalid_aumid);

        // Should fallback to original behavior
        assert_eq!(app_info.name, "InvalidAUMID");
        assert_eq!(app_info.description, "InvalidAUMID");
        assert_eq!(app_info.company, "");
    }
}
