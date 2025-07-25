use tauri::State;
use util::config::Config;
use util::error::Context;
use util::tracing;

use crate::error::AppResult;
use crate::state::AppState;

#[tauri::command]
#[tracing::instrument(err, skip(state))]
pub async fn read_config(state: State<'_, AppState>) -> AppResult<Config> {
    let state = state.read().await;
    Ok(state.assume_init().config.clone())
}

#[tauri::command]
#[tracing::instrument(err, skip(state))]
pub async fn config_set_track_incognito(state: State<'_, AppState>, value: bool) -> AppResult<()> {
    let mut state = state.write().await;
    state
        .assume_init_mut()
        .config
        .set_track_incognito(value)
        .context("set track incognito")?;
    Ok(())
}

#[tauri::command]
#[tracing::instrument(err)]
pub async fn get_icons_dir() -> AppResult<String> {
    let path = Config::icons_dir().context("get icons dir")?;
    Ok(std::path::absolute(path)
        .context("get absolute path")?
        .to_string_lossy()
        .to_string())
}
