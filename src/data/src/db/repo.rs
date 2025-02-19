use std::collections::HashMap;
use std::ops::Deref;
use std::str::FromStr;

use infused::AppSessionUsages;
use serde::Serialize;

use super::*;

/// Collection of methods to do large, complicated queries against apps etc.
pub struct Repository {
    pub(crate) db: Database,
}

/// Duration grouped into target
#[derive(Clone, Debug, Default, PartialEq, Eq, FromRow, Serialize)]
pub struct WithDuration<T: Table> {
    /// Target Identifier
    pub id: Ref<T>,
    /// Duration value
    pub duration: crate::table::Duration,
}

/// Duration grouped into target, period chunks
#[derive(Clone, Debug, Default, PartialEq, Eq, FromRow, Serialize)]
pub struct WithGroupedDuration<T: Table> {
    /// Target Identifier
    pub id: Ref<T>,
    /// Time Period group
    pub group: crate::table::Timestamp,
    /// Duration value
    pub duration: crate::table::Duration,
}

/// List of [Ref<T>]
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct RefVec<T: Table>(pub Vec<Ref<T>>);

impl<T: Table> Deref for RefVec<T> {
    type Target = Vec<Ref<T>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'r, T: Table<Id: FromStr<Err: std::fmt::Debug>>> sqlx::Decode<'r, Sqlite> for RefVec<T> {
    fn decode(
        value: <Sqlite as sqlx::Database>::ValueRef<'r>,
    ) -> std::result::Result<Self, sqlx::error::BoxDynError> {
        let str = <String as sqlx::Decode<'r, Sqlite>>::decode(value)?;
        if str.is_empty() {
            return Ok(Self(Vec::new()));
        }

        let inner = str
            .split(',')
            .map(|id| Ref::new(id.parse().unwrap()))
            .collect();

        Ok(Self(inner))
    }
}

impl<T: Table> sqlx::Type<Sqlite> for RefVec<T> {
    fn type_info() -> <Sqlite as sqlx::Database>::TypeInfo {
        <String as sqlx::Type<Sqlite>>::type_info()
    }

    fn compatible(ty: &<Sqlite as sqlx::Database>::TypeInfo) -> bool {
        <String as sqlx::Type<Sqlite>>::compatible(ty)
    }
}

/// Entities with extra information embedded.
pub mod infused {
    use serde::Deserialize;

    use super::*;
    use crate::table::Color;

    /// Usages of the target
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, FromRow)]
    pub struct UsageInfo {
        /// Usage today
        pub usage_today: crate::table::Duration,
        /// Usage this week
        pub usage_week: crate::table::Duration,
        /// Usage this month
        pub usage_month: crate::table::Duration,
    }

    /// Options to update a [super::App]
    #[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
    pub struct UpdatedApp {
        /// Identifier
        pub id: Ref<super::App>,
        /// Name
        pub name: String,
        /// Description
        pub description: String,
        /// Company
        pub company: String,
        /// Color
        pub color: Color,
        /// Linked [super::Tag]
        pub tag_id: Option<Ref<super::Tag>>,
    }

    /// [super::App] with additional information
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, FromRow)]
    pub struct App {
        #[sqlx(flatten)]
        #[serde(flatten)]
        /// [super::App] itself
        pub inner: super::App,
        /// List of linked [super::App]s
        #[sqlx(flatten)]
        /// Usage Info
        usages: UsageInfo,
    }

    /// [super::Tag] with additional information
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, FromRow)]
    pub struct Tag {
        #[sqlx(flatten)]
        #[serde(flatten)]
        /// [super::Tag] itself
        pub inner: super::Tag,
        /// List of linked [super::App]s
        pub apps: RefVec<super::App>,
        #[sqlx(flatten)]
        /// Usage Info
        usages: UsageInfo,
    }

    /// Options to create a new [super::Tag]
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct CreateTag {
        /// Name
        pub name: String,
        /// Color
        pub color: String,
    }

    /// Options to update a [super::Tag]
    #[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
    pub struct UpdatedTag {
        /// Identifier
        pub id: Ref<super::Tag>,
        /// Name
        pub name: String,
        /// Color
        pub color: String,
    }

    /// [super::Session] with additional information
    #[derive(Debug, Clone, PartialEq, Eq, Serialize)]
    pub struct Session {
        /// Identifier
        pub id: Ref<super::Session>,
        /// Title of Session
        pub title: String,
        /// Minimum Usage of Usages
        pub start: Timestamp,
        /// Maximum Usage of Usages
        pub end: Timestamp,
        /// Usages
        pub usages: Vec<super::Usage>,
    }

    /// [Session]s with [Usage]s, partitioned by [App]s
    pub type AppSessionUsages = HashMap<Ref<super::App>, HashMap<Ref<super::Session>, Session>>;
}

