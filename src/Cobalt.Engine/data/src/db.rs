use std::io::Write;

use rusqlite::{params, Connection, Statement};
use util::{
    config::Config,
    error::{Context, Result},
};

use crate::{
    entities::{App, AppIdentity, InteractionPeriod, Ref, Session, Usage},
    migrations::Migrator,
    table::Table,
};

#[derive(Debug, PartialEq, Eq)]
pub enum FoundOrInserted<T: Table> {
    Found(Ref<T>),
    Inserted(Ref<T>),
}

impl<T: Table> From<FoundOrInserted<T>> for Ref<T> {
    fn from(value: FoundOrInserted<T>) -> Self {
        match value {
            FoundOrInserted::Found(id) => id,
            FoundOrInserted::Inserted(id) => id,
        }
    }
}

macro_rules! prepare_stmt {
    ($conn: expr, $sql:expr) => {{
        let sql = $sql;
        $conn
            .prepare(&sql)
            .with_context(|| format!("prepare stmt: {sql}"))
    }};
}

macro_rules! insert_stmt {
    ($conn: expr, $mdl:ty) => {{
        let name = <$mdl as $crate::table::Table>::name();
        let sql = format!(
            "INSERT INTO {} VALUES (NULL{})",
            name,
            ", ?".repeat(<$mdl as $crate::table::Table>::columns().len() - 1)
        );
        prepare_stmt!($conn, sql)
    }};
}

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(config: &Config) -> Result<Self> {
        let path = config.connection_string();
        let mut conn = Connection::open(path).context("open conn")?;
        conn.pragma_update(None, "journal_mode", "WAL")
            .context("enable WAL")?;
        Migrator::new(&mut conn).migrate().context("migrate")?;
        Ok(Self { conn })
    }
}

pub struct UsageWriter<'a> {
    conn: &'a Connection,
    find_or_insert_app: Statement<'a>,
    insert_session: Statement<'a>,
    insert_usage: Statement<'a>,
    insert_interaction_period: Statement<'a>,
}

impl<'a> UsageWriter<'a> {
    pub fn new(db: &'a Database) -> Result<Self> {
        let conn = &db.conn;
        let find_or_insert_app = prepare_stmt!(
            conn,
            // We set found=1 to force this query to return a result row regardless of
            // conflict result.
            "INSERT INTO apps (identity_is_win32, identity_path_or_aumid) VALUES (?, ?) ON CONFLICT
                DO UPDATE SET found = 1 RETURNING id, found"
        )?;
        let insert_session = insert_stmt!(conn, Session)?;
        let insert_usage = insert_stmt!(conn, Usage)?;
        let insert_interaction_period = insert_stmt!(conn, InteractionPeriod)?;
        Ok(Self {
            conn,
            find_or_insert_app,
            insert_session,
            insert_usage,
            insert_interaction_period,
        })
    }

    /// Find or insert a [App] by its [AppIdentity]
    pub fn find_or_insert_app(&mut self, identity: &AppIdentity) -> Result<FoundOrInserted<App>> {
        let (tag, text0) = Self::destructure_identity(identity);
        let (app_id, found) = self
            .find_or_insert_app
            .query_row(params![tag, text0], |r| Ok((r.get(0)?, r.get(1)?)))?;

        Ok(if found {
            FoundOrInserted::Found(app_id)
        } else {
            FoundOrInserted::Inserted(app_id)
        })
    }

    /// Insert a [Session] into the [Database]
    pub fn insert_session(&mut self, session: &mut Session) -> Result<()> {
        self.insert_session
            .execute(params![session.app, session.title])?;
        session.id = Ref::new(self.last_insert_id());
        Ok(())
    }

    /// Insert a [Usage] into the [Database]
    pub fn insert_usage(&mut self, usage: &Usage) -> Result<()> {
        self.insert_usage
            .execute(params![usage.session, usage.start, usage.end])?;
        Ok(())
    }

    /// Insert a [InteractionPeriod] into the [Database]
    pub fn insert_interaction_period(
        &mut self,
        interaction_period: &InteractionPeriod,
    ) -> Result<()> {
        self.insert_interaction_period.execute(params![
            interaction_period.start,
            interaction_period.end,
            interaction_period.mouseclicks,
            interaction_period.keystrokes
        ])?;
        Ok(())
    }

