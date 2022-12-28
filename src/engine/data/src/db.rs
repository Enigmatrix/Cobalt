use rusqlite::{params, Connection, Result as SqliteResult, Row, Statement};
use utils::errors::*;

use crate::{migrations::default_migrations, migrator::Migrator, models};

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
        let name = <$mdl as $crate::models::Table>::name();
        let sql = format!(
            "INSERT INTO {} VALUES (NULL{})",
            name,
            ", ?".repeat(<$mdl as $crate::models::Table>::columns().len() - 1)
        );
        prepare_stmt!($conn, sql)
    }};
}

macro_rules! update_stmt {
    ($conn: expr, $mdl:ty) => {{
        let name = <$mdl as $crate::models::Table>::name();
        let update_flds = <$mdl as $crate::models::Table>::columns()
            .iter()
            .skip(1) // skip id column
            .map(|c| format!("{c} = ?"))
            .collect::<Vec<_>>()
            .join(", ");
        let sql = format!("UPDATE {} SET {} WHERE id = ?", name, update_flds);
        prepare_stmt!($conn, sql)
    }};
}

pub struct Database<'a> {
    pub(crate) conn: &'a Connection,
    pub(crate) find_or_insert_empty_app_stmt: Statement<'a>,
    pub(crate) initialize_app_stmt: Statement<'a>,
    pub(crate) insert_session_stmt: Statement<'a>,
    pub(crate) insert_usage_stmt: Statement<'a>,
    pub(crate) insert_interaction_period_stmt: Statement<'a>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum FoundOrInserted<T: models::Table> {
    Found(T),
    Inserted(models::Ref<T>),
}

impl<T: models::Table> FoundOrInserted<T> {
    pub fn unwrap_found(&self) -> &T {
        match self {
            Self::Found(v) => v,
            _ => panic!("value is not Found"),
        }
    }

    pub fn unwrap_inserted(&self) -> &models::Ref<T> {
        match self {
            Self::Inserted(v) => v,
            _ => panic!("value is not Inserted"),
        }
    }
}

impl<'a> Database<'a> {
    pub fn initialize_app(&mut self, app: &models::App) -> Result<()> {
        let (identity_tag, identity_text0) = Self::destructure_identity(&app.identity);
        self.initialize_app_stmt
            .execute(params![
                true,
                app.name,
                app.description,
                app.company,
                app.color,
                identity_tag,
                identity_text0,
                app.id
            ])
            .context("execute update app stmt")?;
        Ok(())
    }

    pub fn insert_session(&mut self, session: &mut models::Session) -> Result<()> {
        self.insert_session_stmt
            .execute(params![session.app, session.title, session.cmd_line])
            .context("execute insert session stmt")?;
        session.id = models::Ref::new(self.conn.last_insert_rowid() as u64);
        Ok(())
    }

    pub fn insert_usage(&mut self, usage: &mut models::Usage) -> Result<()> {
        self.insert_usage_stmt
            .execute(params![usage.session, usage.start, usage.end])
            .context("execute insert usage stmt")?;
        usage.id = models::Ref::new(self.conn.last_insert_rowid() as u64);
        Ok(())
    }

    pub fn insert_interaction_period(
        &mut self,
        interaction_period: &mut models::InteractionPeriod,
    ) -> Result<()> {
        self.insert_interaction_period_stmt
            .execute(params![
                interaction_period.start,
                interaction_period.end,
                interaction_period.mouseclicks,
                interaction_period.keystrokes,
            ])
            .context("execute insert interaction period stmt")?;
        interaction_period.id = models::Ref::new(self.conn.last_insert_rowid() as u64);
        Ok(())
    }

    pub fn find_or_insert_empty_app(
        &mut self,
        identity: &models::AppIdentity,
    ) -> Result<FoundOrInserted<models::App>> {
        let (tag, text1) = Self::destructure_identity(identity);

        self.find_or_insert_empty_app_stmt
            .query_row(params! {tag, text1}, |row| Self::to_app(row))
            .context("find or insert empty app query")
    }

