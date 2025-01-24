use std::collections::HashMap;

use data::db::repo::infused;
use data::entities::{App, Ref, Tag};
use tauri::State;

use crate::error::AppResult;
use crate::state::AppState;

#[tauri::command]
pub async fn get_apps(state: State<'_, AppState>) -> AppResult<HashMap<Ref<App>, infused::App>> {
    let now = platform::objects::Timestamp::now();
    let mut state = state.lock().await;
    let res = state.assume_init_mut().repo.get_apps(now).await?;
    Ok(res)
}

#[tauri::command]
pub async fn get_tags(state: State<'_, AppState>) -> AppResult<HashMap<Ref<Tag>, infused::Tag>> {
    let now = platform::objects::Timestamp::now();
    let mut state = state.lock().await;
    let res = state.assume_init_mut().repo.get_tags(now).await?;
    Ok(res)
}
