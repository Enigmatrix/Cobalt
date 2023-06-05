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
        let sql = format!(
            "INSERT INTO {} VALUES (NULL{})",
            name,
            ", ?".repeat(<$mdl as $crate::table::Table>::columns().len() - 1)
        );
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
                // We set found=1 to force this query to return a result row regardless of
                // conflict result.
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
    pub fn insert_usage(&mut self, usage: &Usage) -> Result<()> {
        self.insert_usage
            .execute(params![usage.session, usage.start, usage.end])
            .context("insert usage stmt execute")?;
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
    update_app_icon_size: Statement<'a>,
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
                    color = ?,
                    initialized = 1
                WHERE id = ?"
            )
            .context("update app stmt")?,
            update_app_icon_size: prepare_stmt!(
                conn,
                "UPDATE app SET
                    icon = ZEROBLOB(?)
                WHERE id = ?"
            )
            .context("update app stmt")?,
            conn,
        })
    }

    /// Update the [App] with additional information
    pub fn update_app_icon_size(&mut self, id: Ref<App>, icon_size: u64) -> Result<()> {
        self.update_app_icon_size
            .execute(params![icon_size, id])
            .context("update app stmt execute")?;
        Ok(())
    }

    /// Update the [App] with additional information
    pub fn update_app(&mut self, app: &App) -> Result<()> {
        self.update_app
            .execute(params![
                app.name,
                app.description,
                app.company,
                app.color,
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

pub enum Triggered {
    Alert {
        id: Ref<Alert>,
        app: Ref<App>,
        name: String,
        identity: AppIdentity,
        action: Action,
    },
    Reminder {
        id: Ref<Reminder>,
        alert: Ref<Alert>,
        duration: Duration,
        usage_limit: Duration,
        threshold: f64,
        message: String,
    },
}

/// Reference to hold statements regarding [Alert] and [Reminder] fetching
pub struct Alerter<'a> {
    fetch_triggered: Statement<'a>,
    insert_alert_hit: Statement<'a>,
    insert_reminder_hit: Statement<'a>,
}

impl<'a> Alerter<'a> {
    /// Initialize a [Alerter] from a given [Database]
    pub fn from(db: &'a mut Database) -> Result<Self> {
        Ok(Self {
            fetch_triggered: prepare_stmt!(db.conn, include_str!("queries/alert.sql"))
                .context("fetch triggered stmt")?,
            insert_alert_hit: insert_stmt!(db.conn, AlertHit).context("insert alert hit stmt")?,
            insert_reminder_hit: insert_stmt!(db.conn, ReminderHit)
                .context("insert reminder hit stmt")?,
        })
    }

    /// Fetch all triggered [Alert] and [Reminder]
    pub fn fetch_triggered(
        &'a mut self,
        day_start: Timestamp,
        week_start: Timestamp,
        month_start: Timestamp,
    ) -> Result<impl Iterator<Item = Result<Triggered, rusqlite::Error>> + 'a> {
        let res = self
            .fetch_triggered
            .query_map(params![day_start, week_start, month_start], |r| {
                Ok(if r.get(0)? {
                    Triggered::Alert {
                        id: r.get(1)?,
                        app: r.get(2)?,
                        name: r.get(3)?,
                        identity: match r.get(4)? {
                            0 => AppIdentity::Win32 { path: r.get(5)? },
                            1 => AppIdentity::Uwp { aumid: r.get(5)? },
                            x => Err(rusqlite::Error::InvalidPath(
                                format!("bad index for AppIdentity: {x}").into(),
                            ))?,
                        },
                        action: match r.get(6)? {
                            0 => Action::Kill,
                            1 => Action::Dim(r.get(7)?),
                            2 => Action::Message(r.get(8)?),
                            x => Err(rusqlite::Error::InvalidPath(
                                format!("bad index for Action: {x}").into(),
                            ))?,
                        },
                    }
                } else {
                    Triggered::Reminder {
                        id: r.get(1)?,
                        alert: r.get(2)?,
                        duration: r.get(3)?,
                        usage_limit: r.get(4)?,
                        threshold: r.get(5)?,
                        message: r.get(6)?,
                    }
                })
            })
            .context("fetch stmt execute")?;
        Ok(res)
    }

    /// Insert a [AlertHit] into the [Database]
    pub fn insert_alert_hit(&mut self, hit: &AlertHit) -> Result<()> {
        self.insert_alert_hit
            .execute(params![hit.alert, hit.timestamp])
            .context("insert alert hit stmt execute")?;
        Ok(())
    }

    /// Insert a [ReminderHit] into the [Database]
    pub fn insert_reminder_hit(&mut self, hit: &ReminderHit) -> Result<()> {
        self.insert_reminder_hit
            .execute(params![hit.reminder, hit.timestamp])
            .context("insert reminder hit stmt execute")?;
        Ok(())
    }
}
