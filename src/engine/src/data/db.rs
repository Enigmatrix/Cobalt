use super::model;
use anyhow::{Result, *};
use rusqlite::*;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new() -> Result<Database> {
        let conn =
            Connection::open_in_memory().with_context(|| "Opening connection to database")?;
        Ok(Database { conn })
    }

    pub fn insert_app(&mut self, mut app: model::App) -> Result<model::App> {
        Ok(app)
    }

    pub fn insert_session(&mut self, session: model::Session) -> Result<model::Session> {
        Ok(session)
    }

    pub fn insert_usage(&mut self, usage: model::Usage) -> Result<model::Usage> {
        Ok(usage)
    }

    pub fn app_by_identity(&mut self, identity: &model::AppIdentity) -> Result<Option<model::App>> {
        Ok(None)
    }
}
