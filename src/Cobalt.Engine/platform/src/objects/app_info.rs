use std::io::Write;

use util::error::Result;
use windows::core::Interface;
use windows::ApplicationModel::AppInfo as UWPAppInfo;
use windows::Foundation::Size;
use windows::Storage::Streams::{Buffer, IBuffer, InputStreamOptions};
use windows::Storage::{
    FileProperties::ThumbnailMode, StorageFile, Streams::IRandomAccessStreamWithContentType,
};
use windows::Win32::System::WinRT::IBufferByteAccess;

use crate::objects::FileVersionInfo;

pub type Logo = IRandomAccessStreamWithContentType;

pub struct AppInfo {
    pub name: String,
    pub description: String,
    pub company: String,
    logo: Logo,
}

pub const WIN32_IMAGE_SIZE: u32 = 64;
pub const UWP_IMAGE_SIZE: f32 = 256.0;

impl AppInfo {
    unsafe fn as_mut_bytes(buffer: &IBuffer) -> Result<&mut [u8]> {
        let interop = buffer.cast::<IBufferByteAccess>()?;
        let data = interop.Buffer()?;
        Ok(std::slice::from_raw_parts_mut(data, buffer.Length()? as _))
    }

    pub async fn copy_logo<W: Write>(&self, writer: &mut W) -> Result<()> {
        let size = 0x1000;
        let buffer = Buffer::Create(size)?;
        loop {
            let buffer = self
                .logo
                .ReadAsync(&buffer, size, InputStreamOptions::ReadAhead)?
                .await?;
            let buf = unsafe { Self::as_mut_bytes(&buffer)? };
            if buf.is_empty() {
                break;
            }
            writer.write_all(buf)?;
        }
        Ok(())
    }

    pub fn logo_size(&self) -> Result<u64> {
        Ok(self.logo.Size()?)
    }

    /// Create a new [AppInfo] of a Win32 progrem from its path
    pub async fn from_win32(path: &str) -> Result<Self> {
        let file = StorageFile::GetFileFromPathAsync(&path.into())?.await?;

        let mut fv = FileVersionInfo::new(path)?;

        let logo = file
            .GetThumbnailAsyncOverloadDefaultOptions(ThumbnailMode::SingleItem, WIN32_IMAGE_SIZE)?
            .await?
            .cast()?;

        Ok(AppInfo {
            // not sure why FileDescription is the actual name of the app...
            name: fv.query_value("FileDescription")?,
            description: fv.query_value("ProductName")?,
            company: fv.query_value("CompanyName")?,
            logo,
        })
    }

    /// Create a new [AppInfo] of a given UWP app from its AUMID
    pub async fn from_uwp(aumid: &str) -> Result<Self> {
        let app_info = UWPAppInfo::GetFromAppUserModelId(&aumid.into())?;
        let display_info = app_info.DisplayInfo()?;
        let package = app_info.Package()?;

        let logo = display_info
            .GetLogo(Size {
                Width: UWP_IMAGE_SIZE,
                Height: UWP_IMAGE_SIZE,
            })?
            .OpenReadAsync()?
            .await?;

        Ok(AppInfo {
            name: display_info.DisplayName()?.to_string_lossy(),
            description: display_info.Description()?.to_string_lossy(),
            company: package.PublisherDisplayName()?.to_string_lossy(),
            logo,
        })
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    async fn app_info_from_win32_notepad() -> Result<()> {
        let notepad = r#"C:\Windows\System32\notepad.exe"#;
        let app_info = AppInfo::from_win32(notepad).await?;
        assert_eq!("Notepad", app_info.name);
        assert_eq!("Microsoft® Windows® Operating System", app_info.description);
        assert_eq!("Microsoft Corporation", app_info.company);
        let logo_size = app_info.logo_size()?;
        let mut logo = Vec::new();
        app_info.copy_logo(&mut logo).await?;
        assert_ne!(0, logo_size);
        assert_eq!(logo_size as usize, logo.len());
        Ok(())
    }

    #[tokio::test]
    async fn app_info_from_uwp_store() -> Result<()> {
        let aumid = "Microsoft.WindowsStore_8wekyb3d8bbwe!App";
        let app_info = AppInfo::from_uwp(aumid).await?;
        assert_eq!("Microsoft Store", app_info.name);
        assert_eq!(
            "This is the description of Microsoft Store.",
            app_info.description
        );
        assert_eq!("Microsoft Corporation", app_info.company);
        let logo_size = app_info.logo_size()?;
        let mut logo = Vec::new();
        app_info.copy_logo(&mut logo).await?;
        assert_ne!(0, logo_size);
        assert_eq!(logo_size as usize, logo.len());
        Ok(())
    }
}
