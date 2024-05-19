use std::io::Write;

use rusqlite::{params, Connection, Statement};
use util::{
    config::Config,
    error::{Context, Result},
    time::TimeSystem,
};

use crate::{
    entities::{
        Alert, AlertEvent, App, AppIdentity, InteractionPeriod, Ref, Reminder, ReminderEvent,
        Session, Target, TimeFrame, Timestamp, TriggerAction, Usage, VersionedId,
    },
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

/// Reference to hold statements regarding [Alert] queries
pub struct AlertManager<'a> {
    get_tag_apps: Statement<'a>,
    triggered_alerts: Statement<'a>,
    triggered_reminders: Statement<'a>,
    insert_alert_event: Statement<'a>,
    insert_reminder_event: Statement<'a>,
}

impl<'a> AlertManager<'a> {
    /// Initialize a [AlertManager] from a given [Database]
    pub fn new(db: &'a mut Database) -> Result<Self> {
        let conn = &db.conn;
        let get_tag_apps = prepare_stmt!(conn, "SELECT app_id FROM _app_tags WHERE tag_id = ?")?;
        let triggered_alerts = prepare_stmt!(conn, include_str!("../queries/triggered_alerts.sql"))?;
        let triggered_reminders =
            prepare_stmt!(conn, include_str!("../queries/triggered_reminders.sql"))?;
        let insert_alert_event = insert_stmt!(conn, AlertEvent)?;
        let insert_reminder_event = insert_stmt!(conn, ReminderEvent)?;
        Ok(Self {
            get_tag_apps,
            triggered_alerts,
            triggered_reminders,
            insert_alert_event,
            insert_reminder_event,
        })
    }

    /// Gets all [App]s under the [Target]
    pub fn target_apps(&mut self, target: &Target) -> Result<Vec<Ref<App>>> {
        Ok(match target {
            Target::Tag(tag) => self
                .get_tag_apps
                .query_map(params![tag], |row| row.get(0))?
                .collect::<Result<Vec<_>, _>>()?,
            // this will only return one result, but we get a row iterator nonetheless
            Target::App(app) => vec![app.clone()],
        })
    }

    // TODO optim: use a single query to get all triggered alerts and reminders
    // TODO optim: or use a single transaction for the below two.

    /// Get all [Alert]s that are triggered, including when they were triggered
    pub fn triggered_alerts(
        &mut self,
        times: &impl TimeSystem,
    ) -> Result<Vec<(Alert, Option<Timestamp>)>> {
        let day_start = times.day_start().to_ticks();
        let week_start = times.week_start().to_ticks();
        let month_start = times.month_start().to_ticks();
        let result = self
            .triggered_alerts
            .query_map(params![day_start, week_start, month_start], |row| {
                Ok((Self::row_to_alert(row)?, row.get(9)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(result)
    }

    /// Get all [Reminder]s that are triggered, except those that are already handled
    pub fn triggered_reminders(&mut self, times: &impl TimeSystem) -> Result<Vec<Reminder>> {
        let day_start = times.day_start().to_ticks();
        let week_start = times.week_start().to_ticks();
        let month_start = times.month_start().to_ticks();
        let result = self
            .triggered_reminders
            .query_map(
                params![day_start, week_start, month_start],
                Self::row_to_reminder,
            )?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(result)
    }

    /// Insert a [AlertEvent]
    pub fn insert_alert_event(&mut self, event: &AlertEvent) -> Result<()> {
        self.insert_alert_event.execute(params![
            event.alert.guid,
            event.alert.version,
            event.timestamp,
        ])?;
        Ok(())
    }

    /// Insert a [ReminderEvent]
    pub fn insert_reminder_event(&mut self, event: &ReminderEvent) -> Result<()> {
        self.insert_reminder_event.execute(params![
            event.reminder.guid,
            event.reminder.version,
            event.timestamp,
        ])?;
        Ok(())
    }

    fn row_to_alert(row: &rusqlite::Row) -> Result<Alert, rusqlite::Error> {
        let app_id: Option<Ref<App>> = row.get(2)?;
        Ok(Alert {
            id: Ref::new(VersionedId {
                guid: row.get(0)?,
                version: row.get(1)?,
            }),
            target: if let Some(app_id) = app_id {
                Target::App(app_id)
            } else {
                Target::Tag(row.get(3)?)
            },
            usage_limit: row.get(4)?,
            time_frame: match row.get(5)? {
                0 => TimeFrame::Daily,
                1 => TimeFrame::Weekly,
                2 => TimeFrame::Monthly,
                _ => unreachable!("time frame"),
            },
            trigger_action: match row.get(8)? {
                0 => TriggerAction::Kill,
                1 => TriggerAction::Dim(row.get(6)?),
                2 => TriggerAction::Message(row.get(7)?),
                _ => unreachable!("trigger action"),
            },
        })
    }

    fn row_to_reminder(row: &rusqlite::Row) -> Result<Reminder, rusqlite::Error> {
        Ok(Reminder {
            id: Ref::new(VersionedId {
                guid: row.get(0)?,
                version: row.get(1)?,
            }),
            alert: Ref::new(VersionedId {
                guid: row.get(2)?,
                version: row.get(3)?,
            }),
            threshold: row.get(4)?,
            message: row.get(5)?,
        })
    }
}

#[cfg(test)]
mod tests;
