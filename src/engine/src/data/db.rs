use crate::data::entities::*;
use crate::errors::*;
use rusqlite::{params, Connection};

#[derive(Debug)]
pub struct Database {
    conn: Connection
}

impl Database {
    pub fn new() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute("
            create table Apps(
                id integer primary key autoincrement,
                name text,
                description text,
                icon blob,
                background text,
                identification_tag integer,
                identification_text1 text
            );

            create table Sessions(
                id integer primary key autoincrement,
                app_id integer references App(id),
                arguments text,
                title text
            );
        ", params![])?;
        Ok(Database { conn })
    }

    pub fn insert_app(&mut self, app: &mut App) -> Result<()> {
        let mut stmt = self.conn.prepare_cached(
            "insert into Apps
                (name, description, icon, background, \
                    identification_tag, identification_text1)
            values
                (?1, ?2, zeroblob(0), ?3, ?4, ?5)")?;
        let (app_identification_tag, app_identification_text1) = match &app.identification {
            AppIdentification::Win32{ path } => (1, path.clone()),
            AppIdentification::Uwp { aumid } => (1, aumid.clone())
        };
        app.id = stmt.insert(params![
            app.name,
            app.description,
            app.background,
            app_identification_tag,
            app_identification_text1
        ])?;
        Ok(())
    }

    pub fn insert_session(&mut self, _sess: &mut Session) -> Result<()> {
        Ok(())
    }

    pub fn insert_usage(&mut self, _usage: &mut Usage) -> Result<()> {
        Ok(())
    }

    pub fn app_by_app_identification(
        // TODO use this instead of just re-getting the app everytime!
        &mut self,
        _identification: &AppIdentification,
    ) -> Option<App> {
        None
    }
}
