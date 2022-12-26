use utils::{errors::*, tracing::info};
use windows::Storage::StorageFile;

use crate::objects::FileVersionInfo;

#[derive(Clone, Debug)]
pub struct AppInfo {
    pub name: String,
    pub description: String,
    pub company: String,
    // pub icon:
}

impl AppInfo {
    pub async fn from_win32(path: &str) -> Result<Self> {
        let file = StorageFile::GetFileFromPathAsync(&path.into())?
            .await
            .with_context(|| format!("get storage file for path: {path}"))?;

        let mut fv = FileVersionInfo::new(path).context("get file version info")?;
        let product_name = fv
            .query_value("ProductName")
            .context("get ProductName field")?;
        let file_description = fv
            .query_value("FileDescription")
            .context("get FileDescription field")?;
        let company_name = fv
            .query_value("CompanyName")
            .context("get CompanyName field")?;

        Ok(AppInfo {
            name: product_name,
            description: file_description,
            company: company_name,
        })
    }
}
