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

    #[derive(Debug, Clone, PartialEq, Eq, Serialize)]
    pub struct App {
        #[serde(flatten)]
        pub inner: super::App,
        pub tags: Vec<super::Ref<super::Tag>>,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize)]
    pub struct Tag {
        #[serde(flatten)]
        pub inner: super::Tag,
        pub apps: Vec<super::Ref<super::App>>,
    }
}

impl Repository {
    /// Initialize a [Repository] from a given [Database]
    pub fn new(db: Database) -> Result<Self> {
        Ok(Self { db })
    }

    /// Gets all [App]s from the database
    pub async fn get_apps(&mut self) -> Result<HashMap<Ref<App>, infused::App>> {
        // 1+N (well, 1+1) query pattern here - introduces a little gap for
        // race condition but doesn't matter much. This query pattern doesn't
        // significantly affect the performance of the application for SQLite
        // compared to other DB formats.

        // TODO should we filter by `initialized`?
        let apps: Vec<App> = query_as("SELECT * FROM apps")
            .fetch_all(self.db.executor())
            .await?;

        let mut apps: HashMap<_, _> = apps
            .into_iter()
            .map(|app| {
                (
                    app.id.clone(),
                    infused::App {
                        inner: app,
                        tags: Vec::new(),
                    },
                )
            })
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
    pub async fn get_tags(&mut self) -> Result<HashMap<Ref<Tag>, infused::Tag>> {
        let tags: Vec<Tag> = query_as("SELECT * FROM tags")
            .fetch_all(self.db.executor())
            .await?;

        let mut tags: HashMap<_, _> = tags
            .into_iter()
            .map(|tag| {
                (
                    tag.id.clone(),
                    infused::Tag {
                        inner: tag,
                        apps: Vec::new(),
                    },
                )
            })
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
    ) -> Result<Vec<WithDuration<App>>> {
        let start = start.unwrap_or(0);
        let end = end.unwrap_or(i64::MAX);

        let app_durs = query_as(
            "SELECT a.id AS id,
                COALESCE(SUM(MIN(u.end, ?) - MAX(u.start, ?)), 0) AS duration
            FROM apps a
            INNER JOIN sessions s ON a.id = s.app_id
            INNER JOIN usages u ON s.id = u.session_id
            WHERE u.end > ? AND u.start <= ?
            GROUP BY a.id",
        )
        // TODO named bindings ..?
        .bind(end)
        .bind(start)
        .bind(start)
        .bind(end)
        .fetch_all(self.db.executor())
        .await?;

        Ok(app_durs)
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
    ) -> Result<Vec<WithGroupedDuration<App>>> {
        let start = start.unwrap_or(0);
        let end = end.unwrap_or(i64::MAX);

        let app_durs = query_as(
            "SELECT a.id AS id,
                CAST(u.start / ? AS INT) * ? AS `group`,
                COALESCE(
                    SUM(MIN(u.end, (CAST(u.start / ? AS INT) + 1) * ?)
                        - MAX(u.start, ?)), 0) AS duration
            FROM apps a
            INNER JOIN sessions s ON a.id = s.app_id
            INNER JOIN usages u ON s.id = u.session_id
            WHERE u.end > ? AND u.start <= ?
            GROUP BY CAST(u.start / ? AS INT), a.id",
        )
        // TODO named bindings ..?
        .bind(period)
        .bind(period)
        .bind(period)
        .bind(period)
        .bind(start)
        .bind(start)
        .bind(end)
        .bind(period)
        .fetch_all(self.db.executor())
        .await?;

        Ok(app_durs)
    }
}
