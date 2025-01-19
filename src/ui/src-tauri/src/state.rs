use data::db::repo::Repository;
use data::db::Database;
use tauri::async_runtime::Mutex;
use util::config::{get_config, Config};
use util::error::*;

use crate::error::*;

#[tauri::command]
/// Initiailize the app state. Should only be called once
pub async fn init_state(state: tauri::State<'_, AppState>) -> AppResult<()> {
    let mut state = state.lock().await;
    *state = Initable::Init(AppStateInner::new().await?);
    AppResult::Ok(())
}

/// The real app state
pub struct AppStateInner {
    pub repo: Repository,
    pub config: Config,
}

impl AppStateInner {
    /// Create a new app state
    pub async fn new() -> Result<Self> {
        let config = get_config()?;
        let db = Database::new(&config).await?;
        let repo = Repository::new(db)?;
        Ok(Self { repo, config })
    }
}

#[derive(Default)]
/// Represents data that be in Uninit or Init state
pub enum Initable<T> {
    #[default]
    Uninit,
    Init(T),
}

impl<T> Initable<T> {
    /// Assume the data inside is already initialized
    pub fn assume_init(&self) -> &T {
        match self {
            Initable::Init(inner) => inner,
            Initable::Uninit => panic!("Uninitialized state accessed"),
        }
    }

    /// Assume the data inside is already initialized
    pub fn assume_init_mut(&mut self) -> &mut T {
        match self {
            Initable::Init(inner) => inner,
            Initable::Uninit => panic!("Uninitialized state accessed"),
        }
    }
}

/// App State wrapped for use
pub type AppState = Mutex<Initable<AppStateInner>>;
