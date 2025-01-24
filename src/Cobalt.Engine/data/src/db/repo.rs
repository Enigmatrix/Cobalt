use std::collections::HashMap;

use serde::Serialize;

use super::*;

/// TODO
pub struct Repository {
    db: Database,
}

#[derive(FromRow)]
struct AppTag {
    app_id: Ref<App>,
    tag_id: Ref<Tag>,
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
        #[sqlx(skip)]
        pub tags: Vec<super::Ref<super::Tag>>,
        #[sqlx(flatten)]
        usages: UsageInfo,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, FromRow)]
    pub struct Tag {
        #[sqlx(flatten)]
        #[serde(flatten)]
        pub inner: super::Tag,
        #[sqlx(skip)]
        pub apps: Vec<super::Ref<super::App>>,
        #[sqlx(flatten)]
        usages: UsageInfo,
    }
}

const APP_DUR: &str = "(SELECT 
            COALESCE(SUM(u.end - MAX(u.start, p.start)), 0) AS duration
        FROM (SELECT ? AS start) p
        INNER JOIN sessions s ON s.app_id = a.id
        INNER JOIN usages u ON u.session_id = s.id
        WHERE u.end > p.start)";

const TAG_DUR: &str = "(SELECT 
            COALESCE(SUM(u.end - MAX(u.start, p.start)), 0) AS duration
        FROM _app_tags at, (SELECT ? AS start) p
        INNER JOIN sessions s ON s.app_id = at.app_id
        INNER JOIN usages u ON u.session_id = s.id
        WHERE u.end > p.start AND at.tag_id = t.id)";

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
            "SELECT a.*,
                {APP_DUR} usage_today, {APP_DUR} usage_week, {APP_DUR} usage_month
            FROM apps a"
        ))
        .bind(ts.day_start().to_ticks() as i64)
        .bind(ts.week_start().to_ticks() as i64)
        .bind(ts.month_start().to_ticks() as i64)
        .fetch_all(self.db.executor())
        .await?;

        let mut apps: HashMap<_, _> = apps
            .into_iter()
            .map(|app| (app.inner.id.clone(), app))
            .collect();

        let app_tags: Vec<AppTag> = query_as("SELECT * FROM _app_tags")
            .fetch_all(self.db.executor())
            .await?;

        for app_tag in app_tags {
            if let Some(app) = apps.get_mut(&app_tag.app_id) {
                app.tags.push(app_tag.tag_id);
            }
            // else branch is taken rarely due to race condition e.g.
            // new app was inserted between the two statements.
        }

        Ok(apps)
    }

    /// Gets all [Tag]s from the database
    pub async fn get_tags(
        &mut self,
        ts: impl TimeSystem,
    ) -> Result<HashMap<Ref<Tag>, infused::Tag>> {
        let tags: Vec<infused::Tag> = query_as(&format!(
            "SELECT t.*,
                {TAG_DUR} usage_today, {TAG_DUR} usage_week, {TAG_DUR} usage_month
            FROM tags t"
        ))
        .bind(ts.day_start().to_ticks() as i64)
        .bind(ts.week_start().to_ticks() as i64)
        .bind(ts.month_start().to_ticks() as i64)
        .fetch_all(self.db.executor())
        .await?;

        let mut tags: HashMap<_, _> = tags
            .into_iter()
            .map(|tag| (tag.inner.id.clone(), tag))
            .collect();

        // TODO don't bother collecting this
        let app_tags: Vec<AppTag> = query_as("SELECT * FROM _app_tags")
            .fetch_all(self.db.executor())
            .await?;

        for app_tag in app_tags {
            if let Some(tag) = tags.get_mut(&app_tag.tag_id) {
                tag.apps.push(app_tag.app_id);
            }
            // else branch is taken rarely due to race condition e.g.
            // new tag was inserted between the two statements.
        }

        Ok(tags)
    }

    // TODO test these

    /// Gets all [App]s and its total usage duration in a start-end range.
    /// Assumes start <= end.
    pub async fn get_app_durations(
        &mut self,
        start: Option<Timestamp>,
        end: Option<Timestamp>,
    ) -> Result<HashMap<Ref<App>, WithDuration<App>>> {
        let start = start.unwrap_or(0);
        let end = end.unwrap_or(i64::MAX);

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
        start: Option<Timestamp>,
        end: Option<Timestamp>,
        period: crate::table::Duration,
    ) -> Result<HashMap<Ref<App>, Vec<WithGroupedDuration<App>>>> {
        let start = start.unwrap_or(0);
        let end = end.unwrap_or(i64::MAX);

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
                acc.entry(app_dur.id.clone())
                    .or_insert_with(Vec::new)
                    .push(app_dur);
                acc
            },
        ))
    }
}