    fn last_insert_id(&self) -> u64 {
        self.conn.last_insert_rowid() as u64
    }

    fn destructure_identity(identity: &AppIdentity) -> (u64, &str) {
        match identity {
            AppIdentity::Win32 { path } => (1, path),
            AppIdentity::Uwp { aumid } => (0, aumid),
        }
    }
}

/// Reference to hold statements regarding [App] updates
pub struct AppUpdater<'a> {
    conn: &'a Connection,
    update_app: Statement<'a>,
    update_app_icon_size: Statement<'a>,
}

impl<'a> AppUpdater<'a> {
    /// Initialize a [AppUpdater] from a given [Database]
    pub fn new(db: &'a mut Database) -> Result<Self> {
        let conn = &db.conn;
        Ok(Self {
            update_app: prepare_stmt!(
                conn,
                "UPDATE apps SET
                    name = ?,
                    description = ?,
                    company = ?,
                    color = ?,
                    initialized = 1
                WHERE id = ?"
            )?,
            update_app_icon_size: prepare_stmt!(
                conn,
                "UPDATE apps SET
                    icon = ZEROBLOB(?)
                WHERE id = ?"
            )?,
            conn,
        })
    }

    /// Update the [App] with additional information
    pub fn update_app(&mut self, app: &App) -> Result<()> {
        self.update_app.execute(params![
            app.name,
            app.description,
            app.company,
            app.color,
            app.id
        ])?;
        Ok(())
    }

