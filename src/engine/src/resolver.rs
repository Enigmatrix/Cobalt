use data::db::{AppUpdater, DatabasePool};
use data::entities::{App, AppIdentity, Ref};
use platform::objects::{AppInfo, WebsiteInfo};
use rand::prelude::SliceRandom;
use rand::rngs::ThreadRng;
use util::error::Result;
use util::tracing::info;

/// Resolves application information asynchronously.
pub struct AppInfoResolver;

impl AppInfoResolver {
    /// Resolve the application information for the given [AppIdentity].
    async fn resolve(identity: &AppIdentity) -> Result<AppInfo> {
        match identity {
            AppIdentity::Win32 { path } => AppInfo::from_win32(path).await,
            AppIdentity::Uwp { aumid } => AppInfo::from_uwp(aumid).await,
            AppIdentity::Website { base_url } => {
                let base_url = WebsiteInfo::url_to_base_url(base_url)?;
                Ok(WebsiteInfo::from_base_url(base_url).await?.into())
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
            color: Self::random_color(),
            identity,
            tag_id: None,
            icon: app_info.logo,
            ..Default::default()
        };
        let now = platform::objects::Timestamp::now();
        updater.update_app(&app, now).await?;
        Ok(())
    }

    fn random_color() -> String {
        // ref: https://github.com/catppuccin/catppuccin
        // Mocha colors
        let mut rng = ThreadRng::default();

        [
            "#f5e0dc", "#f2cdcd", "#f5c2e7", "#cba6f7", "#f38ba8", "#eba0ac", "#fab387", "#f9e2af",
            "#a6e3a1", "#94e2d5", "#89dceb", "#74c7ec", "#89b4fa", "#b4befe",
            "#cdd6f4", // Text
        ]
        .choose_mut(&mut rng)
        .unwrap()
        .to_string()
    }
}
