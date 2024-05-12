use data::{
    db::{AppUpdater, Database},
    entities::{App, AppIdentity, Ref},
};
use platform::objects::AppInfo;
use rand::{prelude::SliceRandom, rngs::ThreadRng};
use util::{config::Config, error::Result};

pub struct AppInfoResolver;

impl AppInfoResolver {
    async fn resolve(identity: &AppIdentity) -> Result<AppInfo> {
        match identity {
            AppIdentity::Win32 { path } => AppInfo::from_win32(path).await,
            AppIdentity::Uwp { aumid } => AppInfo::from_uwp(aumid).await,
        }
    }

    pub async fn update_app(config: &Config, app: Ref<App>, identity: AppIdentity) -> Result<()> {
        let app_info = Self::resolve(&identity).await?;

        let mut db = Database::new(config)?;
        let mut updater = AppUpdater::new(&mut db)?;

        let icon_size = app_info.logo_size()?;
        let mut icon = updater.app_icon_writer(app.clone(), icon_size)?;
        app_info.copy_logo(&mut icon).await?;

        let app = App {
            id: app,
            name: app_info.name,
            description: app_info.description,
            company: app_info.company,
            color: Self::random_color(),
            identity,
        };
        updater.update_app(&app)?;
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
