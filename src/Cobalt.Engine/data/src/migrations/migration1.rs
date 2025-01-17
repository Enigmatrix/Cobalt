use async_trait::async_trait;
use sqlx::Executor;
use util::error::{bail, Context, Result};

use super::Migration;
use crate::db::Database;

/// Initial database schema - define the basic entities, relations and indexes.
pub struct Migration1;

#[async_trait]
impl Migration for Migration1 {
    fn version(&self) -> i64 {
        1
    }

    async fn up(&self, db: &mut Database) -> Result<()> {
        let mut tx = db.transaction().await?;

        // all information fields of app are nullable, except identity
        tx.execute(
            "CREATE TABLE apps (
            id                              INTEGER PRIMARY KEY NOT NULL,
            initialized                     TINYINT NOT NULL DEFAULT FALSE,
            found                           TINYINT NOT NULL DEFAULT FALSE,
            name                            TEXT,
            description                     TEXT,
            company                         TEXT,
            color                           TEXT,
            identity_is_win32               INTEGER NOT NULL,
            identity_path_or_aumid          TEXT NOT NULL,
            icon                            BLOB
        )",
        )
        .await
        .context("create table apps")?;

        // cmd_line can be NULL
        tx.execute(
            "CREATE TABLE sessions (
            id                              INTEGER PRIMARY KEY NOT NULL,
            app_id                          INTEGER NOT NULL REFERENCES apps(id),
            title                           TEXT NOT NULL
        )",
        )
        .await
        .context("create table sessions")?;

        tx.execute(
            "CREATE TABLE usages (
            id                              INTEGER PRIMARY KEY NOT NULL,
            session_id                      INTEGER NOT NULL REFERENCES sessions(id),
            start                           INTEGER NOT NULL,
            end                             INTEGER NOT NULL
        )",
        )
        .await
        .context("create table usages")?;

        tx.execute(
            "CREATE TABLE interaction_periods (
            id                              INTEGER PRIMARY KEY NOT NULL,
            start                           INTEGER NOT NULL,
            end                             INTEGER NOT NULL,
            mouse_clicks                    INTEGER NOT NULL,
            key_strokes                     INTEGER NOT NULL
        )",
        )
        .await
        .context("create table interaction_periods")?;

        tx.execute(
            "CREATE TABLE tags (
            id                              INTEGER PRIMARY KEY NOT NULL,
            name                            TEXT NOT NULL,
            color                           TEXT NOT NULL
        )",
        )
        .await
        .context("create table tags")?;

        tx.execute(
            "CREATE TABLE _app_tags (
            app_id                          INTEGER NOT NULL REFERENCES apps(id),
            tag_id                          INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
            PRIMARY KEY (app_id, tag_id)
        )",
        )
        .await
        .context("create table _app_tags")?;

        tx.execute(
            "CREATE TABLE alerts (
            id                              INTEGER NOT NULL,
            version                         INTEGER NOT NULL,
            app_id                          INTEGER REFERENCES apps(id),
            tag_id                          INTEGER REFERENCES tags(id),
            usage_limit                     INTEGER NOT NULL,
            time_frame                      INTEGER NOT NULL,
            trigger_action_dim_duration     INTEGER,
            trigger_action_message_content  TEXT,
            trigger_action_tag              INTEGER NOT NULL,
            PRIMARY KEY (id, version)
        )",
        )
        .await
        .context("create table alerts")?;

        tx.execute(
            "CREATE TABLE reminders (
            id                              INTEGER NOT NULL,
            version                         INTEGER NOT NULL,
            alert_id                        INTEGER NOT NULL,
            alert_version                   INTEGER NOT NULL,
            threshold                       REAL NOT NULL,
            message                         TEXT NOT NULL,
            PRIMARY KEY (id, version),
            FOREIGN KEY (alert_id, alert_version) REFERENCES alerts(id, version)
                ON DELETE CASCADE
        )",
        )
        .await
        .context("create table alerts")?;

        tx.execute(
            "CREATE TABLE alert_events (
            id                              INTEGER PRIMARY KEY NOT NULL,
            alert_id                        INTEGER NOT NULL,
            alert_version                   INTEGER NOT NULL,
            timestamp                       INTEGER NOT NULL,
            FOREIGN KEY (alert_id, alert_version) REFERENCES alerts(id, version)
                ON DELETE CASCADE
        )",
        )
        .await
        .context("create table alert_events")?;

        tx.execute(
            "CREATE TABLE reminder_events (
            id                              INTEGER PRIMARY KEY NOT NULL,
            reminder_id                     INTEGER NOT NULL,
            reminder_version                INTEGER NOT NULL,
            timestamp                       INTEGER NOT NULL,
            FOREIGN KEY (reminder_id, reminder_version) REFERENCES reminders(id, version)
                ON DELETE CASCADE
        )",
        )
        .await
        .context("create table reminder_events")?;

        tx.execute(
            "CREATE TABLE alert_id_seq (
            id                              INTEGER PRIMARY KEY NOT NULL
        )",
        )
        .await
        .context("create table alert_id_seq")?;

        tx.execute("INSERT INTO alert_id_seq (id) VALUES (1)")
            .await
            .context("init alert_id_seq")?;

        tx.execute(
            "CREATE TABLE reminder_id_seq (
            id                              INTEGER PRIMARY KEY NOT NULL
        )",
        )
        .await
        .context("create table reminder_id_seq")?;

        tx.execute("INSERT INTO reminder_id_seq (id) VALUES (1)")
            .await
            .context("init reminder_id_seq")?;

        // Even though sorting by start is the same as sorting by end and vice versa
        // we need both fields so that these are covering indexes.

        tx.execute("CREATE INDEX usage_start_end ON usages(session_id, start, end)")
            .await
            .context("create index usage_start_end")?;

        tx.execute("CREATE INDEX usage_end_start ON usages(session_id, end, start)")
            .await
            .context("create index usage_start_end")?;

        tx.execute("CREATE INDEX interaction_period_start_end ON interaction_periods(start, end)")
            .await
            .context("create index interaction_period")?;

        tx.execute(
            "CREATE UNIQUE INDEX app_identity ON apps(identity_is_win32, identity_path_or_aumid)",
        )
        .await
        .context("create unique index app_identity")?;

        tx.execute("CREATE INDEX alert_event_fks_alert ON alert_events(alert_id, alert_version)")
            .await
            .context("create index alert_event_fks_alert")?;

        tx.execute(
            "CREATE INDEX reminder_event_fks_reminder ON reminder_events(reminder_id, reminder_version)")
        .await
        .context("create index reminder_event_fks_reminder")?;

        tx.commit().await.context("commit transaction")?;
        Ok(())
    }

    async fn down(&self, _db: &mut Database) -> Result<()> {
        bail!("down on base migration")
    }
}
