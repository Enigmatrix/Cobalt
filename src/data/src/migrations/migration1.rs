use async_trait::async_trait;
use sqlx::Executor;
use util::error::{Context, Result, bail};

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

        tx.execute(
            "CREATE TABLE tags (
                id                              INTEGER PRIMARY KEY NOT NULL,
                name                            TEXT NOT NULL,
                color                           TEXT NOT NULL,
                score                           REAL NOT NULL DEFAULT 0 CHECK (score >= -100 AND score <= 100),
                created_at                      INTEGER NOT NULL,
                updated_at                      INTEGER NOT NULL
            )",
        )
        .await
        .context("create table tags")?;

        tx.execute(
            "CREATE TABLE apps (
                id                              INTEGER PRIMARY KEY NOT NULL,
                name                            TEXT NOT NULL,
                description                     TEXT NOT NULL,
                company                         TEXT NOT NULL,
                color                           TEXT NOT NULL,
                tag_id                          INTEGER REFERENCES tags(id) ON DELETE SET NULL,
                identity_tag                    INTEGER NOT NULL,
                identity_text0                  TEXT NOT NULL,
                identity_text1                  TEXT NOT NULL,
                created_at                      INTEGER NOT NULL,
                initialized_at                  INTEGER,
                updated_at                      INTEGER NOT NULL
            )",
        )
        .await
        .context("create table apps")?;

        // Store icons in a separate table to avoid bloating the apps table
        // with large blobs. This speeds up queries that don't need the icon.
        tx.execute(
            "CREATE TABLE app_icons (
                id                              INTEGER PRIMARY KEY NOT NULL REFERENCES apps(id) ON DELETE CASCADE,
                icon                            BLOB NOT NULL
            )",
        )
        .await
        .context("create table app_icons")?;

        // cmd_line can be NULL
        tx.execute(
            "CREATE TABLE sessions (
                id                              INTEGER PRIMARY KEY NOT NULL,
                app_id                          INTEGER NOT NULL REFERENCES apps(id),
                title                           TEXT NOT NULL,
                url                             TEXT
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
            "CREATE TABLE system_events (
                id                              INTEGER PRIMARY KEY NOT NULL,
                timestamp                       INTEGER NOT NULL,
                event                           INTEGER NOT NULL
            )",
        )
        .await
        .context("create table system_events")?;

        tx.execute(
            "CREATE TABLE alerts (
                id                              INTEGER PRIMARY KEY NOT NULL,
                app_id                          INTEGER REFERENCES apps(id) ON DELETE CASCADE,
                tag_id                          INTEGER REFERENCES tags(id) ON DELETE CASCADE,
                usage_limit                     INTEGER NOT NULL,
                time_frame                      INTEGER NOT NULL,
                trigger_action_dim_duration     INTEGER,
                trigger_action_message_content  TEXT,
                trigger_action_tag              INTEGER NOT NULL,
                active                          TINYINT NOT NULL DEFAULT TRUE,
                created_at                      INTEGER NOT NULL,
                updated_at                      INTEGER NOT NULL
            )",
        )
        .await
        .context("create table alerts")?;

        tx.execute(
            "CREATE TABLE reminders (
                id                              INTEGER PRIMARY KEY NOT NULL,
                alert_id                        INTEGER NOT NULL REFERENCES alerts(id) ON DELETE CASCADE,
                threshold                       REAL NOT NULL,
                message                         TEXT NOT NULL,
                active                          TINYINT NOT NULL DEFAULT TRUE,
                created_at                      INTEGER NOT NULL,
                updated_at                      INTEGER NOT NULL
            )",
        )
        .await
        .context("create table alerts")?;

        tx.execute(
            "CREATE TABLE alert_events (
                id                              INTEGER PRIMARY KEY NOT NULL,
                alert_id                        INTEGER NOT NULL REFERENCES alerts(id) ON DELETE CASCADE,
                timestamp                       INTEGER NOT NULL,
                reason                          INTEGER NOT NULL
            )",
        )
        .await
        .context("create table alert_events")?;

        tx.execute(
            "CREATE TABLE reminder_events (
                id                              INTEGER PRIMARY KEY NOT NULL,
                reminder_id                     INTEGER NOT NULL REFERENCES reminders(id) ON DELETE CASCADE,
                timestamp                       INTEGER NOT NULL,
                reason                          INTEGER NOT NULL
            )",
        )
        .await
        .context("create table reminder_events")?;

        tx.execute("CREATE INDEX usage_start_session_end ON usages(start, session_id, end)")
            .await
            .context("create index usage_start_end")?;

        tx.execute("CREATE INDEX usage_end_session_start ON usages(end, session_id, start)")
            .await
            .context("create index usage_start_end")?;

        tx.execute("CREATE INDEX session_app_id ON sessions(id, app_id)")
            .await
            .context("create index session_app_id")?;

        tx.execute("CREATE INDEX session_id_app ON sessions(app_id, id)")
            .await
            .context("create index session_id_app")?;

        tx.execute("CREATE INDEX interaction_period_start_end ON interaction_periods(start, end)")
            .await
            .context("create index interaction_period")?;

        tx.execute("CREATE UNIQUE INDEX app_identity ON apps(identity_tag, identity_text0, identity_text1)")
            .await
            .context("create unique index app_identity")?;

        tx.commit().await.context("commit transaction")?;
        Ok(())
    }

    async fn down(&self, _db: &mut Database) -> Result<()> {
        bail!("down on base migration")
    }
}
