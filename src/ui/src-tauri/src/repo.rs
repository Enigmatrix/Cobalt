use std::collections::HashMap;

use data::db::repo::infused;
use data::entities::{App, Ref, Tag};
use tauri::State;
use util::error::Context;

use crate::error::AppResult;
use crate::state::{init_state, AppState, Initable};

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

#[tauri::command]
pub async fn copy_seed_db(state: State<'_, AppState>) -> AppResult<()> {
    // drop previous state - also drops the db connection
    {
        let mut state = state.lock().await;
        *state = Initable::Uninit;
    }

    std::fs::copy("../../../dbg/seed.db", "main.db").context("copy seed.db")?;

    // reinit state (repo)
    init_state(state).await?;
    Ok(())
}

#[tauri::command]
pub async fn update_usages_end(state: State<'_, AppState>) -> AppResult<()> {
    let now = platform::objects::Timestamp::now();
    let mut state = state.lock().await;
    state
        .assume_init_mut()
        .repo
        .update_usages_set_last(now.into())
        .await?;
    Ok(())
}
