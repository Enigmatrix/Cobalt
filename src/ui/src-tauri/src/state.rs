use data::db::DatabasePool;
use data::db::repo::Repository;
use serde::{Deserialize, Serialize};
use tauri::State;
use tauri::async_runtime::RwLock;
use util::config::{Config, get_config};
use util::error::Result;
use util::tracing;

use crate::error::AppResult;

/// Initialize the app state. Should only be called once
#[tauri::command]
#[tracing::instrument(err, skip(state))]
pub async fn init_state(state: State<'_, AppState>) -> AppResult<()> {
    let mut state = state.write().await;
    if let &Initable::Init(_) = &*state {
        // do not reinit
        return AppResult::Ok(());
    }
    *state = Initable::Init(AppStateInner::new().await?);
    AppResult::Ok(())
}

/// Query options from the frontend
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct QueryOptions {
    /// The current time 'now'.
    pub now: Option<data::entities::Timestamp>,
}

impl QueryOptions {
    /// Get the current time 'now' from the options
    pub fn get_now(&self) -> platform::objects::Timestamp {
        if let Some(now) = self.now {
            platform::objects::Timestamp::from_ticks(now)
        } else {
            platform::objects::Timestamp::now()
        }
    }
}

/// The real app state
pub struct AppStateInner {
    pub db_pool: DatabasePool,
    pub config: Config,
}

impl AppStateInner {
    /// Create a new app state
    pub async fn new() -> Result<Self> {
        let config = get_config()?;
        util::setup(&config)?;
        let db_pool = DatabasePool::new(&config).await?;
        Ok(Self { db_pool, config })
    }

    /// Gets the repo with options
    pub async fn get_repo(&self) -> Result<Repository> {
        let db = self.db_pool.get_db().await?;
        Repository::new(db)
    }

    /// Shutdown the app state
    pub async fn shutdown(&self) -> Result<()> {
        self.db_pool.shutdown().await?;
        Ok(())
    }
}

#[derive(Default)]
/// Represents data that can be in Uninit or Init state
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
pub type AppState = RwLock<Initable<AppStateInner>>;