const APP_DUR: &str = "SELECT a.id AS id,
                COALESCE(SUM(MIN(u.end, p.end) - MAX(u.start, p.start)), 0) AS duration
            FROM apps a, (SELECT ? AS start, ? AS end) p
            INNER JOIN sessions s ON a.id = s.app_id
            INNER JOIN usages u ON s.id = u.session_id
            WHERE u.end > p.start AND u.start <= p.end
            GROUP BY a.id";

const TAG_DUR: &str = "SELECT a.tag_id AS id,
                COALESCE(SUM(MIN(u.end, p.end) - MAX(u.start, p.start)), 0) AS duration
            FROM apps a, (SELECT ? AS start, ? AS end) p
            INNER JOIN sessions s ON a.id = s.app_id
            INNER JOIN usages u ON s.id = u.session_id
            WHERE u.end > p.start AND u.start <= p.end
            GROUP BY a.tag_id";

// TODO when we sqlx has named parameters, fixup our queries to use them.

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

    /// Gets all [App]s from the database
    pub async fn get_apps(
        &mut self,
        ts: impl TimeSystem,
    ) -> Result<HashMap<Ref<App>, infused::App>> {
        // 1+N (well, 1+1) query pattern here - introduces a little gap for
        // race condition but doesn't matter much. This query pattern doesn't
        // significantly affect the performance of the application for SQLite
        // compared to other DB formats.

        let apps: Vec<infused::App> = query_as(&format!(
            "WITH
                usage_daily(id, dur) AS ({APP_DUR}),
                usage_week(id, dur) AS ({APP_DUR}),
                usage_month(id, dur) AS ({APP_DUR})
            SELECT a.*,
                COALESCE(d.dur, 0) AS usage_today,
                COALESCE(w.dur, 0) AS usage_week,
                COALESCE(m.dur, 0) AS usage_month
            FROM apps a
                LEFT JOIN usage_daily d ON a.id = d.id
                LEFT JOIN usage_week  w ON a.id = w.id
                LEFT JOIN usage_month m ON a.id = m.id
            WHERE a.initialized = 1
            GROUP BY a.id"
        ))
        .bind(ts.day_start(true).to_ticks())
        .bind(i64::MAX)
        .bind(ts.week_start(true).to_ticks())
        .bind(i64::MAX)
        .bind(ts.month_start(true).to_ticks())
        .bind(i64::MAX)
        .fetch_all(self.db.executor())
        .await?;

        let apps: HashMap<_, _> = apps
            .into_iter()
            .map(|app| (app.inner.id.clone(), app))
            .collect();

        Ok(apps)
    }

    /// Gets all [Tag]s from the database
    pub async fn get_tags(
        &mut self,
        ts: impl TimeSystem,
    ) -> Result<HashMap<Ref<Tag>, infused::Tag>> {
        let tags: Vec<infused::Tag> = query_as(&format!(
            "WITH
                usage_daily(id, dur) AS ({TAG_DUR}),
                usage_week(id, dur) AS ({TAG_DUR}),
                usage_month(id, dur) AS ({TAG_DUR})
            SELECT t.*, GROUP_CONCAT(a.id, ',') apps,
                COALESCE(d.dur, 0) AS usage_today,
                COALESCE(w.dur, 0) AS usage_week,
                COALESCE(m.dur, 0) AS usage_month
            FROM tags t
                LEFT JOIN usage_daily d ON t.id = d.id
                LEFT JOIN usage_week  w ON t.id = w.id
                LEFT JOIN usage_month m ON t.id = m.id
                LEFT JOIN apps a ON t.id = a.tag_id AND a.initialized = 1
            GROUP BY t.id"
        ))
        .bind(ts.day_start(true).to_ticks())
        .bind(i64::MAX)
        .bind(ts.week_start(true).to_ticks())
        .bind(i64::MAX)
        .bind(ts.month_start(true).to_ticks())
        .bind(i64::MAX)
        .fetch_all(self.db.executor())
        .await?;

        let tags: HashMap<_, _> = tags
            .into_iter()
            .map(|tag| (tag.inner.id.clone(), tag))
            .collect();

        Ok(tags)
    }

    /// Gets all [App]s and its total usage duration in a start-end range.
    /// Assumes start <= end.
    pub async fn get_app_durations(
        &mut self,
        start: Timestamp,
        end: Timestamp,
    ) -> Result<HashMap<Ref<App>, WithDuration<App>>> {
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
            .map(|app_dur: WithDuration<App>| (app_dur.id.clone(), app_dur))
            .collect())
    }

    /// Gets all [App]s and its total usage duration in a start-end range,
    /// grouped per period. Assumes start <= end, and that start and end are
    /// aligned in multiples of period.
    pub async fn get_app_durations_per_period(
        &mut self,
        start: Timestamp,
        end: Timestamp,
        period: crate::table::Duration,
    ) -> Result<HashMap<Ref<App>, Vec<WithGroupedDuration<App>>>> {
        // This expression is surprisingly slow compared to its previous
        // iteration (tho more correct ofc). Fix it later.
        let app_durs = query_as(
            "WITH RECURSIVE
            params(period, start, end) AS (SELECT ?, ?, ?),
            period_intervals AS (
                SELECT a.id AS id,
                    p.start + (p.period * CAST((u.start - p.start) / p.period AS INT)) AS period_start,
                    p.start + (p.period * (CAST((u.start - p.start) / p.period AS INT) + 1)) AS period_end,
                    u.start AS usage_start,
                    MIN(u.end, p.end) AS usage_end
                FROM apps a, params p
                INNER JOIN sessions s ON a.id = s.app_id
                INNER JOIN usages u ON s.id = u.session_id
                WHERE u.end > p.start AND u.start <= p.end

                UNION ALL

                SELECT id,
                    period_start + p.period AS period_start,
                    period_end + p.period AS period_end,
                    usage_start,
                    usage_end
                FROM period_intervals, params p
                WHERE period_start + p.period < MIN(usage_end, p.end)
            )

            SELECT id,
                period_start AS `group`,
                SUM(MIN(period_end, usage_end) - MAX(period_start, usage_start)) AS duration
            FROM period_intervals
            GROUP BY id, period_start;",
        )
        .bind(period)
        .bind(start)
        .bind(end)
        .fetch_all(self.db.executor())
        .await?;

        Ok(app_durs.into_iter().fold(
            HashMap::new(),
            |mut acc, app_dur: WithGroupedDuration<App>| {
                acc.entry(app_dur.id.clone()).or_default().push(app_dur);
                acc
            },
        ))
    }

    // TODO write tests

    /// Gets all [Tags]s and its total usage duration in a start-end range,
    /// grouped per period. Assumes start <= end, and that start and end are
    /// aligned in multiples of period.
    pub async fn get_tag_durations_per_period(
        &mut self,
        start: Timestamp,
        end: Timestamp,
        period: crate::table::Duration,
    ) -> Result<HashMap<Ref<Tag>, Vec<WithGroupedDuration<Tag>>>> {
        // This expression is surprisingly slow compared to its previous
        // iteration (tho more correct ofc). Fix it later.
        let app_durs = query_as(
            "WITH RECURSIVE
            params(period, start, end) AS (SELECT ?, ?, ?),
            period_intervals AS (
                SELECT a.tag_id AS id,
                    p.start + (p.period * CAST((u.start - p.start) / p.period AS INT)) AS period_start,
                    p.start + (p.period * (CAST((u.start - p.start) / p.period AS INT) + 1)) AS period_end,
                    u.start AS usage_start,
                    MIN(u.end, p.end) AS usage_end
                FROM apps a, params p
                INNER JOIN sessions s ON a.id = s.app_id
                INNER JOIN usages u ON s.id = u.session_id
                WHERE u.end > p.start AND u.start <= p.end

                UNION ALL

                SELECT id,
                    period_start + p.period AS period_start,
                    period_end + p.period AS period_end,
                    usage_start,
                    usage_end
                FROM period_intervals, params p
                WHERE period_start + p.period < MIN(usage_end, p.end)
            )

            SELECT id,
                period_start AS `group`,
                SUM(MIN(period_end, usage_end) - MAX(period_start, usage_start)) AS duration
            FROM period_intervals
            GROUP BY id, period_start;",
        )
        .bind(period)
        .bind(start)
        .bind(end)
        .fetch_all(self.db.executor())
        .await?;

        Ok(app_durs.into_iter().fold(
            HashMap::new(),
            |mut acc, tag_dur: WithGroupedDuration<Tag>| {
                acc.entry(tag_dur.id.clone()).or_default().push(tag_dur);
                acc
            },
        ))
    }

    /// Update the [App] with additional information
    pub async fn update_app(&mut self, app: &infused::UpdatedApp) -> Result<()> {
        query(
            "UPDATE apps SET
                    name = ?,
                    description = ?,
                    company = ?,
                    color = ?,
                    tag_id = ?,
                    initialized = 1
                WHERE id = ?",
        )
        .bind(&app.name)
        .bind(&app.description)
        .bind(&app.company)
        .bind(&app.color)
        .bind(&app.tag_id)
        .bind(&app.id)
        .execute(self.db.executor())
        .await?;
        Ok(())
    }

    /// Update the [Tag] with additional information
    pub async fn update_tag(&mut self, tag: &infused::UpdatedTag) -> Result<()> {
        query(
            "UPDATE tags SET
                    name = ?,
                    color = ?
                WHERE id = ?",
        )
        .bind(&tag.name)
        .bind(&tag.color)
        .bind(&tag.id)
        .execute(self.db.executor())
        .await?;
        Ok(())
    }

    /// Update old and new [App]s with [Tag] atomically
    pub async fn update_tag_apps(
        &mut self,
        tag_id: Ref<Tag>,
        removed_apps: Vec<Ref<App>>,
        added_apps: Vec<Ref<App>>,
    ) -> Result<()> {
        let mut tx = self.db.transaction().await?;
        let updates = removed_apps.into_iter().map(|app| (app, None)).chain(
            added_apps
                .into_iter()
                .map(|app| (app, Some(tag_id.clone()))),
        );
        for (app, tag) in updates {
            query("UPDATE apps SET tag_id = ? WHERE id = ?")
                .bind(tag)
                .bind(app)
                .execute(&mut *tx)
                .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    /// Create a new [Tag] from a [infused::CreateTag]
    pub async fn create_tag(&mut self, tag: &infused::CreateTag) -> Result<Ref<Tag>> {
        let res = query("INSERT INTO tags VALUES (NULL, ?, ?)")
            .bind(&tag.name)
            .bind(&tag.color)
            .execute(self.db.executor())
            .await?;
        Ok(Ref::new(res.last_insert_rowid()))
    }

    /// Removes a [Tag]
    pub async fn remove_tag(&mut self, tag_id: Ref<Tag>) -> Result<()> {
        query("DELETE FROM tags WHERE id = ?")
            .bind(tag_id)
            .execute(self.db.executor())
            .await?;
        Ok(())
    }

    /// Gets all [Session]s and [Usage]s in a time range
    pub async fn app_session_usages(
        &mut self,
        start: Timestamp,
        end: Timestamp,
    ) -> Result<AppSessionUsages> {
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
