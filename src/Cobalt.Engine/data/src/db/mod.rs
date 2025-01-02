use std::io::Write;
use std::str::FromStr;

use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode};
use sqlx::{query_as, ConnectOptions, Connection, Executor, Sqlite, SqliteConnection, Transaction};
use util::config::Config;
use util::error::{Context, Result};
use util::time::{TimeSystem, ToTicks};

use crate::entities::{
    Alert, AlertEvent, App, AppIdentity, InteractionPeriod, Ref, Reminder, ReminderEvent, Session,
    Target, TimeFrame, Timestamp, TriggerAction, Usage, VersionedId,
};
use crate::migrations::Migrator;
use crate::table::Table;

/// Either we found the row or we just inserted it.
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

/// Database connection stored in the file system.
pub struct Database {
    conn: SqliteConnection,
}

impl Database {
    /// Create a new [Database] from the given [Config]
    pub async fn new(config: &Config) -> Result<Self> {
        let path = config.connection_string()?;
        let conn = SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .connect()
            .await?;
        let mut ret = Self { conn };
        Migrator::new(&mut ret).migrate().await.context("migrate")?;
        Ok(ret)
    }

    pub(crate) fn executor(&mut self) -> impl Executor<'_, Database = Sqlite> {
        &mut self.conn
    }

    pub(crate) async fn transaction(&mut self) -> Result<Transaction<'_, Sqlite>> {
        Ok(self.conn.begin().await.context("begin transaction")?)
    }
}

// /// Reference to hold statements regarding [Usage] and associated entities' queries
// pub struct UsageWriter {
//     db: Database,
// }

// impl UsageWriter {
//     /// Initialize a [UsageWriter] from a given [Database]
//     pub fn new(db: Database) -> Result<Self> {
//         // let conn = &db.conn;
//         // let find_or_insert_app = prepare_stmt!(
//         //     conn,
//         //     // We set found=1 to force this query to return a result row regardless of
//         //     // conflict result.
//         //     "INSERT INTO apps (identity_is_win32, identity_path_or_aumid) VALUES (?, ?) ON CONFLICT
//         //         DO UPDATE SET found = 1 RETURNING id, found"
//         // )?;
//         // let insert_session = insert_stmt!(conn, Session)?;
//         // let insert_usage = insert_stmt!(conn, Usage)?;
//         // let update_usage = prepare_stmt!(conn, "UPDATE usages SET end = ? WHERE id = ?")?;
//         // let insert_interaction_period = insert_stmt!(conn, InteractionPeriod)?;
//         Ok(Self { db })
//     }

//     /// Find or insert a [App] by its [AppIdentity]
//     pub async fn find_or_insert_app(
//         &mut self,
//         identity: &AppIdentity,
//     ) -> Result<FoundOrInserted<App>> {
//         let (tag, text0) = Self::destructure_identity(identity);
//         let (app_id, found): (Ref<App>, bool) = query_as(
//             // We set found=1 to force this query to return a result row regardless of
//             // conflict result.
//             "INSERT INTO apps (identity_is_win32, identity_path_or_aumid) VALUES (?, ?) ON CONFLICT
//                 DO UPDATE SET found = 1 RETURNING id, found",
//         )
//         .bind(tag)
//         .bind(text0)
//         .fetch_one(self.db.executor())
//         .await?;

//         Ok(if found {
//             FoundOrInserted::Found(app_id)
//         } else {
//             FoundOrInserted::Inserted(app_id)
//         })
//     }

// /// Insert a [Session] into the [Database]
// pub fn insert_session(&mut self, session: &mut Session) -> Result<()> {
//     self.insert_session
//         .execute(params![session.app, session.title])?;
//     session.id = Ref::new(self.last_insert_id());
//     Ok(())
// }

// /// Insert or update (only the end timestamp) a [Usage] into the [Database]
// pub fn insert_or_update_usage(&mut self, usage: &mut Usage) -> Result<()> {
//     if usage.id == Ref::default() {
//         self.insert_usage
//             .execute(params![usage.session, usage.start, usage.end])?;
//         usage.id = Ref::new(self.last_insert_id());
//     } else {
//         self.update_usage.execute(params![usage.end, usage.id])?;
//     }
//     Ok(())
// }