    /// Get a reference to the [App] icon's writer
    pub fn app_icon_writer(&mut self, app_id: Ref<App>, size: u64) -> Result<impl Write + 'a> {
        self.update_app_icon_size(app_id.clone(), size)?;
        Ok(self.conn.blob_open(
            rusqlite::DatabaseName::Main,
            "apps",
            "icon",
            app_id.inner as i64,
            false,
        )?)
    }

    /// Update the [App] with additional information
    fn update_app_icon_size(&mut self, id: Ref<App>, icon_size: u64) -> Result<()> {
        self.update_app_icon_size.execute(params![icon_size, id])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    fn test_db() -> Result<Database> {
        let mut conn = Connection::open_in_memory()?;
        Migrator::new(&mut conn).migrate()?;
        Ok(Database { conn })
    }

    #[test]
    fn insert_new_app() -> Result<()> {
        let db = test_db()?;
        let mut writer = UsageWriter::new(&db)?;
        let res = writer.find_or_insert_app(&AppIdentity::Win32 {
            path: "notepad.exe".to_string(),
        })?;
        assert_eq!(res, FoundOrInserted::Inserted(Ref::<App>::new(1)));
        let (init, path): (bool, String) =
            db.conn.query_row("SELECT * FROM apps", params![], |f| {
                Ok((f.get("initialized")?, f.get("identity_path_or_aumid")?))
            })?;
        assert_eq!("notepad.exe", path);
        assert_eq!(false, init);
        Ok(())
    }

    #[test]
    fn found_old_app() -> Result<()> {
        let db = test_db()?;
        let mut writer = UsageWriter::new(&db)?;
        let res = writer.find_or_insert_app(&AppIdentity::Win32 {
            path: "notepad.exe".to_string(),
        })?;
        assert_eq!(res, FoundOrInserted::Inserted(Ref::<App>::new(1)));
        let res = writer.find_or_insert_app(&AppIdentity::Win32 {
            path: "notepad.exe".to_string(),
        })?;
        assert_eq!(res, FoundOrInserted::Found(Ref::<App>::new(1)));
        Ok(())
    }

    #[test]
    fn insert_session() -> Result<()> {
        let db = test_db()?;
        let mut writer = UsageWriter::new(&db)?;
        writer.find_or_insert_app(&AppIdentity::Win32 {
            path: "notepad.exe".to_string(),
        })?;
        let mut sess = Session {
            id: Default::default(),
            app: Ref::new(1),
            title: "TITLE".to_string(),
        };
        writer.insert_session(&mut sess)?;
        let sess_from_db = db
            .conn
            .query_row("SELECT * FROM sessions", params![], |f| {
                Ok(Session {
                    id: f.get(0)?,
                    app: f.get(1)?,
                    title: f.get(2)?,
                })
            })?;
        assert_eq!(sess, sess_from_db);
        Ok(())
    }

    #[test]
    fn insert_usage() -> Result<()> {
        let db = test_db()?;
        let mut writer = UsageWriter::new(&db)?;
        writer.find_or_insert_app(&AppIdentity::Win32 {
            path: "notepad.exe".to_string(),
        })?;
        let mut sess = Session {
            id: Default::default(),
            app: Ref::new(1),
            title: "TITLE".to_string(),
        };
        writer.insert_session(&mut sess)?;
        let mut usage = Usage {
            id: Default::default(),
            session: sess.id.clone(),
            start: 42,
            end: 420,
        };
        writer.insert_usage(&mut usage)?;
        let usage_from_db = db.conn.query_row("SELECT * FROM usages", params![], |f| {
            Ok(Usage {
                id: f.get(0)?,
                session: f.get(1)?,
                start: f.get(2)?,
                end: f.get(3)?,
            })
        })?;
        usage.id = Ref::new(1); // not updated by writer
        assert_eq!(usage, usage_from_db);
        Ok(())
    }

    #[test]
    fn insert_interaction_period() -> Result<()> {
        let db = test_db()?;
        let mut writer = UsageWriter::new(&db)?;
        writer.find_or_insert_app(&AppIdentity::Win32 {
            path: "notepad.exe".to_string(),
        })?;
        let mut ip = InteractionPeriod {
            id: Default::default(),
            start: 42,
            end: 420,
            mouseclicks: 23,
            keystrokes: 45,
        };
        writer.insert_interaction_period(&ip)?;
        let ip_from_db =
            db.conn
                .query_row("SELECT * FROM interaction_periods", params![], |f| {
                    Ok(InteractionPeriod {
                        id: f.get(0)?,
                        start: f.get(1)?,
                        end: f.get(2)?,
                        mouseclicks: f.get(3)?,
                        keystrokes: f.get(4)?,
                    })
                })?;
        ip.id = Ref::new(1);
        assert_eq!(ip, ip_from_db);
        Ok(())
    }

    #[test]
    fn update_app() -> Result<()> {
        let mut db = test_db()?;
        let mut writer = UsageWriter::new(&db)?;
        let identity = AppIdentity::Win32 {
            path: "notepad.exe".to_string(),
        };
        writer.find_or_insert_app(&identity)?;
        drop(writer);
        let mut updater = AppUpdater::new(&mut db)?;
        let app = App {
            id: Ref::new(1),
            name: "name".to_string(),
            description: "desc".to_string(),
            company: "comp".to_string(),
            color: "red".to_string(),
            identity: identity.clone(), // ignored by query
        };
        updater.update_app(&app)?;
        drop(updater);

        let (init, app_from_db): (bool, _) =
            db.conn.query_row("SELECT * FROM apps", params![], |f| {
                Ok((
                    f.get("initialized")?,
                    App {
                        id: f.get("id")?,
                        name: f.get("name")?,
                        description: f.get("description")?,
                        company: f.get("company")?,
                        color: f.get("color")?,
                        identity: if f.get("identity_is_win32")? {
                            AppIdentity::Win32 {
                                path: f.get("identity_path_or_aumid")?,
                            }
                        } else {
                            AppIdentity::Uwp {
                                aumid: f.get("identity_path_or_aumid")?,
                            }
                        },
                    },
                ))
            })?;

        assert_eq!(true, init);
        assert_eq!(app, app_from_db);
        Ok(())
    }

    #[test]
    fn update_app_icon() -> Result<()> {
        let mut db = test_db()?;
        let mut writer = UsageWriter::new(&db)?;
        let identity = AppIdentity::Win32 {
            path: "notepad.exe".to_string(),
        };
        writer.find_or_insert_app(&identity)?;
        drop(writer);

        let icon = &[42, 233].repeat(50); // 50 * 2 = 100 bytes length

        {
            let mut updater = AppUpdater::new(&mut db)?;
            let mut writer = updater.app_icon_writer(Ref::new(1), icon.len() as u64)?;
            writer.write_all(&icon)?;
        }

        let icon_from_db: Vec<u8> = db
            .conn
            .query_row("SELECT icon FROM apps", params![], |f| f.get("icon"))?;

        assert_eq!(icon, &icon_from_db);
        Ok(())
    }
}
