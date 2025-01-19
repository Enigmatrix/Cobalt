use std::collections::HashMap;

use data::db::repo::infused;
use data::entities::{App, Ref, Tag};
use tauri::State;

use crate::error::AppResult;
use crate::state::AppState;

#[tauri::command]
pub async fn get_apps(state: State<'_, AppState>) -> AppResult<HashMap<Ref<App>, infused::App>> {
    let mut state = state.lock().await;
    let res = state.assume_init_mut().repo.get_apps().await?;
    Ok(res)
}

#[tauri::command]
pub async fn get_tags(state: State<'_, AppState>) -> AppResult<HashMap<Ref<Tag>, infused::Tag>> {
    let mut state = state.lock().await;
    let res = state.assume_init_mut().repo.get_tags().await?;
    Ok(res)
}
