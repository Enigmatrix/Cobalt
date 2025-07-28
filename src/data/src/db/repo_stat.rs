use sqlx::{FromRow, query_as};

use super::repo_crud::APP_DUR;
use super::*;
use crate::db::repo::Repository;

impl Repository {
    /// Gets the weighted average score of all apps in the given time range
    pub async fn get_score(&mut self, start: Timestamp, end: Timestamp) -> Result<f64> {
        #[derive(FromRow)]
        struct ScoreResult {
            total_score: f64,
        }

        // Get app durations and their tag scores in the given time range
        // For apps without a tag, use score 0
        // Normalize by dividing by total duration to get weighted average score
        let score_result: ScoreResult = query_as(&format!(
            "WITH
                appscore AS (
                    SELECT a.*, COALESCE(t.score, 0) AS score
                    FROM apps a
                    LEFT JOIN tags t ON a.tag_id = t.id
                ),
                appdur(id, dur) AS ({APP_DUR})
            SELECT COALESCE(
                SUM(CAST(d.dur AS REAL) * COALESCE(a.score, 0)) / COALESCE(SUM(d.dur), 1.0)
            , 0.0) AS total_score
            FROM appdur d
            INNER JOIN appscore a ON d.id = a.id"
        ))
        .bind(start)
        .bind(end)
        .fetch_one(self.db.executor())
        .await?;

        Ok(score_result.total_score)
    }
}
