use std::collections::HashMap;

use super::repo_crud::APP_DUR;
use super::*;
use crate::entities::Period;

/// Collection of methods to do large, complicated queries against apps etc.
pub struct Repository {
    pub(crate) db: Database,
}

impl Repository {
    /// Initialize a [Repository] from a given [Database]
    pub fn new(db: Database) -> Result<Self> {
        Ok(Self { db })
    }

    /// Update all [Usage]s, such that the last usage's end is equal to `last`
    pub async fn update_usages_set_last(&mut self, last: Timestamp) -> Result<()> {
        let last_usage: i64 = self
            .db
            .executor()
            .fetch_one("SELECT MAX(end) FROM usages")
            .await?
            .get(0);
        let delta = last - last_usage;
        query("UPDATE usages SET start = start + ?, end = end + ?")
            .persistent(false)
            .bind(delta)
            .bind(delta)
            .execute(self.db.executor())
            .await?;
        Ok(())
    }

    /// Gets all [App]s and their total usage duration in a start-end range.
    /// Assumes start <= end.
    pub async fn get_app_durations(
        &mut self,
        start: Timestamp,
        end: Timestamp,
    ) -> Result<HashMap<Ref<App>, infused::WithDuration<App>>> {
        // yes, we need to do this WITH expression and COALESCE
        // once more so that all apps are present in the result.

        let app_durs = query_as(&format!(
            "WITH appdur(id, dur) AS ({APP_DUR})
            SELECT a.*, COALESCE(d.dur, 0) AS duration
            FROM apps a
                LEFT JOIN appdur d ON a.id = d.id"
        ))
        .bind(start)
        .bind(end)
        .fetch_all(self.db.executor())
        .await?;

        Ok(app_durs
            .into_iter()
            .map(|app_dur: infused::WithDuration<App>| (app_dur.id.clone(), app_dur))
            .collect())
    }

    fn sql_period_start_end(expr: &str, period: &Period) -> (String, String) {
        match period {
            Period::Hour => (
                format!("unixepoch((unixepoch({expr}, 'unixepoch', 'localtime')/3600) * 3600, 'unixepoch', 'utc')"),
                format!("unixepoch((unixepoch({expr}, 'unixepoch', 'localtime')/3600) * 3600 + 3600, 'unixepoch', 'utc')"),
            ),
            Period::Day => (
                format!("unixepoch({expr}, 'unixepoch', 'localtime', 'start of day', 'utc')"),
                format!("unixepoch({expr}, 'unixepoch', 'localtime', 'start of day', '+1 day', 'utc')"),
            ),
            Period::Week => (
                format!("unixepoch({expr}, 'unixepoch', 'localtime', 'start of day', 'weekday 0', '-6 days', 'utc')"),
                format!("unixepoch({expr}, 'unixepoch', 'localtime', 'start of day', 'weekday 0', '+1 day', 'utc')"),
            ),
            Period::Month => (
                format!("unixepoch({expr}, 'unixepoch', 'localtime', 'start of month', 'utc')"),
                format!("unixepoch({expr}, 'unixepoch', 'localtime', 'start of month', '+1 month', 'utc')"),
            ),
            Period::Year => (
                format!("unixepoch({expr}, 'unixepoch', 'localtime', 'start of year', 'utc')"),
                format!("unixepoch({expr}, 'unixepoch', 'localtime', 'start of year', '+1 year', 'utc')"),
            ),
        }
    }

    fn sql_period_next(expr: &str, period: &Period) -> String {
        match period {
            Period::Hour => format!("(({expr}) + 3600)"),
            Period::Day => format!("unixepoch({expr}, 'unixepoch', '+1 day')"),
            Period::Week => format!("unixepoch({expr}, 'unixepoch', '+7 days')"),
            Period::Month => format!("unixepoch({expr}, 'unixepoch', '+1 month')"),
            Period::Year => format!("unixepoch({expr}, 'unixepoch', '+1 year')"),
        }
    }

    fn sql_ticks_to_unix(expr: &str) -> String {
        format!("((({expr}) / 10000 - 62135596800000)/1000)")
    }

    fn sql_unix_to_ticks(expr: &str) -> String {
        format!("((({expr}) * 1000 + 62135596800000)*10000)")
    }

