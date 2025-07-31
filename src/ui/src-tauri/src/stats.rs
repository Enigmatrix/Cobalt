use data::entities::Timestamp;
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
) -> AppResult<f64> {
    let mut repo = {
        let state = state.read().await;
        state.assume_init().get_repo().await?
    };
    let res = repo.get_score(start, end).await?;
    Ok(res)
}
