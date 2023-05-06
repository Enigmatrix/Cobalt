use common::errors::*;
use data::*;
use platform::objects::AppInfo;

pub struct AppInfoRequest {
    id: Ref<App>,
    app_identity: AppIdentity,
}

pub struct AppInfoResolver;

impl AppInfoResolver {
    /// Find information about the [App] and save it into the [Database]
    pub async fn resolve(&self, mut db: Database, req: AppInfoRequest) -> Result<()> {
        let mut updater = AppUpdater::from(&mut db).context("create app updater")?;
        let info = match req.app_identity {
            AppIdentity::Win32 { path } => AppInfo::from_win32(&path)
                .await
                .context("get win32 app info")?,
            AppIdentity::Uwp { aumid } => AppInfo::from_uwp(&aumid)
                .await
                .context("get uwp app info")?,
        };
        let icon_size = info.logo.Size().context("get app icon size")?;

        updater
            .update_app(
                &App {
                    id: req.id.clone(),
                    name: info.name,
                    description: info.description,
                    company: info.company,
                    ..Default::default()
                },
                icon_size,
            )
            .context("update app info")?;

        let mut icon_writer = updater
            .app_icon(req.id)
            .context("open app icon for writing")?;
        AppInfo::copy_logo_to(info.logo, &mut icon_writer)
            .await
            .context("copy logo to witer")?;

        Ok(())
    }
}
