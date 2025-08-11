use data::db::{AppUpdater, DatabasePool};
use data::entities::{App, AppIdentity, Ref};
use platform::objects::AppInfo;
use platform::web::WebsiteInfo;
use util::config::Config;
use util::error::Result;
use util::future::fs;
use util::time::{TimeSystem, ToTicks};
use util::tracing::{info, warn};

/// Resolves application information asynchronously.
pub struct AppInfoResolver;

impl AppInfoResolver {
    /// Basic app filled out by identity
    pub fn default_from(identity: &AppIdentity, ts: impl TimeSystem) -> App {
        let app_info = match identity {
            AppIdentity::Uwp { aumid } => AppInfo::default_from_uwp(aumid),
            AppIdentity::Win32 { path } => AppInfo::default_from_win32_path(path),
            AppIdentity::Website { base_url } => {
                WebsiteInfo::default_from_url(WebsiteInfo::url_to_base_url(base_url)).into()
            }
        };
        App {
            id: Ref::new(0),
            name: app_info.name,
            description: app_info.description,
            company: app_info.company,
            color: app_info.color,
            icon: None, // app.icon might be set in app_info but we set it to None here anyway.
            identity: identity.clone(),
            tag_id: None,
            created_at: ts.now().to_ticks(),
            updated_at: ts.now().to_ticks(),
        }
    }

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
                let base_url = WebsiteInfo::url_to_base_url(base_url);
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

        // Store icon in filesystem if present
        let icon = if let Some(icon) = app_info.icon {
            let icons_dir = Config::icons_dir()?;
            let id = app.0.to_string();
            let ext = icon.deduce_ext();
            let file_name = format!("{}.{}", id, ext.unwrap_or("bin".to_string()));
            let icon_path = icons_dir.join(&file_name);
            fs::write(&icon_path, icon.data).await?;
            Some(file_name)
        } else {
            None
        };

        let app = App {
            id: app,
            name: app_info.name,
            description: app_info.description,
            company: app_info.company,
            color: app_info.color,
            icon,
            identity,
            tag_id: None,
            ..Default::default()
        };

        let now = platform::objects::Timestamp::now();
        updater.update_app(&app, now).await?;
        Ok(())
    }
}
