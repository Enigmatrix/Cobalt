use common::errors::*;
use data::db::{AppUpdater, Database};
use data::entities::{App, AppIdentity};
use data::table::Ref;
use platform::objects::AppInfo;
use rand::seq::SliceRandom;

#[derive(Debug)]
pub struct AppInfoRequest {
    pub id: Ref<App>,
    pub app_identity: AppIdentity,
}

pub struct AppInfoResolver;

impl AppInfoResolver {
    /// Find information about the [App] and save it into the [Database]
    pub async fn resolve(&self, mut db: Database, req: AppInfoRequest) -> Result<()> {
        let mut updater = AppUpdater::from(&mut db).context("create app updater")?;
        let info: AppInfo = match req.app_identity {
            AppIdentity::Win32 { path } => AppInfo::from_win32(&path)
                .await
                .context("get win32 app info")?,
            AppIdentity::Uwp { aumid } => AppInfo::from_uwp(&aumid)
                .await
                .context("get uwp app info")?,
        };

        let color = self.random_color();

        {
            let icon_size = info.logo.Size().context("get app icon size")?;
            updater
                .update_app_icon_size(req.id.clone(), icon_size)
                .context("update app icon size")?;

            let mut icon_writer = updater
                .app_icon(req.id.clone())
                .context("open app icon for writing")?;

            info.copy_logo_to(&mut icon_writer)
                .await
                .context("copy logo to writer")?;
        }

        {
            updater
                .update_app(&App {
                    id: req.id,
                    name: info.name,
                    description: info.description,
                    company: info.company,
                    color,
                    ..Default::default()
                })
                .context("update app info")?;
        }

        Ok(())
    }

    fn random_color(&self) -> String {
        // ref: https://github.com/catppuccin/catppuccin
        // Mocha colors

        let mut rng = rand::rngs::ThreadRng::default();

        [
            "#f5e0dc", "#f2cdcd", "#f5c2e7", "#cba6f7", "#f38ba8", "#eba0ac", "#fab387", "#f9e2af",
            "#a6e3a1", "#94e2d5", "#89dceb", "#74c7ec", "#89b4fa", "#b4befe",
            "#cdd6f4", // Text
        ]
        .choose(&mut rng)
        .unwrap()
        .to_string()
    }
}

impl Default for AppInfoResolver {
    fn default() -> Self {
        Self
    }
}
