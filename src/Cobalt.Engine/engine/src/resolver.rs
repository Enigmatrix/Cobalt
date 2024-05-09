use data::entities::{App, AppIdentity, Ref};
use platform::objects::AppInfo;
use util::error::Result;

pub struct AppInfoResolver;

impl AppInfoResolver {
    async fn resolve(identity: AppIdentity) -> Result<AppInfo> {
        match identity {
            AppIdentity::Win32 { path } => AppInfo::from_win32(&path).await,
            AppIdentity::Uwp { aumid } => AppInfo::from_uwp(&aumid).await,
        }
    }

    pub async fn update_app(app: Ref<App>, identity: AppIdentity) -> Result<()> {
        todo!()
    }
}
