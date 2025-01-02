use std::io::Write;
use std::str::FromStr;

use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqliteRow};
use sqlx::{
    query, query_as, ConnectOptions, Connection, Executor, Row, Sqlite, SqliteConnection,
    Transaction,
};
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

/// Reference to hold statements regarding [Usage] and associated entities' queries
pub struct UsageWriter {
    db: Database,
}

impl UsageWriter {
    /// Initialize a [UsageWriter] from a given [Database]
    pub fn new(db: Database) -> Result<Self> {
        Ok(Self { db })
    }

    /// Find or insert a [App] by its [AppIdentity]
    pub async fn find_or_insert_app(
        &mut self,
        identity: &AppIdentity,
    ) -> Result<FoundOrInserted<App>> {
        let (tag, text0) = Self::destructure_identity(identity);
        let (app_id, found): (Ref<App>, bool) = query_as(
            // We set found=1 to force this query to return a result row regardless of
            // conflict result.
            "INSERT INTO apps (identity_is_win32, identity_path_or_aumid) VALUES (?, ?) ON CONFLICT
                DO UPDATE SET found = 1 RETURNING id, found",
        )
        .bind(tag)
        .bind(text0)
        .fetch_one(self.db.executor())
        .await?;

        Ok(if found {
            FoundOrInserted::Found(app_id)
        } else {
            FoundOrInserted::Inserted(app_id)
        })
    }

    /// Insert a [Session] into the [Database]
    pub async fn insert_session(&mut self, session: &mut Session) -> Result<()> {
        let res = query("INSERT INTO sessions VALUES (NULL, ?, ?)")
            .bind(session.app.clone())
            .bind(session.title.clone())
            .execute(self.db.executor())
            .await?;
        session.id = Ref::new(res.last_insert_rowid());
        Ok(())
    }

    /// Insert or update (only the end timestamp) a [Usage] into the [Database]
    pub async fn insert_or_update_usage(&mut self, usage: &mut Usage) -> Result<()> {
        if usage.id == Ref::default() {
            let res = query("INSERT INTO usages VALUES (NULL, ?, ?, ?)")
                .bind(usage.session.clone())
                .bind(usage.start.clone())
                .bind(usage.end.clone())
                .execute(self.db.executor())
                .await?;
            usage.id = Ref::new(res.last_insert_rowid());
        } else {
            query("UPDATE usages SET end = ? WHERE id = ?")
                .bind(usage.end.clone())
                .bind(usage.id.clone())
                .execute(self.db.executor())
                .await?;
        }
        Ok(())
    }

    /// Insert a [InteractionPeriod] into the [Database]
    pub async fn insert_interaction_period(
        &mut self,
        interaction_period: &InteractionPeriod,
    ) -> Result<()> {
        query("INSERT INTO interaction_periods VALUES (NULL, ?, ?, ?, ?)")
            .bind(interaction_period.start.clone())
            .bind(interaction_period.end.clone())
            .bind(interaction_period.mouse_clicks.clone())
            .bind(interaction_period.key_strokes.clone())
            .execute(self.db.executor())
            .await?;
        Ok(())
    }

    fn destructure_identity(identity: &AppIdentity) -> (i64, &str) {
        match identity {
            AppIdentity::Win32 { path } => (1, path),
            AppIdentity::Uwp { aumid } => (0, aumid),
        }
    }
}

/// Reference to hold statements regarding [App] updates
pub struct AppUpdater {
    db: Database,
}

impl AppUpdater {
    /// Initialize a [AppUpdater] from a given [Database]
    pub fn new(db: Database) -> Result<Self> {
        Ok(Self { db })
    }

    /// Update the [App] with additional information
    pub async fn update_app(&mut self, app: &App) -> Result<()> {
        query(
            "UPDATE apps SET
                    name = ?,
                    description = ?,
                    company = ?,
                    color = ?,
                    initialized = 1
                WHERE id = ?",
        )
        .bind(app.name.clone())
        .bind(app.description.clone())
        .bind(app.company.clone())
        .bind(app.color.clone())
        .bind(app.id.clone())
        .execute(self.db.executor())
        .await?;
        Ok(())
    }

    /// Get a reference to the [App] icon's writer
    pub async fn update_app_icon(&mut self, app_id: Ref<App>, icon: &[u8]) -> Result<()> {
        query(
            "UPDATE apps SET
                    icon = ?
                WHERE id = ?",
        )
        .bind(icon)
        .bind(app_id)
        .execute(self.db.executor())
        .await?;
        Ok(())
    }
}

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
