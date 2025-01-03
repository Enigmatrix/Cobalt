use std::time::Duration;

use sqlx::prelude::FromRow;
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
    Target, Timestamp, Usage,
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
            .log_statements(log::LevelFilter::Trace)
            .log_slow_statements(log::LevelFilter::Info, Duration::from_secs(1))
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
        self.conn.begin().await.context("begin transaction")
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
            .bind(session.app_id.clone())
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
                .bind(usage.session_id.clone())
                .bind(usage.start)
                .bind(usage.end)
                .execute(self.db.executor())
                .await?;
            usage.id = Ref::new(res.last_insert_rowid());
        } else {
            query("UPDATE usages SET end = ? WHERE id = ?")
                .bind(usage.end)
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
            .bind(interaction_period.start)
            .bind(interaction_period.end)
            .bind(interaction_period.mouse_clicks)
            .bind(interaction_period.key_strokes)
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

/// [Alert] that was triggered from a [Target]
#[derive(Debug, PartialEq, Eq, Clone, FromRow)]
pub struct TriggeredAlert {
    #[sqlx(flatten)]
    pub alert: Alert,
    pub timestamp: Option<Timestamp>,
    pub name: String,
}

/// [Reminder] that was triggered from a [Target]
#[derive(Debug, Clone, FromRow)]
pub struct TriggeredReminder {
    #[sqlx(flatten)]
    pub reminder: Reminder,
    pub name: String,
}

/// Reference to hold statements regarding [Alert] and [Reminder] queries
pub struct AlertManager {
    db: Database,
}

impl AlertManager {
    /// Initialize a [AlertManager] from a given [Database]
    pub fn new(db: Database) -> Result<Self> {
        Ok(Self { db })
    }

    /// Gets all [App]s under the [Target]
    pub async fn target_apps(&mut self, target: &Target) -> Result<Vec<Ref<App>>> {
        Ok(match target {
            Target::Tag(tag) => {
                query("SELECT app_id FROM _app_tags WHERE tag_id = ?")
                    .bind(tag.clone())
                    .map(|r: SqliteRow| r.get(0))
                    .fetch_all(self.db.executor())
                    .await?
            }
            // this will only return one result, but we get a row iterator nonetheless
            Target::App(app) => vec![app.clone()],
        })
    }

    // TODO optim: use a single query to get all triggered alerts and reminders
    // TODO optim: or use a single transaction for the below two.

    /// Get all [Alert]s that are triggered, including when they were triggered
    pub async fn triggered_alerts(
        &mut self,
        times: &impl TimeSystem,
    ) -> Result<Vec<TriggeredAlert>> {
        let day_start = times.day_start().to_ticks() as i64;
        let week_start = times.week_start().to_ticks() as i64;
        let month_start = times.month_start().to_ticks() as i64;
        let result = query_as(include_str!("../queries/triggered_alerts.sql"))
            .bind(day_start)
            .bind(week_start)
            .bind(month_start)
            .fetch_all(self.db.executor())
            .await?;
        Ok(result)
    }

    /// Get all [Reminder]s that are triggered, except those that are already handled
    pub async fn triggered_reminders(
        &mut self,
        times: &impl TimeSystem,
    ) -> Result<Vec<TriggeredReminder>> {
        let day_start = times.day_start().to_ticks() as i64;
        let week_start = times.week_start().to_ticks() as i64;
        let month_start = times.month_start().to_ticks() as i64;
        let result = query_as(include_str!("../queries/triggered_reminders.sql"))
            .bind(day_start)
            .bind(week_start)
            .bind(month_start)
            .fetch_all(self.db.executor())
            .await?;
        Ok(result)
    }

    /// Insert a [AlertEvent]
    pub async fn insert_alert_event(&mut self, event: &AlertEvent) -> Result<()> {
        query("INSERT INTO alert_events VALUES (NULL, ?, ?, ?)")
            .bind(event.alert.id)
            .bind(event.alert.version)
            .bind(event.timestamp)
            .execute(self.db.executor())
            .await?;
        Ok(())
    }

    /// Insert a [ReminderEvent]
    pub async fn insert_reminder_event(&mut self, event: &ReminderEvent) -> Result<()> {
        query("INSERT INTO reminder_events VALUES (NULL, ?, ?, ?)")
            .bind(event.reminder.id)
            .bind(event.reminder.version)
            .bind(event.timestamp)
            .execute(self.db.executor())
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests;
