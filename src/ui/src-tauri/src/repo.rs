use std::collections::HashMap;

use data::db::infused;
use data::entities::{
    Alert, AlertEvent, App, InteractionPeriod, Period, Reason, Ref, SystemEvent, Tag, Timestamp,
};
use tauri::State;
use util::error::Context;
use util::time::ToTicks;
use util::tracing;

use crate::error::AppResult;
use crate::state::{AppState, Initable, QueryOptions, init_state};

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
pub async fn copy_from_seed_db(state: State<'_, AppState>) -> AppResult<()> {
    // drop previous state - also drops the db connection
    {
        let mut state = state.write().await;
        state.assume_init().shutdown().await?;
        *state = Initable::Uninit;
    }

    let base_dir = util::config::Config::config_base_dir().context("config base dir")?;
    util::fs::remove_icon_files(&base_dir)?;
    util::fs::remove_db_files(&base_dir)?;

    let to_file = util::config::Config::config_path("main.db").context("config path")?;
    std::fs::copy("../../../dev/seed.db", to_file).context("copy seed.db")?;

    // reinit state for the sake of repo
    init_state(state.clone()).await?;

    {
        let state = state.write().await;
        let mut repo = state.assume_init().get_repo().await?;
        repo.extract_seed_db_icons().await?;
    }

    // shutdown state again, lose the cached type info for sqlx
    {
        let mut state = state.write().await;
        state.assume_init().shutdown().await?;
        *state = Initable::Uninit;
    }
    // reinit state
    init_state(state.clone()).await?;

    Ok(())
}

#[tauri::command]
#[tracing::instrument(err, skip(state))]
pub async fn copy_from_install_db(state: State<'_, AppState>) -> AppResult<()> {
    use util::error::ContextCompat;

    // drop previous state - also drops the db connection
    {
        let mut state = state.write().await;
        state.assume_init().shutdown().await?;
        *state = Initable::Uninit;
    }

    let base_dir = util::config::Config::config_base_dir().context("config base dir")?;
    util::fs::remove_icon_files(&base_dir)?;
    util::fs::remove_db_files(&base_dir)?;

    let install_dir = util::config::data_local_dir()
        .context("data local dir")?
        .join("me.enigmatrix.cobalt");

    util::fs::copy_icon_files(&install_dir, &base_dir)?;
    util::fs::copy_db_files(&install_dir, &base_dir)?;

    // reinit state (repo)
    init_state(state).await?;
    Ok(())
}

#[tauri::command]
#[tracing::instrument(err, skip(state))]
pub async fn update_usages_end(state: State<'_, AppState>) -> AppResult<()> {
    let now = platform::objects::Timestamp::now();
    let state = state.read().await;
    state
        .assume_init()
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
        let state = state.read().await;
        state.assume_init().get_repo().await?
    };
    repo.update_app(&app, now).await?;
    Ok(())
}

#[tauri::command]
#[tracing::instrument(err, skip(state))]
pub async fn update_tag(state: State<'_, AppState>, tag: infused::UpdatedTag) -> AppResult<()> {
    let now = platform::objects::Timestamp::now();
    let mut repo = {
        let state = state.read().await;
        state.assume_init().get_repo().await?
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
        let state = state.read().await;
        state.assume_init().get_repo().await?
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
        let state = state.read().await;
        state.assume_init().get_repo().await?
    };
    Ok(repo.create_tag(&tag, now).await?)
}

#[tauri::command]
#[tracing::instrument(err, skip(state))]
pub async fn remove_tag(state: State<'_, AppState>, tag_id: Ref<Tag>) -> AppResult<()> {
    let mut repo = {
        let state = state.read().await;
        state.assume_init().get_repo().await?
    };
    Ok(repo.remove_tag(tag_id).await?)
}

#[tauri::command]
#[tracing::instrument(err, skip(state))]
pub async fn create_alert(
    state: State<'_, AppState>,
    alert: infused::CreateAlert,
) -> AppResult<Ref<Alert>> {
    let now = platform::objects::Timestamp::now();
    let mut repo = {
        let state = state.read().await;
        state.assume_init().get_repo().await?
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
        let state = state.read().await;
        state.assume_init().get_repo().await?
    };
    Ok(repo.update_alert(prev, next, now).await?)
}

#[tauri::command]
#[tracing::instrument(err, skip(state))]
pub async fn remove_alert(state: State<'_, AppState>, alert_id: Ref<Alert>) -> AppResult<()> {
    let mut repo = {
        let state = state.read().await;
        state.assume_init().get_repo().await?
    };
    Ok(repo.remove_alert(alert_id).await?)
}

#[tauri::command]
#[tracing::instrument(err, skip(state))]
pub async fn create_alert_event(
    state: State<'_, AppState>,
    alert_id: Ref<Alert>,
    timestamp: Timestamp,
    reason: Reason,
) -> AppResult<()> {
    let mut repo = {
        let state = state.read().await;
        state.assume_init().get_repo().await?
    };
    Ok(repo.create_alert_event(alert_id, timestamp, reason).await?)
}

#[tauri::command]
#[tracing::instrument(err, skip(state))]
pub async fn get_app_session_usages(
    state: State<'_, AppState>,
    start: Timestamp,
    end: Timestamp,
) -> AppResult<infused::AppSessionUsages> {
    let mut repo = {
        let state = state.read().await;
        state.assume_init().get_repo().await?
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
        let state = state.read().await;
        state.assume_init().get_repo().await?
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
        let state = state.read().await;
        state.assume_init().get_repo().await?
    };
    Ok(repo.get_system_events(start, end).await?)
}

#[tauri::command]
#[tracing::instrument(err, skip(state))]
pub async fn get_alert_events(
    state: State<'_, AppState>,
    start: Timestamp,
    end: Timestamp,
    alert_id: Ref<Alert>,
) -> AppResult<Vec<AlertEvent>> {
    let mut repo = {
        let state = state.read().await;
        state.assume_init().get_repo().await?
    };
    Ok(repo.get_alert_events(start, end, alert_id).await?)
}

#[tauri::command]
#[tracing::instrument(err, skip(state))]
pub async fn get_alert_reminder_events(
    state: State<'_, AppState>,
    start: Timestamp,
    end: Timestamp,
    alert_id: Ref<Alert>,
) -> AppResult<Vec<infused::ReminderEvent>> {
    let mut repo = {
        let state = state.read().await;
        state.assume_init().get_repo().await?
    };
    Ok(repo.get_alert_reminder_events(start, end, alert_id).await?)
}