    /// Gets all [App]s and their total usage duration in a start-end range,
    /// grouped per period. Assumes start <= end, and that start and end are
    /// aligned in multiples of period.
    pub async fn get_app_durations_per_period(
        &mut self,
        start: Timestamp,
        end: Timestamp,
        period: Period,
    ) -> Result<HashMap<Ref<App>, Vec<infused::WithGroupedDuration<App>>>> {
        let (period_start, period_end) =
            Self::sql_period_start_end(&Self::sql_ticks_to_unix("u.start"), &period);

        let period_next = Self::sql_period_next("period_end", &period);

        let period_start_ticks = Self::sql_unix_to_ticks("period_start");
        let period_end_ticks = Self::sql_unix_to_ticks("period_end");

        let query = format!("WITH RECURSIVE
            params(start, end) AS (SELECT ?, ?),
            period_intervals AS (
                SELECT a.id AS id,
                    {period_start} AS period_start,
                    {period_end} AS period_end,
                    u.start AS usage_start,
                    u.end AS usage_end
                FROM apps a, params p
                INNER JOIN sessions s ON a.id = s.app_id
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
            )

            SELECT id,
                {period_start_ticks} AS `group`,
                SUM(MIN({period_end_ticks}, usage_end, p.end) - MAX({period_start_ticks}, usage_start, p.start)) AS duration
            FROM period_intervals, params p
            GROUP BY id, period_start");

        let app_durs = query_as(&query)
            .bind(start)
            .bind(end)
            .fetch_all(self.db.executor())
            .await?;

        Ok(app_durs.into_iter().fold(
            HashMap::new(),
            |mut acc, app_dur: infused::WithGroupedDuration<App>| {
                acc.entry(app_dur.id.clone()).or_default().push(app_dur);
                acc
            },
        ))
    }

    // TODO write tests

    /// Gets all [Tags]s and their total usage duration in a start-end range,
    /// grouped per period. Assumes start <= end, and that start and end are
    /// aligned in multiples of period.
    pub async fn get_tag_durations_per_period(
        &mut self,
        start: Timestamp,
        end: Timestamp,
        period: Period,
    ) -> Result<HashMap<Ref<Tag>, Vec<infused::WithGroupedDuration<Tag>>>> {
        let (period_start, period_end) =
            Self::sql_period_start_end(&Self::sql_ticks_to_unix("u.start"), &period);

        let period_next = Self::sql_period_next("period_end", &period);

        let period_start_ticks = Self::sql_unix_to_ticks("period_start");
        let period_end_ticks = Self::sql_unix_to_ticks("period_end");

        let query = format!("WITH RECURSIVE
            params(start, end) AS (SELECT ?, ?),
            period_intervals AS (
                SELECT a.tag_id AS id,
                    {period_start} AS period_start,
                    {period_end} AS period_end,
                    u.start AS usage_start,
                    u.end AS usage_end
                FROM apps a, params p
                INNER JOIN sessions s ON a.id = s.app_id
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
            )

            SELECT id,
                {period_start_ticks} AS `group`,
                SUM(MIN({period_end_ticks}, usage_end, p.end) - MAX({period_start_ticks}, usage_start, p.start)) AS duration
            FROM period_intervals, params p
            GROUP BY id, period_start");

        let tag_durs = query_as(&query)
            .bind(start)
            .bind(end)
            .fetch_all(self.db.executor())
            .await?;

        Ok(tag_durs.into_iter().fold(
            HashMap::new(),
            |mut acc, tag_dur: infused::WithGroupedDuration<Tag>| {
                acc.entry(tag_dur.id.clone()).or_default().push(tag_dur);
                acc
            },
        ))
    }

    /// Gets all [Session]s and [Usage]s in a time range
    pub async fn get_app_session_usages(
        &mut self,
        start: Timestamp,
        end: Timestamp,
    ) -> Result<infused::AppSessionUsages> {
        #[derive(FromRow)]
        struct AppSessionUsage {
            app_id: Ref<App>,
            session_id: Ref<Session>,
            usage_id: Ref<Usage>,
            session_title: String,
            start: Timestamp,
            end: Timestamp,
        }

        let usages_raw: Vec<AppSessionUsage> = query_as(
            "SELECT
                s.app_id AS app_id,
                u.session_id AS session_id,
                u.id AS usage_id,
                s.title AS session_title,
                MAX(u.start, p.start) AS start,
                MIN(u.end, p.end) AS end
            FROM usages u, (SELECT ? AS start, ? AS end) p
            INNER JOIN sessions s ON u.session_id = s.id 
            WHERE u.end > p.start AND u.start <= p.end
            ORDER BY u.start ASC",
        )
        .bind(start)
        .bind(end)
        .fetch_all(self.db.executor())
        .await?;

        let mut usages = HashMap::<Ref<App>, HashMap<Ref<Session>, infused::Session>>::new();
        for usage in usages_raw {
            let session_id = usage.session_id.clone();
            let sessions = usages.entry(usage.app_id.clone()).or_default();
            let session = sessions
                .entry(session_id.clone())
                .or_insert_with(|| infused::Session {
                    id: session_id,
                    title: usage.session_title,
                    start: usage.start,
                    end: usage.end,
                    usages: Vec::new(),
                });
            session.start = session.start.min(usage.start);
            session.end = session.end.max(usage.end);
            session.usages.push(Usage {
                id: usage.usage_id,
                session_id: usage.session_id,
                start: usage.start,
                end: usage.end,
            });
        }

        Ok(usages)
    }
}
