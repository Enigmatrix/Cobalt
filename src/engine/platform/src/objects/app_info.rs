use utils::errors::*;
use windows::ApplicationModel::AppInfo as UWPAppInfo;
use windows::Foundation::Size;
use windows::Storage::FileProperties::ThumbnailMode;
use windows::Storage::StorageFile;
use windows::Storage::Streams::IRandomAccessStreamWithContentType;

use crate::objects::FileVersionInfo;

#[derive(Clone, Debug)]
pub struct AppInfo {
    pub name: String,
    pub description: String,
    pub company: String,
    pub logo: IRandomAccessStreamWithContentType,
}

impl AppInfo {
    pub async fn from_win32(path: &str) -> Result<Self> {
        let file = StorageFile::GetFileFromPathAsync(&path.into())?
            .await
            .context("get storage file")?;

        let mut fv = FileVersionInfo::new(path).context("get file version info")?;

        let logo = file
            .GetThumbnailAsyncOverloadDefaultOptions(ThumbnailMode::SingleItem, 64)?
            .await
            .context("get thumbnail")?;

        Ok(AppInfo {
            name: fv.query_value("ProductName")?,
            description: fv.query_value("FileDescription")?,
            company: fv.query_value("CompanyName")?,
            logo: logo
                .try_into()
                .context("cast StorageItemThumbnail to IRandomAccessStreamWithContentType")?,
        })
    }

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
