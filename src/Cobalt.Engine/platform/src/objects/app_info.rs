use std::io::Write;

use common::errors::*;
use windows::core::ComInterface;
use windows::ApplicationModel::AppInfo as UWPAppInfo;
use windows::Foundation::Size;
use windows::Storage::FileProperties::ThumbnailMode;
use windows::Storage::StorageFile;
use windows::Storage::Streams::{
    Buffer, IBuffer, IRandomAccessStreamWithContentType, InputStreamOptions,
};
use windows::Win32::System::WinRT::IBufferByteAccess;

use crate::objects::FileVersionInfo;

pub type Logo = IRandomAccessStreamWithContentType;

// TODO app_info fetch should be result -> option and should log the result

#[derive(Clone, Debug)]
pub struct AppInfo {
    pub name: String,
    pub description: String,
    pub company: String,
    pub logo: Logo,
}

impl AppInfo {
    unsafe fn as_mut_bytes(buffer: &IBuffer) -> Result<&mut [u8]> {
        let interop = buffer.cast::<IBufferByteAccess>()?;
        let data = interop.Buffer()?;
        Ok(std::slice::from_raw_parts_mut(data, buffer.Length()? as _))
    }

    pub async fn copy_logo_to<W: Write>(&self, writer: &mut W) -> Result<()> {
        let capacity = 4096;
        let buffer = Buffer::Create(capacity)?;
        loop {
            let win_buffer = self
                .logo
                .ReadAsync(&buffer, capacity, InputStreamOptions::None)?
                .await
                .context("read from logo stream")?;
            let buf = unsafe { Self::as_mut_bytes(&win_buffer)? };
            if buf.is_empty() {
                break;
            }
            writer.write_all(buf).context("write bytes to writer")?;
        }
        Ok(())
    }

    /// Create a new [AppInfo] of a Win32 progrem from its path
    pub async fn from_win32(path: &str) -> Result<Self> {
        let file = StorageFile::GetFileFromPathAsync(&path.into())?
            .await
            .context("get storage file")?;

        let mut fv = FileVersionInfo::new(path).context("get file version info")?;

        let logo = file
            .GetThumbnailAsyncOverloadDefaultOptions(ThumbnailMode::SingleItem, 64)?
            .await
            .context("get thumbnail")?
            .cast()?;

        Ok(AppInfo {
            name: fv.query_value("ProductName")?,
            description: fv.query_value("FileDescription")?,
            company: fv.query_value("CompanyName")?,
            logo,
        })
    }

    /// Create a new [AppInfo] of a given UWP app from its AUMID
    pub async fn from_uwp(aumid: &str) -> Result<Self> {
        let app_info =
            UWPAppInfo::GetFromAppUserModelId(&aumid.into()).context("get app info with aumid")?;
        let display_info = app_info.DisplayInfo()?;
        let package = app_info.Package()?;

        let logo = display_info
            .GetLogo(Size {
                Width: 64.0,
                Height: 64.0,
            })?
            .OpenReadAsync()?
            .await
            .context("open logo for reading")?;

        Ok(AppInfo {
            name: display_info.DisplayName()?.to_string_lossy(),
            description: display_info.Description()?.to_string_lossy(),
            company: package.PublisherDisplayName()?.to_string_lossy(),
            logo,
        })
    }
}