    fn to_app(row: &Row) -> SqliteResult<FoundOrInserted<models::App>> {
        let id = models::Ref::new(row.get(0)?);
        Ok(if row.get(1)? {
            let app = models::App {
                id,
                name: row.get(2)?,
                description: row.get(3)?,
                company: row.get(4)?,
                color: row.get(5)?,
                identity: if row.get::<usize, u64>(6)? == 0 {
                    models::AppIdentity::Win32 { path: row.get(7)? }
                } else {
                    models::AppIdentity::UWP { aumid: row.get(7)? }
                },
            };

            FoundOrInserted::Found(app)
        } else {
            FoundOrInserted::Inserted(id)
        })
    }

    fn destructure_identity(identity: &models::AppIdentity) -> (u64, &str) {
        match identity {
            models::AppIdentity::Win32 { path } => (0, path),
            models::AppIdentity::UWP { aumid } => (1, aumid),
        }
    }
}

pub struct DatabaseHolder {
    conn: Connection,
}

impl DatabaseHolder {
    // we borrow as mutably here so that no two databases can exist at the same time
    pub fn database(&mut self) -> Result<Database<'_>> {
        let conn = &self.conn;

        Ok(Database {
            conn,
            find_or_insert_empty_app_stmt: prepare_stmt!(
                conn,
                // The `DO UPDATE id = id` _should_ be replaced with `DO NOTHING` in future versions of SQLite
                // whenever it starts working...
                "INSERT INTO app (identity_tag, identity_text0) VALUES (?, ?)
                    ON CONFLICT (identity_tag, identity_text0) DO UPDATE SET id = id RETURNING *"
            )?,
            initialize_app_stmt: update_stmt!(conn, models::App).context("prep app update stmt")?,
            insert_session_stmt: insert_stmt!(conn, models::Session)
                .context("prep session insert stmt")?,
            insert_usage_stmt: insert_stmt!(conn, models::Usage)
                .context("prep usage insert stmt")?,
            insert_interaction_period_stmt: insert_stmt!(conn, models::InteractionPeriod)
                .context("interaction period insert stmt")?,
        })
    }
}