// /// Insert a [InteractionPeriod] into the [Database]
// pub fn insert_interaction_period(
//     &mut self,
//     interaction_period: &InteractionPeriod,
// ) -> Result<()> {
//     self.insert_interaction_period.execute(params![
//         interaction_period.start,
//         interaction_period.end,
//         interaction_period.mouse_clicks,
//         interaction_period.key_strokes
//     ])?;
//     Ok(())
// }

// /// Get the last inserted row id
// fn last_insert_id(&self) -> u64 {
//     self.conn.last_insert_rowid() as u64
// }

//     fn destructure_identity(identity: &AppIdentity) -> (i64, &str) {
//         match identity {
//             AppIdentity::Win32 { path } => (1, path),
//             AppIdentity::Uwp { aumid } => (0, aumid),
//         }
//     }
// }

// /// Reference to hold statements regarding [App] updates
// pub struct AppUpdater<'a> {
//     conn: &'a Connection,
//     update_app: Statement<'a>,
//     update_app_icon_size: Statement<'a>,
// }

// impl<'a> AppUpdater<'a> {
//     /// Initialize a [AppUpdater] from a given [Database]
//     pub fn new(db: &'a mut Database) -> Result<Self> {
//         let conn = &db.conn;
//         Ok(Self {
//             update_app: prepare_stmt!(
//                 conn,
//                 "UPDATE apps SET
//                     name = ?,
//                     description = ?,
//                     company = ?,
//                     color = ?,
//                     initialized = 1
//                 WHERE id = ?"
//             )?,
//             update_app_icon_size: prepare_stmt!(
//                 conn,
//                 "UPDATE apps SET
//                     icon = ZEROBLOB(?)
//                 WHERE id = ?"
//             )?,
//             conn,
//         })
//     }

//     /// Update the [App] with additional information
//     pub fn update_app(&mut self, app: &App) -> Result<()> {
//         self.update_app.execute(params![
//             app.name,
//             app.description,
//             app.company,
//             app.color,
//             app.id
//         ])?;
//         Ok(())
//     }

//     /// Get a reference to the [App] icon's writer
//     pub fn app_icon_writer(&mut self, app_id: Ref<App>, size: u64) -> Result<impl Write + 'a> {
//         self.update_app_icon_size(app_id.clone(), size)?;
//         Ok(self.conn.blob_open(
//             rusqlite::DatabaseName::Main,
//             "apps",
//             "icon",
//             *app_id as i64,
//             false,
//         )?)
//     }

//     /// Update the [App] with additional information
//     fn update_app_icon_size(&mut self, id: Ref<App>, icon_size: u64) -> Result<()> {
//         self.update_app_icon_size.execute(params![icon_size, id])?;
//         Ok(())
//     }
// }

// /// [Alert] that was triggered from a [Target]
// #[derive(Debug, PartialEq, Eq, Clone)]
// pub struct TriggeredAlert {
//     pub alert: Alert,
//     pub timestamp: Option<Timestamp>,
//     pub name: String,
// }

// /// [Reminder] that was triggered from a [Target]
// #[derive(Debug, Clone)]
// pub struct TriggeredReminder {
//     pub reminder: Reminder,
//     pub name: String,
// }

// /// Reference to hold statements regarding [Alert] and [Reminder] queries
// pub struct AlertManager<'a> {
//     get_tag_apps: Statement<'a>,
//     triggered_alerts: Statement<'a>,
//     triggered_reminders: Statement<'a>,
//     insert_alert_event: Statement<'a>,
//     insert_reminder_event: Statement<'a>,
// }

// impl<'a> AlertManager<'a> {
//     /// Initialize a [AlertManager] from a given [Database]
//     pub fn new(db: &'a mut Database) -> Result<Self> {
//         let conn = &db.conn;
//         let get_tag_apps = prepare_stmt!(conn, "SELECT app_id FROM _app_tags WHERE tag_id = ?")?;
//         let triggered_alerts =
//             prepare_stmt!(conn, include_str!("../queries/triggered_alerts.sql"))?;
//         let triggered_reminders =
//             prepare_stmt!(conn, include_str!("../queries/triggered_reminders.sql"))?;
//         let insert_alert_event = insert_stmt!(conn, AlertEvent)?;
//         let insert_reminder_event = insert_stmt!(conn, ReminderEvent)?;
//         Ok(Self {
//             get_tag_apps,
//             triggered_alerts,
//             triggered_reminders,
//             insert_alert_event,
//             insert_reminder_event,
//         })
//     }

