use sqlx::{FromRow, query_as};

use super::repo_crud::APP_DUR;
use super::*;
use crate::db::infused::WithGroup;
use crate::db::repo::Repository;
use crate::table::{Duration, Period, Score};

impl Repository {
    /// Gets the weighted average score of all apps in the given time range
    pub async fn get_score(&mut self, start: Timestamp, end: Timestamp) -> Result<Score> {
        #[derive(FromRow)]
        struct ScoreResult {
            total_score: Score,
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
    ) -> Result<Vec<WithGroup<Score>>> {
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

        let score_results: Vec<WithGroup<Score>> = query_as(&query)
            .bind(start)
            .bind(end)
            .fetch_all(self.db.executor())
            .await?;

        Ok(score_results)
    }
}

/// Represents a period of time that is either focused or distractive.
#[derive(FromRow, Debug, PartialEq)]
pub struct FocusPeriod {
    /// The start time of the period.
    pub start: Timestamp,
    /// The end time of the period.
    pub end: Timestamp,
    /// Whether the period is a focused period. If false, it is a distractive period.
    pub is_focused: bool,
}

/// Settings for focus periods.
#[derive(FromRow, Debug, PartialEq)]
pub struct FocusPeriodSettings {
    /// The minimum score of a focused app.
    pub min_focus_score: i64,
    /// The minimum duration of a focused usage.
    pub min_focus_usage_dur: Duration,
    /// The maximum gap between two focused periods.
    pub max_focus_gap: Duration,
}

/// Settings for distractive periods.
#[derive(FromRow, Debug, PartialEq)]
pub struct DistractivePeriodSettings {
    /// The maximum score of a distractive app.
    pub max_distractive_score: i64,
    /// The minimum duration of a distractive usage.
    pub min_distractive_usage_dur: Duration,
    /// The maximum gap between two distractive periods.
    pub max_distractive_gap: Duration,
}

impl Repository {
    /// Gets all distractive and focused periods in the given time range, using the specified parameters.
    ///
    /// The algorithm is described in detail in `docs/get_periods_algorithm.md`.
    pub async fn get_periods(
        &mut self,
        start: Timestamp,
        end: Timestamp,
        focus_settings: FocusPeriodSettings,
        distractive_settings: DistractivePeriodSettings,
    ) -> Result<Vec<FocusPeriod>> {
        let query = "
        WITH
            -- Parameters
            params(start, end, min_focus_score, max_distractive_score, min_focus_usage_dur, min_distractive_usage_dur, max_focus_gap, max_distractive_gap) AS (
                SELECT ?, ?, ?, ?, ?, ?, ?, ?
            ),
            -- App scores
            app_scores AS (
                SELECT a.id, COALESCE(t.score, 0) AS score
                FROM apps a
                LEFT JOIN tags t ON a.tag_id = t.id
            ),
            -- Usages within the time range
            usages_in_range AS (
                SELECT u.start, u.end, s.app_id
                FROM usages u
                JOIN sessions s ON u.session_id = s.id
                CROSS JOIN params p
                WHERE u.end > p.start AND u.start < p.end
            ),
            -------------------------------------------------------------
            -- Distractive side
            -------------------------------------------------------------
            distractive_usages AS (
                SELECT u.start, u.end
                FROM usages_in_range u
                JOIN app_scores a ON u.app_id = a.id
                CROSS JOIN params p
                WHERE a.score < p.max_distractive_score
                  AND (u.end - u.start) >= p.min_distractive_usage_dur
            ),
            distractive_ordered AS (
                SELECT start,
                       end,
                       LAG(end) OVER (ORDER BY start) AS prev_end
                FROM distractive_usages
            ),
            distractive_flag AS (
                SELECT d.start,
                       d.end,
                       CASE
                         WHEN d.prev_end IS NULL
                              OR d.start - d.prev_end > (SELECT max_distractive_gap FROM params)
                         THEN 1 ELSE 0 END AS new_group
                FROM distractive_ordered d
            ),
            distractive_grouped AS (
                SELECT start,
                       end,
                       SUM(new_group) OVER (ORDER BY start) AS grp
                FROM distractive_flag
            ),
            distractive_periods AS (
                SELECT MIN(start) AS start, MAX(end) AS end
                FROM distractive_grouped
                GROUP BY grp
            ),
            -------------------------------------------------------------
            -- Focus side
            -------------------------------------------------------------
            focus_usages AS (
                SELECT u.start, u.end
                FROM usages_in_range u
                JOIN app_scores a ON u.app_id = a.id
                CROSS JOIN params p
                WHERE a.score > p.min_focus_score
                  AND (u.end - u.start) >= p.min_focus_usage_dur
            ),
            focus_usages_pruned AS (
                SELECT f.start, f.end
                FROM focus_usages f
                WHERE NOT EXISTS (
                    SELECT 1 FROM distractive_periods dp
                    WHERE f.start < dp.end AND f.end > dp.start
                )
            ),
            focus_ordered AS (
                SELECT start,
                       end,
                       LAG(end) OVER (ORDER BY start) AS prev_end
                FROM focus_usages_pruned
            ),
            focus_flag AS (
                SELECT fo.start,
                       fo.end,
                       CASE
                         WHEN fo.prev_end IS NULL
                              OR fo.start - fo.prev_end > (SELECT max_focus_gap FROM params)
                              OR EXISTS (
                                   SELECT 1 FROM distractive_periods dp
                                   WHERE dp.start < fo.start AND dp.end > fo.prev_end
                              )
                         THEN 1 ELSE 0 END AS new_group
                FROM focus_ordered fo
            ),
            focus_grouped AS (
                SELECT start,
                       end,
                       SUM(new_group) OVER (ORDER BY start) AS grp
                FROM focus_flag
            ),
            focus_periods AS (
                SELECT MIN(start) AS start, MAX(end) AS end
                FROM focus_grouped
                GROUP BY grp
            )
        -- Final output: union of focus and distractive periods
        SELECT
            MAX(start, (SELECT start FROM params)) as start,
            MIN(end, (SELECT end FROM params)) as end,
            1 as is_focused
        FROM focus_periods
        WHERE start < end  -- Ensure valid periods
        
        UNION ALL
        
        SELECT
            MAX(start, (SELECT start FROM params)) as start,
            MIN(end, (SELECT end FROM params)) as end,
            0 as is_focused
        FROM distractive_periods
        WHERE start < end  -- Ensure valid periods
        
        ORDER BY start;";

        let periods: Vec<FocusPeriod> = query_as(query)
            .bind(start)
            .bind(end)
            .bind(focus_settings.min_focus_score)
            .bind(distractive_settings.max_distractive_score)
            .bind(focus_settings.min_focus_usage_dur)
            .bind(distractive_settings.min_distractive_usage_dur)
            .bind(focus_settings.max_focus_gap)
            .bind(distractive_settings.max_distractive_gap)
            .fetch_all(self.db.executor())
            .await?;

        Ok(periods)
    }
}
