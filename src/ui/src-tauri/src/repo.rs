use std::collections::HashMap;

use data::db::repo::{infused, WithDuration, WithGroupedDuration};
use data::entities::{App, Duration, Ref, Tag, Timestamp};
use tauri::State;
use util::error::Context;
use util::tracing;

use crate::error::AppResult;
use crate::state::{init_state, AppState, Initable, QueryOptions};

#[tauri::command]
#[tracing::instrument(err, skip(state))]
pub async fn get_apps(
    state: State<'_, AppState>,
    query_options: QueryOptions,
) -> AppResult<HashMap<Ref<App>, infused::App>> {
    let now = query_options.get_now();
    let mut repo = {
        let state = state.read().await;
        state.assume_init().get_repo().await?
    };
    let res = repo.get_apps(now).await?;
    Ok(res)
}

#[tauri::command]
#[tracing::instrument(err, skip(state))]
pub async fn get_tags(
    state: State<'_, AppState>,
    query_options: QueryOptions,
) -> AppResult<HashMap<Ref<Tag>, infused::Tag>> {
    let now = query_options.get_now();
    let mut repo = {
        let state = state.read().await;
        state.assume_init().get_repo().await?
    };
    let res = repo.get_tags(now).await?;
    Ok(res)
}

#[tauri::command]
#[tracing::instrument(err, skip(state))]
pub async fn get_app_durations(
    state: State<'_, AppState>,
    _query_options: QueryOptions,
    start: Timestamp,
    end: Timestamp,
) -> AppResult<HashMap<Ref<App>, WithDuration<App>>> {
    let mut repo = {
        let state = state.read().await;
        state.assume_init().get_repo().await?
    };
    let res = repo.get_app_durations(start, end).await?;
    Ok(res)
}

#[tauri::command]
#[tracing::instrument(err, skip(state))]
pub async fn get_app_durations_per_period(
    state: State<'_, AppState>,
    _query_options: QueryOptions,
    start: Timestamp,
    end: Timestamp,
    period: Duration,
) -> AppResult<HashMap<Ref<App>, Vec<WithGroupedDuration<App>>>> {
    let mut repo = {
        let state = state.read().await;
        state.assume_init().get_repo().await?
    };
    let res = repo
        .get_app_durations_per_period(start, end, period)
        .await?;
    Ok(res)
}

#[tauri::command]
#[tracing::instrument(err, skip(state))]
pub async fn copy_seed_db(state: State<'_, AppState>) -> AppResult<()> {
    // drop previous state - also drops the db connection
    {
        let mut state = state.write().await;
        *state = Initable::Uninit;
    }

    std::fs::copy("../../../dbg/seed.db", "main.db").context("copy seed.db")?;

    // reinit state (repo)
    init_state(state).await?;
    Ok(())
}

#[tauri::command]
#[tracing::instrument(err, skip(state))]
pub async fn update_usages_end(state: State<'_, AppState>) -> AppResult<()> {
    let now = platform::objects::Timestamp::now();
    let mut state = state.write().await;
    state
        .assume_init_mut()
        .get_repo()
        .await?
        .update_usages_set_last(now.into())
        .await?;
    Ok(())
}

#[tauri::command]
#[tracing::instrument(err, skip(state))]
pub async fn update_app(state: State<'_, AppState>, app: infused::UpdatedApp) -> AppResult<()> {
    let mut repo = {
        let mut state = state.write().await;
        state.assume_init_mut().get_repo().await?
    };
    repo.update_app(&app).await?;
    Ok(())
}

#[tauri::command]
#[tracing::instrument(err, skip(state))]
pub async fn create_tag(
    state: State<'_, AppState>,
    tag: infused::CreateTag,
) -> AppResult<Ref<Tag>> {
    let mut repo = {
        let mut state = state.write().await;
        state.assume_init_mut().get_repo().await?
    };
    Ok(repo.create_tag(&tag).await?)
}