//     /// Gets all [App]s under the [Target]
//     pub fn target_apps(&mut self, target: &Target) -> Result<Vec<Ref<App>>> {
//         Ok(match target {
//             Target::Tag(tag) => self
//                 .get_tag_apps
//                 .query_map(params![tag], |row| row.get(0))?
//                 .collect::<Result<Vec<_>, _>>()?,
//             // this will only return one result, but we get a row iterator nonetheless
//             Target::App(app) => vec![app.clone()],
//         })
//     }

//     // TODO optim: use a single query to get all triggered alerts and reminders
//     // TODO optim: or use a single transaction for the below two.

//     /// Get all [Alert]s that are triggered, including when they were triggered
//     pub fn triggered_alerts(&mut self, times: &impl TimeSystem) -> Result<Vec<TriggeredAlert>> {
//         let day_start = times.day_start().to_ticks();
//         let week_start = times.week_start().to_ticks();
//         let month_start = times.month_start().to_ticks();
//         let result = self
//             .triggered_alerts
//             .query_map(params![day_start, week_start, month_start], |row| {
//                 Ok(TriggeredAlert {
//                     alert: Self::row_to_alert(row)?,
//                     timestamp: row.get(9)?,
//                     name: row.get(10)?,
//                 })
//             })?
//             .collect::<Result<Vec<_>, _>>()?;
//         Ok(result)
//     }

//     /// Get all [Reminder]s that are triggered, except those that are already handled
//     pub fn triggered_reminders(
//         &mut self,
//         times: &impl TimeSystem,
//     ) -> Result<Vec<TriggeredReminder>> {
//         let day_start = times.day_start().to_ticks();
//         let week_start = times.week_start().to_ticks();
//         let month_start = times.month_start().to_ticks();
//         let result = self
//             .triggered_reminders
//             .query_map(params![day_start, week_start, month_start], |row| {
//                 Ok(TriggeredReminder {
//                     reminder: Self::row_to_reminder(row)?,
//                     name: row.get(6)?,
//                 })
//             })?
//             .collect::<Result<Vec<_>, _>>()?;
//         Ok(result)
//     }

//     /// Insert a [AlertEvent]
//     pub fn insert_alert_event(&mut self, event: &AlertEvent) -> Result<()> {
//         self.insert_alert_event.execute(params![
//             event.alert.id,
//             event.alert.version,
//             event.timestamp,
//         ])?;
//         Ok(())
//     }

//     /// Insert a [ReminderEvent]
//     pub fn insert_reminder_event(&mut self, event: &ReminderEvent) -> Result<()> {
//         self.insert_reminder_event.execute(params![
//             event.reminder.id,
//             event.reminder.version,
//             event.timestamp,
//         ])?;
//         Ok(())
//     }

//     fn row_to_alert(row: &rusqlite::Row) -> Result<Alert, rusqlite::Error> {
//         let app_id: Option<Ref<App>> = row.get(2)?;
//         Ok(Alert {
//             id: Ref::new(VersionedId {
//                 id: row.get(0)?,
//                 version: row.get(1)?,
//             }),
//             target: if let Some(app_id) = app_id {
//                 Target::App(app_id)
//             } else {
//                 Target::Tag(row.get(3)?)
//             },
//             usage_limit: row.get(4)?,
//             time_frame: match row.get(5)? {
//                 0 => TimeFrame::Daily,
//                 1 => TimeFrame::Weekly,
//                 2 => TimeFrame::Monthly,
//                 _ => unreachable!("time frame"),
//             },
//             trigger_action: match row.get(8)? {
//                 0 => TriggerAction::Kill,
//                 1 => TriggerAction::Dim(row.get(6)?),
//                 2 => TriggerAction::Message(row.get(7)?),
//                 _ => unreachable!("trigger action"),
//             },
//         })
//     }

//     fn row_to_reminder(row: &rusqlite::Row) -> Result<Reminder, rusqlite::Error> {
//         Ok(Reminder {
//             id: Ref::new(VersionedId {
//                 id: row.get(0)?,
//                 version: row.get(1)?,
//             }),
//             alert: Ref::new(VersionedId {
//                 id: row.get(2)?,
//                 version: row.get(3)?,
//             }),
//             threshold: row.get(4)?,
//             message: row.get(5)?,
//         })
//     }
// }

#[cfg(test)]
mod tests;
