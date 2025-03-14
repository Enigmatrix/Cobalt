use std::collections::HashMap;

use data::db::infused;
use data::entities::{Alert, App, InteractionPeriod, Period, Ref, SystemEvent, Tag, Timestamp};
use tauri::State;
use util::error::Context;
use util::time::ToTicks;
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
pub async fn get_alerts(
    state: State<'_, AppState>,
    query_options: QueryOptions,
) -> AppResult<HashMap<Ref<Alert>, infused::Alert>> {
    let now = query_options.get_now();
    let mut repo = {
        let state = state.read().await;
        state.assume_init().get_repo().await?
    };
    let res = repo.get_alerts(now).await?;
    Ok(res)
}

#[tauri::command]
#[tracing::instrument(err, skip(state))]
pub async fn get_app_durations(
    state: State<'_, AppState>,
    _query_options: QueryOptions,
    start: Timestamp,
    end: Timestamp,
) -> AppResult<HashMap<Ref<App>, infused::WithDuration<App>>> {
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
    period: Period,
) -> AppResult<HashMap<Ref<App>, Vec<infused::WithGroupedDuration<App>>>> {
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
pub async fn get_tag_durations_per_period(
    state: State<'_, AppState>,
    _query_options: QueryOptions,
    start: Timestamp,
    end: Timestamp,
    period: Period,
) -> AppResult<HashMap<Ref<Tag>, Vec<infused::WithGroupedDuration<Tag>>>> {
    let mut repo = {
        let state = state.read().await;
        state.assume_init().get_repo().await?
    };
    let res = repo
        .get_tag_durations_per_period(start, end, period)
        .await?;
    Ok(res)
}

#[tauri::command]
#[tracing::instrument(err, skip(state))]
pub async fn copy_seed_db(state: State<'_, AppState>) -> AppResult<()> {
    // drop previous state - also drops the db connection
    {
        let mut state = state.write().await;
        state.assume_init().shutdown().await?;
        *state = Initable::Uninit;
    }

    fn check_and_remove(file: &str) -> util::error::Result<()> {
        if std::fs::metadata(file)
            .map(|f| f.is_file())
            .unwrap_or(false)
        {
            std::fs::remove_file(file).context(format!("remove {}", file))?;
        }
        Ok(())
    }

    // remove previous files (especially the non-main.db files)
    check_and_remove("main.db")?;
    check_and_remove("main.db-journal")?;
    check_and_remove("main.db-shm")?;
    check_and_remove("main.db-wal")?;

    std::fs::copy("../../../dev/seed.db", "main.db").context("copy seed.db")?;

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
        .update_usages_set_last(now.to_ticks())
        .await?;
    Ok(())
}

#[tauri::command]
#[tracing::instrument(err, skip(state))]
pub async fn update_app(state: State<'_, AppState>, app: infused::UpdatedApp) -> AppResult<()> {
    let now = platform::objects::Timestamp::now();
    let mut repo = {
        let mut state = state.write().await;
        state.assume_init_mut().get_repo().await?
    };
    repo.update_app(&app, now).await?;
    Ok(())
}

#[tauri::command]
#[tracing::instrument(err, skip(state))]
pub async fn update_tag(state: State<'_, AppState>, tag: infused::UpdatedTag) -> AppResult<()> {
    let now = platform::objects::Timestamp::now();
    let mut repo = {
        let mut state = state.write().await;
        state.assume_init_mut().get_repo().await?
    };
    repo.update_tag(&tag, now).await?;
    Ok(())
}

#[tauri::command]
#[tracing::instrument(err, skip(state))]
pub async fn update_tag_apps(
    state: State<'_, AppState>,
    tag_id: Ref<Tag>,
    removed_apps: Vec<Ref<App>>,
    added_apps: Vec<Ref<App>>,
) -> AppResult<()> {
    let now = platform::objects::Timestamp::now();
    let mut repo = {
        let mut state = state.write().await;
        state.assume_init_mut().get_repo().await?
    };
    repo.update_tag_apps(tag_id, removed_apps, added_apps, now)
        .await?;
    Ok(())
}

#[tauri::command]
#[tracing::instrument(err, skip(state))]
pub async fn create_tag(
    state: State<'_, AppState>,
    tag: infused::CreateTag,
) -> AppResult<infused::Tag> {
    let now = platform::objects::Timestamp::now();
    let mut repo = {
        let mut state = state.write().await;
        state.assume_init_mut().get_repo().await?
    };
    Ok(repo.create_tag(&tag, now).await?)
}

#[tauri::command]
#[tracing::instrument(err, skip(state))]
pub async fn remove_tag(state: State<'_, AppState>, tag_id: Ref<Tag>) -> AppResult<()> {
    let mut repo = {
        let mut state = state.write().await;
        state.assume_init_mut().get_repo().await?
    };
    Ok(repo.remove_tag(tag_id).await?)
}

#[tauri::command]
#[tracing::instrument(err, skip(state))]
pub async fn create_alert(
    state: State<'_, AppState>,
    alert: infused::CreateAlert,
) -> AppResult<infused::Alert> {
    let now = platform::objects::Timestamp::now();
    let mut repo = {
        let mut state = state.write().await;
        state.assume_init_mut().get_repo().await?
    };
    Ok(repo.create_alert(alert, now).await?)
}

#[tauri::command]
#[tracing::instrument(err, skip(state))]
pub async fn update_alert(
    state: State<'_, AppState>,
    prev: infused::Alert,
    next: infused::UpdatedAlert,
) -> AppResult<infused::Alert> {
    let now = platform::objects::Timestamp::now();
    let mut repo = {
        let mut state = state.write().await;
        state.assume_init_mut().get_repo().await?
    };
    Ok(repo.update_alert(prev, next, now).await?)
}

#[tauri::command]
#[tracing::instrument(err, skip(state))]
pub async fn remove_alert(state: State<'_, AppState>, alert_id: Ref<Alert>) -> AppResult<()> {
    let mut repo = {
        let mut state = state.write().await;
        state.assume_init_mut().get_repo().await?
    };
    Ok(repo.remove_alert(alert_id).await?)
}

#[tauri::command]
#[tracing::instrument(err, skip(state))]
pub async fn create_alert_event_ignore(
    state: State<'_, AppState>,
    alert_id: Ref<Alert>,
    timestamp: Timestamp,
) -> AppResult<()> {
    let mut repo = {
        let mut state = state.write().await;
        state.assume_init_mut().get_repo().await?
    };
    Ok(repo.create_alert_event_ignore(alert_id, timestamp).await?)
}

#[tauri::command]
#[tracing::instrument(err, skip(state))]
pub async fn get_app_session_usages(
    state: State<'_, AppState>,
    start: Timestamp,
    end: Timestamp,
) -> AppResult<infused::AppSessionUsages> {
    let mut repo = {
        let mut state = state.write().await;
        state.assume_init_mut().get_repo().await?
    };
    Ok(repo.get_app_session_usages(start, end).await?)
}

#[tauri::command]
#[tracing::instrument(err, skip(state))]
pub async fn get_interaction_periods(
    state: State<'_, AppState>,
    start: Timestamp,
    end: Timestamp,
) -> AppResult<Vec<InteractionPeriod>> {
    let mut repo = {
        let mut state = state.write().await;
        state.assume_init_mut().get_repo().await?
    };
    Ok(repo.get_interaction_periods(start, end).await?)
}

#[tauri::command]
#[tracing::instrument(err, skip(state))]
pub async fn get_system_events(
    state: State<'_, AppState>,
    start: Timestamp,
    end: Timestamp,
) -> AppResult<Vec<SystemEvent>> {
    let mut repo = {
        let mut state = state.write().await;
        state.assume_init_mut().get_repo().await?
    };
    Ok(repo.get_system_events(start, end).await?)
}
