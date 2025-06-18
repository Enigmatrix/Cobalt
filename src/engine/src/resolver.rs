use std::path::Path;

use data::db::{AppUpdater, DatabasePool};
use data::entities::{App, AppIdentity, Ref};
use platform::objects::{AppInfo, WebsiteInfo};
use util::config::Config;
use util::error::Result;
use util::future::fs;
use util::tracing::{info, warn};

/// Resolves application information asynchronously.
pub struct AppInfoResolver;

impl AppInfoResolver {
    /// Resolve the application information for the given [AppIdentity].
    async fn resolve(identity: &AppIdentity) -> Result<AppInfo> {
        match identity {
            AppIdentity::Win32 { path } => {
                Ok(AppInfo::from_win32(path).await.unwrap_or_else(|e| {
                    warn!("failed to get app info for {path}, using default: {e:?}");
                    AppInfo::default_from_win32_path(path)
                }))
            }
            AppIdentity::Uwp { aumid } => AppInfo::from_uwp(aumid).await,
            AppIdentity::Website { base_url } => {
                let base_url = WebsiteInfo::url_to_base_url(base_url)?;
                Ok(WebsiteInfo::from_base_url(base_url.clone())
                    .await
                    .unwrap_or_else(|e| {
                        warn!("failed to get website info for {base_url}, using default: {e:?}");
                        WebsiteInfo::default_from_url(base_url)
                    })
                    .into())
            }
        }
    }

    /// Resolves and updates the application information for the given [AppIdentity] into the [Database].
    pub async fn update_app(
        db_pool: DatabasePool,
        app: Ref<App>,
        identity: AppIdentity,
    ) -> Result<()> {
        info!("updating app info {:?} ({:?})", app, identity);
        let app_info = Self::resolve(&identity).await?;

        let db = db_pool.get_db().await?;
        let mut updater = AppUpdater::new(db)?;

        let app = App {
            id: app,
            name: app_info.name,
            description: app_info.description,
            company: app_info.company,
            color: app_info.color,
            identity,
            tag_id: None,
            ..Default::default()
        };

        // Store icon in filesystem if present
        if let Some(icon_data) = app_info.logo {
            let icons_dir = Config::icons_dir()?;
            let icon_path = Path::new(&icons_dir).join(app.id.0.to_string());
            fs::write(icon_path, icon_data).await?;
        }

        let now = platform::objects::Timestamp::now();
        updater.update_app(&app, now).await?;
        Ok(())
    }
}
