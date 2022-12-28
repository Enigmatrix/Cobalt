use rusqlite::{params, Connection, OptionalExtension, Statement};
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
    ($conn: expr, $tbl:expr, $mdl:ty) => {{
        let sql = format!(
            "INSERT INTO {} VALUES (NULL{})",
            $tbl,
            ", ?".repeat(<$mdl as $crate::models::Table>::columns().len() - 1)
        );
        prepare_stmt!($conn, sql)
    }};
}

macro_rules! update_stmt {
    ($conn: expr, $tbl:expr, $mdl:ty) => {{
        let update_flds = <$mdl as $crate::models::Table>::columns()
            .iter()
            .skip(1) // skip id column
            .map(|c| format!("{c} = ?"))
            .collect::<Vec<_>>()
            .join(", ");
        let sql = format!("UPDATE {} SET {} WHERE id = ?", $tbl, update_flds);
        prepare_stmt!($conn, sql)
    }};
}

pub struct Database<'a> {
    pub(crate) conn: &'a Connection,
    pub(crate) find_app_stmt: Statement<'a>,
    pub(crate) update_app_stmt: Statement<'a>,
    pub(crate) insert_empty_app_stmt: Statement<'a>,
    pub(crate) insert_session_stmt: Statement<'a>,
    pub(crate) insert_usage_stmt: Statement<'a>,
    pub(crate) insert_interaction_period_stmt: Statement<'a>,
}

impl<'a> Database<'a> {
    // TODO or make this take a AppIdentity? then the cols for identity_* can be
    // non null and will need to change the trigger
    pub fn insert_empty_app(&mut self) -> Result<models::Ref<models::App>> {
        self.insert_empty_app_stmt
            .raw_execute()
            .context("execute insert empty app stmt")?;
        Ok(models::Ref::new(self.conn.last_insert_rowid() as u64))
    }

    pub fn update_app(&mut self, app: &models::App) -> Result<()> {
        let (identity_tag, identity_text0) = self.destructure_identity(&app.identity);
        self.update_app_stmt
            .execute(params![
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

    pub fn find_app(
        &mut self,
        identity: &models::AppIdentity,
    ) -> Result<Option<models::Ref<models::App>>> {
        let (tag, text1) = self.destructure_identity(identity);
        self.find_app_stmt
            .query_row(params! {tag, text1}, |row| row.get(0).map(models::Ref::new))
            .optional()
            .context("find app query")
    }

    fn destructure_identity<'b>(&self, identity: &'b models::AppIdentity) -> (u64, &'b str) {
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
            find_app_stmt: prepare_stmt!(
                conn,
                "SELECT id FROM app WHERE identity_tag = ? AND identity_text0 = ?"
            )?,
            //TODO get table name from model (models::Table::name())
            insert_empty_app_stmt: prepare_stmt!(conn, "INSERT INTO app(id) VALUES (NULL)")
                .context("prep empty app insert stmt")?,
            update_app_stmt: update_stmt!(conn, "app", models::App)
                .context("prep app update stmt")?,
            insert_session_stmt: insert_stmt!(conn, "session", models::Session)
                .context("prep session insert stmt")?,
            insert_usage_stmt: insert_stmt!(conn, "usage", models::Usage)
                .context("prep usage insert stmt")?,
            insert_interaction_period_stmt: insert_stmt!(
                conn,
                "interaction_period",
                models::InteractionPeriod
            )
            .context("interaction period insert stmt")?,
        })
    }
}

impl DatabaseHolder {
    pub fn new(conn_str: &str) -> Result<DatabaseHolder> {
        //TODO the other options can disable mutexes
        let mut conn = Connection::open(conn_str).context("open db using conn str")?;
        // TODO enable fkey
        // TODO turn on WAL

        // run migration
        Migrator::new(&mut conn, default_migrations())
            .migrate()
            .context("run default migration")?;

        Ok(DatabaseHolder { conn })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_all_apps(db: &mut Database<'_>) -> Result<Vec<models::App>> {
        let mut stmt = db.conn.prepare("SELECT * FROM app ORDER BY id")?;
        let rows = stmt.query_map([], |row| {
            Ok(models::App {
                id: models::Ref::new(row.get(0)?),
                name: row.get(1)?,
                description: row.get(2)?,
                company: row.get(3)?,
                color: row.get(4)?,
                identity: if row.get::<usize, u64>(5)? == 0 {
                    models::AppIdentity::Win32 { path: row.get(6)? }
                } else {
                    models::AppIdentity::UWP { aumid: row.get(6)? }
                },
            })
        })?;
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
        let fpath = std::env::temp_dir().join("test.db");
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
    fn insert_empty_app() -> Result<()> {
        let mut db_holder = DatabaseHolder::new(":memory:")?;
        let mut db = db_holder.database()?;
        let app_id = db.insert_empty_app()?;
        assert_ne!(app_id, models::Ref::default()); // test that we actually inserted something
        Ok(())
    }

    #[test]
    fn insert_session() -> Result<()> {
        let mut db_holder = DatabaseHolder::new(":memory:")?;
        let mut db = db_holder.database()?;
        let app_id = db.insert_empty_app()?;
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
        let app_id = db.insert_empty_app()?;
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
    fn update_app() -> Result<()> {
        let mut db_holder = DatabaseHolder::new(":memory:")?;
        let mut db = db_holder.database()?;

        let app_id0 = db.insert_empty_app()?;
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
        db.update_app(&app0)?;
        check_slice_eq(&[app0], &get_all_apps(&mut db)?);

        // update existing app + update to UWP + update with color
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
        db.update_app(&app1)?;
        check_slice_eq(&[app1], &get_all_apps(&mut db)?);

        // Failing duplicate app identity
        let app_id1 = db.insert_empty_app()?;
        let app2 = models::App {
            id: app_id1.clone(),
            name: "NotMail".into(),
            color: Some("#234567".into()),
            company: "Not Daddy MS".into(),
            description: "NotMail App".into(),
            identity: models::AppIdentity::UWP {
                aumid: "MS!Mail".into(),
            }, // dup app identity
        };
        assert!(db.update_app(&app2).is_err());

        Ok(())
    }

    #[test]
    fn find_app() -> Result<()> {
        let mut db_holder = DatabaseHolder::new(":memory:")?;
        let mut db = db_holder.database()?;

        let app_id0 = db.insert_empty_app()?;
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
        db.update_app(&app0)?;

        let app_id1 = db.insert_empty_app()?;
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
        db.update_app(&app1)?;

        assert_eq!(
            db.find_app(&models::AppIdentity::Win32 {
                path: r"C:\Users\User\not_cobalt.exe".into()
            })?,
            None
        );
        assert_eq!(
            db.find_app(&models::AppIdentity::UWP {
                aumid: r"NotMS!Mail".into()
            })?,
            None
        );
        assert_eq!(
            db.find_app(&models::AppIdentity::Win32 {
                path: r"C:\Users\User\cobalt.exe".into()
            })?,
            Some(app_id0)
        );
        assert_eq!(
            db.find_app(&models::AppIdentity::UWP {
                aumid: r"MS!Mail".into()
            })?,
            Some(app_id1)
        );

        Ok(())
    }
}
