use tauri::State;
use util::config::{Config, DistractiveStreakSettings, FocusStreakSettings};
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
#[tracing::instrument(err, skip(state))]
pub async fn config_set_default_focus_streak_settings(
    state: State<'_, AppState>,
    value: FocusStreakSettings,
) -> AppResult<()> {
    let mut state = state.write().await;
    state
        .assume_init_mut()
        .config
        .set_default_focus_streak_settings(value)
        .context("set default focus streak settings")?;
    Ok(())
}

#[tauri::command]
#[tracing::instrument(err, skip(state))]
pub async fn config_set_default_distractive_streak_settings(
    state: State<'_, AppState>,
    value: DistractiveStreakSettings,
) -> AppResult<()> {
    let mut state = state.write().await;
    state
        .assume_init_mut()
        .config
        .set_default_distractive_streak_settings(value)
        .context("set default distractive streak settings")?;
    Ok(())
}

#[tauri::command]
#[tracing::instrument(err, skip(state))]
pub async fn config_reset_default_focus_streak_settings(
    state: State<'_, AppState>,
) -> AppResult<()> {
    let mut state = state.write().await;
    state
        .assume_init_mut()
        .config
        .set_default_focus_streak_settings(Default::default())
        .context("reset default focus streak settings")?;
    Ok(())
}

#[tauri::command]
#[tracing::instrument(err, skip(state))]
pub async fn config_reset_default_distractive_streak_settings(
    state: State<'_, AppState>,
) -> AppResult<()> {
    let mut state = state.write().await;
    state
        .assume_init_mut()
        .config
        .set_default_distractive_streak_settings(Default::default())
        .context("reset default distractive streak settings")?;
    Ok(())
}