impl DatabaseHolder {
    pub fn new(conn_str: &str) -> Result<DatabaseHolder> {
        let mut conn = Connection::open(conn_str).context("open db using conn str")?;
        conn.pragma_update(None, "foreign_keys", "ON").context("turn on foreign keys")?;
        conn.pragma_update(None, "journal_mode", "WAL").context("turn on WAL")?;

        // run migration
        Migrator::new(&mut conn, default_migrations())
            .migrate()
            .context("run default migration")?;

        Ok(DatabaseHolder { conn })
    }
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, str::FromStr};

    use super::*;

    fn get_all_apps(db: &mut Database<'_>) -> Result<Vec<FoundOrInserted<models::App>>> {
        let mut stmt = db.conn.prepare("SELECT * FROM app ORDER BY id")?;
        let rows = stmt.query_map([], |row| Database::to_app(row))?;
        let mut v = Vec::new();
        for r in rows {
            v.push(r?);
        }
        Ok(v)
    }

    fn get_all_sessions(db: &mut Database<'_>) -> Result<Vec<models::Session>> {
        let mut stmt = db.conn.prepare("SELECT * FROM session ORDER BY id")?;
        let rows = stmt.query_map([], |row| {
            Ok(models::Session {
                id: models::Ref::new(row.get(0)?),
                app: models::Ref::new(row.get(1)?),
                title: row.get(2)?,
                cmd_line: row.get(3)?,
            })
        })?;
        let mut v = Vec::new();
        for r in rows {
            v.push(r?);
        }
        Ok(v)
    }

    fn get_all_usages(db: &mut Database<'_>) -> Result<Vec<models::Usage>> {
        let mut stmt = db.conn.prepare("SELECT * FROM usage ORDER BY id")?;
        let rows = stmt.query_map([], |row| {
            Ok(models::Usage {
                id: models::Ref::new(row.get(0)?),
                session: models::Ref::new(row.get(1)?),
                start: row.get(2)?,
                end: row.get(3)?,
            })
        })?;
        let mut v = Vec::new();
        for r in rows {
            v.push(r?);
        }
        Ok(v)
    }

    fn get_all_interaction_period(db: &mut Database<'_>) -> Result<Vec<models::InteractionPeriod>> {
        let mut stmt = db
            .conn
            .prepare("SELECT * FROM interaction_period ORDER BY id")?;
        let rows = stmt.query_map([], |row| {
            Ok(models::InteractionPeriod {
                id: models::Ref::new(row.get(0)?),
                start: row.get(1)?,
                end: row.get(2)?,
                mouseclicks: row.get(3)?,
                keystrokes: row.get(4)?,
            })
        })?;
        let mut v = Vec::new();
        for r in rows {
            v.push(r?);
        }
        Ok(v)
    }

    fn check_slice_eq<T: std::fmt::Debug + Eq>(v1: &[T], v2: &[T]) {
        assert_eq!(v1.len(), v2.len(), "collections are not same length");
        for i in 0..v1.len() {
            assert_eq!(v1[i], v2[i], "items at index {} are not equal", i);
        }
    }

    #[test]
    fn create_file_backed_db() -> Result<()> {
        let fpath = PathBuf::from_str("./test.db")?;
        {
            let mut db_holder = DatabaseHolder::new(&fpath.to_string_lossy())?;
            let _ = db_holder.database()?;
        }
        std::fs::remove_file(fpath)?;
        Ok(())
    }

    #[test]
    fn create_memory_backed_db() -> Result<()> {
        let mut db_holder = DatabaseHolder::new(":memory:")?;
        let _ = db_holder.database()?;
        Ok(())
    }

    #[test]
    fn insert_session() -> Result<()> {
        let mut db_holder = DatabaseHolder::new(":memory:")?;
        let mut db = db_holder.database()?;
        let app_identity = models::AppIdentity::Win32 {
            path: r"C:\Users\User\cobalt.exe".into(),
        };
        let app_id = match db.find_or_insert_empty_app(&app_identity)? {
            FoundOrInserted::Inserted(id) => id,
            _ => panic!(),
        };

        let mut session0 = models::Session {
            id: Default::default(),
            app: app_id.clone(),
            title: "Titular title!".into(),
            cmd_line: None,
        };
        db.insert_session(&mut session0)?;
        assert_ne!(session0.id, models::Ref::default());

        let mut session1 = models::Session {
            id: Default::default(),
            app: app_id.clone(),
            title: "I am title!".into(),
            cmd_line: Some("run.exe --best-args".into()),
        };
        db.insert_session(&mut session1)?;
        assert_ne!(session1.id, models::Ref::default());

        check_slice_eq(&[session0, session1], &get_all_sessions(&mut db)?);

        Ok(())
    }

    #[test]
    fn insert_usage() -> Result<()> {
        let mut db_holder = DatabaseHolder::new(":memory:")?;
        let mut db = db_holder.database()?;
        let app_identity = models::AppIdentity::Win32 {
            path: r"C:\Users\User\cobalt.exe".into(),
        };
        let app_id = match db.find_or_insert_empty_app(&app_identity)? {
            FoundOrInserted::Inserted(id) => id,
            _ => panic!(),
        };
        let mut session = models::Session {
            id: Default::default(),
            app: app_id.clone(),
            title: "Titular title!".into(),
            cmd_line: None,
        };
        db.insert_session(&mut session)?;

        let mut usage = models::Usage {
            id: Default::default(),
            session: session.id,
            start: 12345,
            end: 67890,
        };
        db.insert_usage(&mut usage)?;
        assert_ne!(usage.id, models::Ref::default());

        check_slice_eq(&[usage], &get_all_usages(&mut db)?);

        Ok(())
    }

    #[test]
    fn insert_interaction_period() -> Result<()> {
        let mut db_holder = DatabaseHolder::new(":memory:")?;
        let mut db = db_holder.database()?;
        let mut interaction_period = models::InteractionPeriod {
            id: Default::default(),
            start: 12345,
            end: 67890,
            keystrokes: 123,
            mouseclicks: 456,
        };
        db.insert_interaction_period(&mut interaction_period)?;
        assert_ne!(interaction_period.id, models::Ref::default());

        check_slice_eq(&[interaction_period], &get_all_interaction_period(&mut db)?);

        Ok(())
    }

    #[test]
    fn initialize_app() -> Result<()> {
        let mut db_holder = DatabaseHolder::new(":memory:")?;
        let mut db = db_holder.database()?;

        let app_id0 = db.find_or_insert_empty_app(&models::AppIdentity::Win32 {
            path: r"C:\Users\User\cobalt.exe".into(),
        })?;
        let app_id0 = app_id0.unwrap_inserted();

        let app0 = models::App {
            id: app_id0.clone(),
            name: "Cobalt".into(),
            color: None,
            company: "Cobalt Private Limited Corporate of World".into(),
            description: "This app will make your computer explode".into(),
            identity: models::AppIdentity::Win32 {
                path: r"C:\Users\User\cobalt.exe".into(),
            },
        };
        db.initialize_app(&app0)?;

        check_slice_eq(
            &[app0],
            &get_all_apps(&mut db)?
                .into_iter()
                .map(|a| a.unwrap_found().clone())
                .collect::<Vec<_>>(),
        );

        // update existing app + update to UWP + update with color
        // Specifically, we overwrite the AppIdentity with another AppIdentity
        let app1 = models::App {
            id: app_id0.clone(),
            name: "Mail".into(),
            color: Some("#123456".into()),
            company: "Daddy MS".into(),
            description: "Read your mail using a UWP app".into(),
            identity: models::AppIdentity::UWP {
                aumid: "MS!Mail".into(),
            },
        };
        db.initialize_app(&app1)?;
        check_slice_eq(
            &[app1],
            &get_all_apps(&mut db)?
                .into_iter()
                .map(|a| a.unwrap_found().clone())
                .collect::<Vec<_>>(),
        );

        // Failing duplicate app identity
        let app_id2 = db.find_or_insert_empty_app(&models::AppIdentity::UWP {
            aumid: "NotMS!Mail".into(),
        })?;
        let app_id2 = app_id2.unwrap_inserted();

        let app2 = models::App {
            id: app_id2.clone(),
            name: "NotMail".into(),
            color: Some("#234567".into()),
            company: "Not Daddy MS".into(),
            description: "NotMail App".into(),
            identity: models::AppIdentity::UWP {
                aumid: "MS!Mail".into(),
            }, // dup app identity, but in a different row
        };
        assert!(db.initialize_app(&app2).is_err());

        Ok(())
    }

    #[test]
    fn find_or_insert_empty_app() -> Result<()> {
        let mut db_holder = DatabaseHolder::new(":memory:")?;
        let mut db = db_holder.database()?;

        let app_id0 = db.find_or_insert_empty_app(&models::AppIdentity::Win32 {
            path: r"C:\Users\User\cobalt.exe".into(),
        })?;
        let app_id0 = if let FoundOrInserted::Inserted(re) = app_id0 {
            assert_ne!(re, models::Ref::default());
            re
        } else {
            panic!("not inserted");
        };

        let app0 = models::App {
            id: app_id0.clone(),
            name: "Cobalt".into(),
            color: None,
            company: "Cobalt Private Limited Corporate of World".into(),
            description: "This app will make your computer explode".into(),
            identity: models::AppIdentity::Win32 {
                path: r"C:\Users\User\cobalt.exe".into(),
            },
        };
        db.initialize_app(&app0)?;

        let app_id1 = db.find_or_insert_empty_app(&models::AppIdentity::UWP {
            aumid: "MS!Mail".into(),
        })?;
        let app_id1 = if let FoundOrInserted::Inserted(re) = app_id1 {
            assert_ne!(re, models::Ref::default());
            re
        } else {
            panic!("not inserted");
        };

        let app1 = models::App {
            id: app_id1.clone(),
            name: "Mail".into(),
            color: Some("#123456".into()),
            company: "Daddy MS".into(),
            description: "Read your mail using a UWP app".into(),
            identity: models::AppIdentity::UWP {
                aumid: "MS!Mail".into(),
            },
        };
        db.initialize_app(&app1)?;

        assert!(
            matches!( db.find_or_insert_empty_app(&models::AppIdentity::Win32 {
                path: r"C:\Users\User\not_cobalt.exe".into()
            })?, FoundOrInserted::Inserted(r) if r != models::Ref::default())
        );

        assert!(
            matches!( db.find_or_insert_empty_app(&models::AppIdentity::UWP {
                aumid: r"NotMS!Mail".into()
            })?, FoundOrInserted::Inserted(r) if r != models::Ref::default())
        );

        assert_eq!(
            db.find_or_insert_empty_app(&models::AppIdentity::Win32 {
                path: r"C:\Users\User\cobalt.exe".into()
            })?,
            FoundOrInserted::Found(app0)
        );

        assert_eq!(
            db.find_or_insert_empty_app(&models::AppIdentity::UWP {
                aumid: r"MS!Mail".into()
            })?,
            FoundOrInserted::Found(app1)
        );

        Ok(())
    }
}
