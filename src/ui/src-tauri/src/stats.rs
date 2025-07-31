use data::db::infused::WithGroup;
use data::entities::{Period, Score, Timestamp};
use tauri::State;
use util::tracing;

use crate::error::AppResult;
use crate::state::{AppState, QueryOptions};

#[tauri::command]
#[tracing::instrument(err, skip(state))]
pub async fn get_score(
    state: State<'_, AppState>,
    _query_options: QueryOptions,
    start: Timestamp,
    end: Timestamp,
) -> AppResult<Score> {
    let mut repo = {
        let state = state.read().await;
        state.assume_init().get_repo().await?
    };
    let res = repo.get_score(start, end).await?;
    Ok(res)
}

#[tauri::command]
#[tracing::instrument(err, skip(state))]
pub async fn get_score_per_period(
    state: State<'_, AppState>,
    _query_options: QueryOptions,
    start: Timestamp,
    end: Timestamp,
    period: Period,
) -> AppResult<Vec<WithGroup<Score>>> {
    let mut repo = {
        let state = state.read().await;
        state.assume_init().get_repo().await?
    };
    let res = repo.get_score_per_period(start, end, period).await?;
    Ok(res)
}
