use std::io::Write;

use common::errors::*;
use common::settings::Settings;
use rusqlite::{params, Connection, Statement};

use crate::entities::*;
use crate::table::{Ref, Table};

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
        let sql = if <$mdl as $crate::table::Table>::has_id() {
            format!(
                "INSERT INTO {} ({}) VALUES (NULL{})",
                name,
                <$mdl as $crate::table::Table>::columns().join(", "),
                ", ?".repeat(<$mdl as $crate::table::Table>::columns().len() - 1)
            )
        } else {
            format!(
                "INSERT INTO {} ({}) VALUES ({})",
                name,
                <$mdl as $crate::table::Table>::columns().join(", "),
                ", ?".repeat(<$mdl as $crate::table::Table>::columns().len())
            )
        };
        prepare_stmt!($conn, sql)
    }};
}

pub enum FoundOrInserted<T: Table> {
    Found(Ref<T>),
    Inserted(Ref<T>),
}

/// Represents a connection to the database
pub struct Database {
    pub(crate) conn: Connection,
}

impl Database {
    /// Create a new [Database] from a connection string
    pub fn new(settings: &Settings) -> Result<Database> {
        // TODO check if arbitrary conn_str works
        let conn = Connection::open(&settings.connection_strings.database_path)
            .context("open connection")?;
        Ok(Database { conn })
    }
}

/// Reference to hold statements regarding entity insertion
pub struct EntityInserter<'a> {
    conn: &'a Connection,
    find_or_insert_app_stmt: Statement<'a>,
    insert_session: Statement<'a>,
    insert_usage: Statement<'a>,
    insert_interaction_period: Statement<'a>,
}

impl<'a> EntityInserter<'a> {
    /// Initialize a [EntityInserter] from a given [Database]
    pub fn from(db: &'a mut Database) -> Result<Self> {
        let conn = &db.conn;
        Ok(Self {
            find_or_insert_app_stmt: prepare_stmt!(
                conn,
                "INSERT INTO app (identity_tag, identity_text0) VALUES (?, ?) ON CONFLICT
                    DO UPDATE SET found = 1 RETURNING id, found"
            )
            .context("find or insert app stmt")?,
            insert_session: insert_stmt!(conn, Session).context("session insert stmt")?,
            insert_usage: insert_stmt!(conn, Usage).context("usage insert stmt")?,
            insert_interaction_period: insert_stmt!(conn, InteractionPeriod)
                .context("interaction period insert stmt")?,
            conn,
        })
    }

    /// Find or insert a [App] by its [AppIdentity]
    pub fn find_or_insert_app(&mut self, identity: &AppIdentity) -> Result<FoundOrInserted<App>> {
        let (tag, text0) = Self::destructure_identity(identity);
        let (app_id, found) = self
            .find_or_insert_app_stmt
            .query_row(params![tag, text0], |r| Ok((r.get(0)?, r.get(1)?)))
            .context("find or insert app stmt execute")?;

        Ok(if found {
            FoundOrInserted::Found(app_id)
        } else {
            FoundOrInserted::Inserted(app_id)
        })
    }

    /// Insert a [Session] into the [Database]
    pub fn insert_session(&mut self, session: &mut Session) -> Result<()> {
        self.insert_session
            .execute(params![session.app, session.title, session.cmd_line])
            .context("insert session stmt execute")?;
        session.id = Ref::new(self.last_insert_id());
        Ok(())
    }

    /// Insert a [Usage] into the [Database]
    pub fn insert_usage(&mut self, usage: &mut Usage) -> Result<()> {
        self.insert_usage
            .execute(params![usage.session, usage.start, usage.end])
            .context("insert usage stmt execute")?;
        usage.id = Ref::new(self.last_insert_id());
        Ok(())
    }

    /// Insert a [InteractionPeriod] into the [Database]
    pub fn insert_interaction_period(
        &mut self,
        interaction_period: &InteractionPeriod,
    ) -> Result<()> {
        self.insert_interaction_period
            .execute(params![
                interaction_period.start,
                interaction_period.end,
                interaction_period.mouseclicks,
                interaction_period.keystrokes
            ])
            .context("insert interaction period stmt execute")?;
        Ok(())
    }

    fn last_insert_id(&self) -> u64 {
        self.conn.last_insert_rowid() as u64
    }

    fn destructure_identity(identity: &AppIdentity) -> (u64, &str) {
        match identity {
            AppIdentity::Win32 { path } => (0, path),
            AppIdentity::Uwp { aumid } => (1, aumid),
        }
    }
}

/// Reference to hold statements regarding [App] updates
pub struct AppUpdater<'a> {
    conn: &'a Connection,
    update_app: Statement<'a>,
}

impl<'a> AppUpdater<'a> {
    /// Initialize a [AppUpdater] from a given [Database]
    pub fn from(db: &'a mut Database) -> Result<Self> {
        let conn = &db.conn;
        Ok(Self {
            update_app: prepare_stmt!(
                conn,
                "UPDATE app SET
                    name = ?,
                    description = ?,
                    company = ?,
                    color = ?
                    icon = ZEROBLOB(?),
                    initialized = 1
                WHERE id = ?"
            )
            .context("update app stmt")?,
            conn,
        })
    }

    /// Update the [App] with additional information
    pub fn update_app(&mut self, app: &App, icon_size: u64) -> Result<()> {
        self.update_app
            .execute(params![
                app.name,
                app.description,
                app.company,
                app.color,
                icon_size,
                app.id
            ])
            .context("update app stmt execute")?;
        Ok(())
    }

    /// Get a reference to the [App] icon's writer
    pub fn app_icon(&'a self, app_id: Ref<App>) -> Result<impl Write + 'a> {
        self.conn
            .blob_open(
                rusqlite::DatabaseName::Main,
                "app",
                "icon",
                app_id.inner as i64,
                false,
            )
            .context("open app icon blob")
    }
}
