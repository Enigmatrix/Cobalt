use util::error::Result;

pub struct AppInfo {}

impl AppInfo {
    pub async fn from_win32(path: &str) -> Result<AppInfo> {
        todo!()
    }

    pub async fn from_uwp(aumid: &str) -> Result<AppInfo> {
        todo!()
    }
}
