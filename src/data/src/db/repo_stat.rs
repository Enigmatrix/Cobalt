use sqlx::{FromRow, query_as};

use super::repo_crud::APP_DUR;
use super::*;
use crate::db::infused::WithGroup;
use crate::db::repo::Repository;
use crate::table::Period;

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

    /// Gets the weighted average score of all apps per period in the given time range
    pub async fn get_score_per_period(
        &mut self,
        start: Timestamp,
        end: Timestamp,
        period: Period,
    ) -> Result<Vec<WithGroup<f64>>> {
        let (period_start, period_end) =
            Self::sql_period_start_end(&Self::sql_ticks_to_unix("u.start"), &period);

        let period_next = Self::sql_period_next("period_end", &period);

        let period_start_ticks = Self::sql_unix_to_ticks("period_start");
        let period_end_ticks = Self::sql_unix_to_ticks("period_end");

        let query = format!("WITH RECURSIVE
            params(start, end) AS (SELECT ?, ?),
            period_intervals AS (
                SELECT s.app_id AS id,
                    {period_start} AS period_start,
                    {period_end} AS period_end,
                    u.start AS usage_start,
                    u.end AS usage_end
                FROM sessions s, params p
                INNER JOIN usages u ON s.id = u.session_id
                WHERE u.end > p.start AND u.start <= p.end

                UNION ALL

                SELECT id,
                    period_end AS period_start,
                    {period_next} AS period_end,
                    usage_start,
                    usage_end
                FROM period_intervals, params p
                WHERE {period_end_ticks} < MIN(usage_end, p.end)
            ),
            appscore AS (
                SELECT a.*, COALESCE(t.score, 0) AS score
                FROM apps a
                LEFT JOIN tags t ON a.tag_id = t.id
            )

            SELECT 
                {period_start_ticks} AS `group`,
                COALESCE(
                    SUM(CAST(MIN({period_end_ticks}, usage_end, p.end) - MAX({period_start_ticks}, usage_start, p.start) AS REAL) * COALESCE(a.score, 0)) / 
                    COALESCE(SUM(MIN({period_end_ticks}, usage_end, p.end) - MAX({period_start_ticks}, usage_start, p.start)), 1.0)
                , 0.0) AS value
            FROM period_intervals, params p
            INNER JOIN appscore a ON period_intervals.id = a.id
            WHERE {period_start_ticks} BETWEEN p.start AND p.end
            GROUP BY period_start
           ");

        let score_results: Vec<WithGroup<f64>> = query_as(&query)
            .bind(start)
            .bind(end)
            .fetch_all(self.db.executor())
            .await?;

        Ok(score_results)
    }
}
