use super::model;
use rusqlite::*;
use util::{Result, *};

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new() -> Result<Database> {
        let conn =
            Connection::open_in_memory().with_context(|| "Opening connection to database")?;
        Ok(Database { conn })
    }

    pub fn insert_app(&mut self, app: &mut model::App) -> Result<()> {
        Ok(())
    }

    pub fn insert_session(&mut self, session: &mut model::Session) -> Result<()> {
        Ok(())
    }

    pub fn insert_usage(&mut self, usage: &mut model::Usage) -> Result<()> {
        Ok(())
    }

    pub fn app_by_identity(&mut self, identity: &model::AppIdentity) -> Result<Option<model::App>> {
        Ok(None)
    }
}
