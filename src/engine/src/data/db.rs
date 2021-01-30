use super::model;
use crate::data::migrations::Migrator;
use blob::ZeroBlob;
use rusqlite::*;
use std::io::{copy, Read, Seek};
use util::{config::Config, Result, *};

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new() -> Result<Database> {
        let conn_str = Config::data_dir().with_context(|| "Find data directory of App")?.join("data.db");
        let mut conn =
            Connection::open(conn_str).with_context(|| "Opening connection to database")?;
        conn.pragma_update(None, "journal_mode", &"wal")?;
        Migrator::migrate(&mut conn).with_context(|| "Run migrations on database")?;
        Ok(Database { conn })
    }

    pub fn insert_app(&mut self, app: &mut model::App) -> Result<()> {
        let mut stmt = self
            .conn
            .prepare_cached(
                "insert into Apps (
                Name,
                Description,
                Icon,
                Color,
                Identity_Tag,
                Identity_Text1
            ) values ( ?1, ?2, ?3, ?4, ?5, ?6 )",
            )
            .with_context(|| "Get cached App insert statement")?;

        let (identity_tag, identity_text1) = Database::unwrap_app_identity(&app.identity);
        app.id = stmt
            .insert(params![
                app.name,
                app.description,
                Option::<ZeroBlob>::None,
                app.color,
                identity_tag,
                identity_text1
            ])
            .with_context(|| "Insert App")?;

        Ok(())
    }

    pub fn insert_session(&mut self, session: &mut model::Session) -> Result<()> {
        let mut stmt = self
            .conn
            .prepare_cached(
                "insert into Sessions (
                Title,
                Arguments,
                AppId
            ) values ( ?1, ?2, ?3 )",
            )
            .with_context(|| "Get cached Session insert statement")?;

        session.id = stmt
            .insert(params![session.title, session.arguments, session.app_id])
            .with_context(|| "Insert Session")?;

        Ok(())
    }

    pub fn insert_usage(&mut self, usage: &mut model::Usage) -> Result<()> {
        let mut stmt = self
            .conn
            .prepare_cached(
                "insert into Usages (
                Start,
                End,
                DuringIdle,
                SessionId
            ) values ( ?1, ?2, ?3, ?4 )",
            )
            .with_context(|| "Get cached Usage insert statement")?;

        usage.id = stmt
            .insert(params![
                usage.start.to_i64(),
                usage.end.to_i64(),
                usage.idle,
                usage.sess_id,
            ])
            .with_context(|| "Insert Usage")?;

        Ok(())
    }

    pub fn update_app(
        &mut self,
        app_id: model::Id,
        name: String,
        description: String,
        mut icon: impl Read + Seek,
    ) -> Result<()> {
        let mut stmt = self
            .conn
            .prepare_cached(
                "
            update Apps set
                Name = ?1,
                Description = ?2,
                Icon = ?3
            where Id = ?4
            ",
            )
            .with_context(|| "Get cached Session insert statement")?;

        let stream_len = icon
            .stream_len()
            .with_context(|| "Get length of icon stream")?;

        stmt.execute(params![
            name,
            description,
            ZeroBlob(stream_len as i32),
            app_id
        ])
        .with_context(|| "Update App with extra information")?;

        let mut blob = self
            .conn
            .blob_open(DatabaseName::Main, "Apps", "Icon", app_id, false)
            .with_context(|| "Open icon blob for writing")?;
        copy(&mut icon, &mut blob).with_context(|| "Copy icon to blob")?; // TODO check buffering

        Ok(())
    }

    pub fn app_by_identity(&mut self, identity: &model::AppIdentity) -> Result<Option<model::App>> {
        let mut stmt = self
            .conn
            .prepare_cached(
                "select
                Id,
                Name,
                Description,
                /* Icon, */
                Color,
                Identity_Tag,
                Identity_Text1
            from Apps where Identity_Tag = ?1 and Identity_Text1 = ?2",
            )
            .with_context(|| "Get cached find App by AppIdentity statement")?;
        let (identity_tag, identity_text1) = Database::unwrap_app_identity(identity);
        let app = stmt
            .query_row(params![identity_tag, identity_text1], |q| {
                Ok(model::App {
                    id: q.get(0)?,
                    name: q.get(1)?,
                    description: q.get(2)?,
                    color: q.get(3)?,
                    identity: Database::wrap_app_identity(q.get(4)?, q.get(5)?),
                })
            })
            .optional()
            .with_context(|| "Query App by AppIdentity")?;

        Ok(app)
    }

    fn unwrap_app_identity(identity: &model::AppIdentity) -> (i64, &String) {
        match identity {
            model::AppIdentity::Win32 { path } => (0, path),
            model::AppIdentity::UWP { aumid } => (1, aumid),
        }
    }

    fn wrap_app_identity(tag: i64, text1: String) -> model::AppIdentity {
        match tag {
            0 => model::AppIdentity::Win32 { path: text1 },
            1 => model::AppIdentity::UWP { aumid: text1 },
            _ => unreachable!(),
        }
    }
}
