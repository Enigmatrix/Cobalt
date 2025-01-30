use std::collections::HashMap;
use std::ops::Deref;
use std::str::FromStr;

use serde::Serialize;

use super::*;

/// TODO
pub struct Repository {
    db: Database,
}

#[derive(FromRow, Serialize)]
pub struct WithDuration<T: Table> {
    id: Ref<T>,
    duration: crate::table::Duration,
}

#[derive(FromRow, Serialize)]
pub struct WithGroupedDuration<T: Table> {
    id: Ref<T>,
    group: crate::table::Timestamp,
    duration: crate::table::Duration,
}

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

// Entities with extra information embedded.
pub mod infused {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, FromRow)]
    pub struct UsageInfo {
        pub usage_today: crate::table::Duration,
        pub usage_week: crate::table::Duration,
        pub usage_month: crate::table::Duration,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, FromRow)]
    pub struct App {
        #[sqlx(flatten)]
        #[serde(flatten)]
        pub inner: super::App,
        pub tags: RefVec<super::Tag>,
        #[sqlx(flatten)]
        usages: UsageInfo,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, FromRow)]
    pub struct Tag {
        #[sqlx(flatten)]
        #[serde(flatten)]
        pub inner: super::Tag,
        pub apps: RefVec<super::App>,
        #[sqlx(flatten)]
        usages: UsageInfo,
    }
}

const APP_DUR: &str = "SELECT a.id AS id,
                COALESCE(SUM(MIN(u.end, p.end) - MAX(u.start, p.start)), 0) AS duration
            FROM apps a, (SELECT ? AS start, ? AS end) p
            INNER JOIN sessions s ON a.id = s.app_id
            INNER JOIN usages u ON s.id = u.session_id
            WHERE u.end > p.start AND u.start <= p.end
            GROUP BY a.id";

const TAG_DUR: &str = "SELECT at.tag_id AS id,
                COALESCE(SUM(MIN(u.end, p.end) - MAX(u.start, p.start)), 0) AS duration
            FROM _app_tags at, (SELECT ? AS start, ? AS end) p
            INNER JOIN apps a ON a.id = at.app_id
            INNER JOIN sessions s ON a.id = s.app_id
            INNER JOIN usages u ON s.id = u.session_id
            WHERE u.end > p.start AND u.start <= p.end
            GROUP BY at.tag_id";

impl Repository {
    /// Initialize a [Repository] from a given [Database]
    pub fn new(db: Database) -> Result<Self> {
        Ok(Self { db })
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

        // TODO should we filter by `initialized`?
        let apps: Vec<infused::App> = query_as(&format!(
            "WITH
                usage_daily(id, dur) AS ({APP_DUR}),
                usage_week(id, dur) AS ({APP_DUR}),
                usage_month(id, dur) AS ({APP_DUR})
            SELECT a.*, GROUP_CONCAT(at.tag_id, ',') tags,
                COALESCE(d.dur, 0) AS usage_today,
                COALESCE(w.dur, 0) AS usage_week,
                COALESCE(m.dur, 0) AS usage_month
            FROM apps a
                LEFT JOIN usage_daily d ON a.id = d.id
                LEFT JOIN usage_week  w ON a.id = w.id
                LEFT JOIN usage_month m ON a.id = m.id
                LEFT JOIN _app_tags at ON a.id = at.app_id
            GROUP BY a.id"
        ))
        .bind(ts.day_start().to_ticks() as i64)
        .bind(i64::MAX)
        .bind(ts.week_start().to_ticks() as i64)
        .bind(i64::MAX)
        .bind(ts.month_start().to_ticks() as i64)
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
            SELECT t.*, GROUP_CONCAT(at.app_id, ',') apps,
                COALESCE(d.dur, 0) AS usage_today,
                COALESCE(w.dur, 0) AS usage_week,
                COALESCE(m.dur, 0) AS usage_month
            FROM tags t
                LEFT JOIN usage_daily d ON t.id = d.id
                LEFT JOIN usage_week  w ON t.id = w.id
                LEFT JOIN usage_month m ON t.id = m.id
                LEFT JOIN _app_tags at ON t.id = at.tag_id
            GROUP BY t.id"
        ))
        .bind(ts.day_start().to_ticks() as i64)
        .bind(i64::MAX)
        .bind(ts.week_start().to_ticks() as i64)
        .bind(i64::MAX)
        .bind(ts.month_start().to_ticks() as i64)
        .bind(i64::MAX)
        .fetch_all(self.db.executor())
        .await?;

        let tags: HashMap<_, _> = tags
            .into_iter()
            .map(|tag| (tag.inner.id.clone(), tag))
            .collect();

        Ok(tags)
    }

    // TODO test these

    /// Gets all [App]s and its total usage duration in a start-end range.
    /// Assumes start <= end.
    pub async fn get_app_durations(
        &mut self,
        start: Timestamp,
        end: Timestamp,
    ) -> Result<HashMap<Ref<App>, WithDuration<App>>> {
        // TODO actual named bindings please!
        let app_durs = query_as(
            "SELECT a.id AS id,
                COALESCE(SUM(MIN(u.end, p.end) - MAX(u.start, p.start)), 0) AS duration
            FROM apps a, (SELECT ? AS start, ? AS end) p
            INNER JOIN sessions s ON a.id = s.app_id
            INNER JOIN usages u ON s.id = u.session_id
            WHERE u.end > p.start AND u.start <= p.end
            GROUP BY a.id",
        )
        .bind(start)
        .bind(end)
        .fetch_all(self.db.executor())
        .await?;

        Ok(app_durs
            .into_iter()
            .map(|app_dur: WithDuration<App>| (app_dur.id.clone(), app_dur))
            .collect())
    }

    // TODO test these

    /// Gets all [App]s and its total usage duration in a start-end range,
    /// grouped per period. Assumes start <= end, and that start and end are
    /// aligned in multiples of period.
    pub async fn get_app_durations_per_period(
        &mut self,
        start: Timestamp,
        end: Timestamp,
        period: crate::table::Duration,
    ) -> Result<HashMap<Ref<App>, Vec<WithGroupedDuration<App>>>> {
        // TODO actual named bindings please!
        let app_durs = query_as(
            "SELECT a.id AS id,
                CAST(u.start / p.period AS INT) * p.period AS `group`,
                COALESCE(
                    SUM(MIN(u.end, (CAST(u.start / p.period AS INT) + 1) * p.period)
                        - MAX(u.start, p.start)), 0) AS duration
            FROM apps a, (SELECT ? AS period, ? AS start, ? AS end) p
            INNER JOIN sessions s ON a.id = s.app_id
            INNER JOIN usages u ON s.id = u.session_id
            WHERE u.end > p.start AND u.start <= p.end
            GROUP BY CAST(u.start / p.period AS INT), a.id",
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
}
