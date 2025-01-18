use std::collections::HashMap;

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

// Entities with extra information embedded.
mod infused {
    pub struct App {
        pub inner: super::App,
        pub tags: Vec<super::Ref<super::Tag>>,
    }

    pub struct Tag {
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
}
