use super::Migration;
use rusqlite::{params, Connection};
use util::error::{bail, Context, Result};

pub struct Migration1;

impl Migration for Migration1 {
    fn version(&self) -> u64 {
        1
    }

    fn up(&self, conn: &mut Connection) -> Result<()> {
        let tx = conn.transaction().context("create transaction")?;

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
            params![],
        )
        .context("create table apps")?;

        // cmd_line can be NULL
        tx.execute(
            "CREATE TABLE sessions (
            id                              INTEGER PRIMARY KEY NOT NULL,
            app_id                          INTEGER NOT NULL REFERENCES apps(id),
            title                           TEXT NOT NULL
        )",
            params![],
        )
        .context("create table sessions")?;

        tx.execute(
            "CREATE TABLE usages (
            id                              INTEGER PRIMARY KEY NOT NULL,
            session_id                      INTEGER NOT NULL REFERENCES sessions(id),
            start                           INTEGER NOT NULL,
            end                             INTEGER NOT NULL
        )",
            params![],
        )
        .context("create table usages")?;

        tx.execute(
            "CREATE TABLE interaction_periods (
            id                              INTEGER PRIMARY KEY NOT NULL,
            start                           INTEGER NOT NULL,
            end                             INTEGER NOT NULL,
            mouseclicks                     INTEGER NOT NULL,
            keystrokes                      INTEGER NOT NULL
        )",
            params![],
        )
        .context("create table interaction_periods")?;

        tx.execute(
            "CREATE TABLE tags (
            id                              INTEGER PRIMARY KEY NOT NULL,
            name                            TEXT NOT NULL,
            color                           TEXT
        )",
            params![],
        )
        .context("create table tags")?;

        tx.execute(
            "CREATE TABLE _app_tags (
            app_id                          INTEGER NOT NULL REFERENCES apps(id),
            tag_id                          INTEGER NOT NULL REFERENCES tags(id),
            PRIMARY KEY (app_id, tag_id)
        )",
            params![],
        )
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
            params![],
        )
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
        )",
            params![],
        )
        .context("create table alerts")?;

        tx.execute(
            "CREATE TABLE alert_events (
            id                              INTEGER PRIMARY KEY NOT NULL,
            alert_id                        INTEGER NOT NULL,
            alert_version                   INTEGER NOT NULL,
            timestamp                       INTEGER NOT NULL,
            FOREIGN KEY (alert_id, alert_version) REFERENCES alerts(id, version)
        )",
            params![],
        )
        .context("create table alert_events")?;

        tx.execute(
            "CREATE TABLE reminder_events (
            id                              INTEGER PRIMARY KEY NOT NULL,
            reminder_id                     INTEGER NOT NULL,
            reminder_version                INTEGER NOT NULL,
            timestamp                       INTEGER NOT NULL,
            FOREIGN KEY (reminder_id, reminder_version) REFERENCES reminders(id, version)
        )",
            params![],
        )
        .context("create table reminder_events")?;

        tx.execute(
            "CREATE TABLE alert_id_seq (
            id                              INTEGER PRIMARY KEY NOT NULL
        );
        INSERT INTO alert_id_seq (id) VALUES (1)",
            params![],
        )
        .context("create table alert_id_seq")?;

        tx.execute(
            "CREATE TABLE reminder_id_seq (
            id                              INTEGER PRIMARY KEY NOT NULL
        );
        INSERT INTO reminder_id_seq (id) VALUES (1)",
            params![],
        )
        .context("create table reminder_id_seq")?;

        tx.execute(
            "CREATE INDEX usage_start_end ON usages(start ASC, end DESC)",
            params![],
        )
        .context("create index usage_start_end")?;

        tx.execute(
            "CREATE INDEX interaction_period_start_end ON interaction_periods(start ASC, end DESC)",
            params![],
        )
        .context("create index interaction_period")?;

        tx.execute(
            "CREATE UNIQUE INDEX app_identity ON apps(identity_is_win32, identity_path_or_aumid)",
            params![],
        )
        .context("create unique index app_identity")?;

        tx.commit().context("commit transaction")?;
        Ok(())
    }

    fn down(&self, _conn: &mut Connection) -> Result<()> {
        bail!("down on base migration")
    }
}

#[test]
fn test_up() -> Result<()> {
    let mut conn = Connection::open_in_memory()?;
    let m1 = Migration1;
    m1.up(&mut conn)?;
    Ok(())
}
