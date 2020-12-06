use super::model;
use crate::data::migrations::Migrator;
use blob::ZeroBlob;
use rusqlite::*;
use util::{Result, *};

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new() -> Result<Database> {
        // TODO read connection string from config
        let mut conn =
            Connection::open_in_memory().with_context(|| "Opening connection to database")?;
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
            .insert(params![
                session.title,
                session.arguments,
                session.app_id
            ])
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
        let app = stmt.query_row(params![identity_tag, identity_text1], |q| {
            Ok(model::App {
                id: q.get(0)?,
                name: q.get(1)?,
                description: q.get(2)?,
                color: q.get(3)?,
                identity: model::AppIdentity::UWP {
                    aumid: String::new(),
                },
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
}
