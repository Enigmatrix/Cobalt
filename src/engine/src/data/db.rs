use crate::data::entities::*;
use crate::errors::*;
use rusqlite::{params, Connection, OptionalExtension};

#[derive(Debug)]
pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        // TODO for testing!
        conn.execute_batch(
            "
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
                app_id integer references Apps(id),
                arguments text,
                title text
            );

            create table Usages(
                id integer primary key autoincrement,
                start integer,
                end integer,
                during_idle integer,
                session_id integer references Sessions(id)
            );
        ",
        )?;
        Ok(Database { conn })
    }

    fn expand_identification(ident: &AppIdentification) -> (i64, String) {
        match ident {
            AppIdentification::Win32 { path } => (0, path.clone()),
            AppIdentification::Uwp { aumid } => (1, aumid.clone()),
        }
    }

    pub fn insert_app(&mut self, app: &mut App) -> Result<()> {
        let mut stmt = self.conn.prepare_cached(
            "insert into Apps
                (name, description, icon, background, \
                    identification_tag, identification_text1)
            values
                (?1, ?2, zeroblob(0), ?3, ?4, ?5)",
        )?;
        let (app_identification_tag, app_identification_text1) =
            Database::expand_identification(&app.identification);
        app.id = stmt.insert(params![
            app.name,
            app.description,
            app.background,
            app_identification_tag,
            app_identification_text1
        ])?;
        Ok(())
    }

    pub fn insert_session(&mut self, sess: &mut Session) -> Result<()> {
        let mut stmt = self.conn.prepare_cached(
            "insert into Sessions
                (app_id, arguments, title)
            values
                (?1, ?2, ?3)",
        )?;
        sess.id = stmt.insert(params![sess.app_id, sess.arguments, sess.title])?;
        Ok(())
    }

    pub fn insert_usage(&mut self, usage: &mut Usage) -> Result<()> {
        let mut stmt = self.conn.prepare_cached(
            "insert into Usages
                (start, end, during_idle, session_id)
            values
                (?1, ?2, ?3, ?4)",
        )?;
        usage.id = stmt.insert(params![
            usage.start,
            usage.end,
            usage.during_idle,
            usage.session_id
        ])?;
        Ok(())
    }

    pub fn app_by_app_identification(
        // TODO use this instead of just re-getting the app everytime!
        &mut self,
        identification: &AppIdentification,
    ) -> Result<Option<App>> {
        let mut stmt = self.conn.prepare_cached(
            "select 
                    id, name, description, background, 
                        identification_tag, identification_text1
                from Apps where identification_tag=?1 and identification_text1=?2",
        )?;
        let (app_identification_tag, app_identification_text1) =
            Database::expand_identification(identification);
        let app = stmt.query_row(
            params![app_identification_tag, app_identification_text1],
            |row| {
                let identification_tag: i64 = row.get(4)?;
                let identification_text1: String = row.get(5)?;
                let identification = match identification_tag {
                    0 => AppIdentification::Win32 { path: identification_text1 },
                    1 => AppIdentification::Uwp { aumid: identification_text1 },
                    _ => unreachable!()
                };
                Ok(App {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    background: row.get(3)?,
                    identification
                })
            },
        ).optional()?;
        Ok(app)
    }
}
