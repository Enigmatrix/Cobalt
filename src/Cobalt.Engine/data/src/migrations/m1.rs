use common::errors::*;
use rusqlite::{params, Connection};

use crate::migrator::Migration;

pub(crate) struct Migration1;

impl Migration for Migration1 {
    fn version(&self) -> u64 {
        1
    }

    fn up(&mut self, conn: &mut Connection) -> Result<()> {
        let tx = conn.transaction().context("create transaction")?;

        // TODO make color non-null

        // all fields of app are nullable, except identity
        tx.execute(
            "CREATE TABLE app (
            id              INTEGER PRIMARY KEY NOT NULL,
            initialized     TINYINT NOT NULL DEFAULT FALSE,
            found           TINYINT NOT NULL DEFAULT FALSE,
            name            TEXT,
            description     TEXT,
            company         TEXT,
            color           TEXT,
            identity_tag    INTEGER NOT NULL,
            identity_text0  TEXT NOT NULL,
            icon            BLOB
        )",
            params![],
        )
        .context("create table app")?;

        // cmd_line can be NULL
        tx.execute(
            "CREATE TABLE session (
            id              INTEGER PRIMARY KEY NOT NULL,
            app             INTEGER NOT NULL REFERENCES app(id),
            title           TEXT NOT NULL,
            cmd_line        TEXT
        )",
            params![],
        )
        .context("create table session")?;

        tx.execute(
            "CREATE TABLE usage (
            id              INTEGER PRIMARY KEY NOT NULL,
            session         INTEGER NOT NULL REFERENCES session(id),
            start           INTEGER NOT NULL,
            end             INTEGER NOT NULL
        )",
            params![],
        )
        .context("create table usage")?;

        tx.execute(
            "CREATE TABLE interaction_period (
            id              INTEGER PRIMARY KEY NOT NULL,
            start           INTEGER NOT NULL,
            end             INTEGER NOT NULL,
            mouseclicks     INTEGER NOT NULL,
            keystrokes      INTEGER NOT NULL
        )",
            params![],
        )
        .context("create table interaction_period")?;

        tx.execute(
            "CREATE TABLE tag (
            id              INTEGER PRIMARY KEY NOT NULL,
            name            TEXT NOT NULL,
            color           TEXT
        )",
            params![],
        )
        .context("create table tag")?;

        tx.execute(
            "CREATE TABLE _app_tag (
            app             INTEGER NOT NULL REFERENCES app(id),
            tag             INTEGER NOT NULL REFERENCES tag(id),
            PRIMARY KEY (app, tag)
        )",
            params![],
        )
        .context("create table _app_tag")?;

        tx.execute(
            "CREATE TABLE alert (
            id              INTEGER PRIMARY KEY NOT NULL,
            target_is_app   TINYINT NOT NULL,
            app             INTEGER REFERENCES app(id),
            tag             INTEGER REFERENCES tag(id),
            usage_limit     INTEGER NOT NULL,
            time_frame      INTEGER NOT NULL,
            action_tag      INTEGER NOT NULL,
            action_int0     INTEGER,
            action_text0    TEXT
        )",
            params![],
        )
        .context("create table alert")?;

        tx.execute(
            "CREATE TABLE reminder (
            id              INTEGER PRIMARY KEY NOT NULL,
            alert           INTEGER REFERENCES alert(id),
            threshold       REAL NOT NULL,
            message         TEXT NOT NULL
        )",
            params![],
        )
        .context("create table alert")?;

        tx.execute(
            "CREATE INDEX usage_start_end ON usage (start ASC, end DESC)",
            params![],
        )
        .context("create index usage_start_end")?;

        tx.execute(
            "CREATE INDEX interaction_period_start_end ON interaction_period (start ASC, end DESC)",
            params![],
        )
        .context("create index interaction_period")?;

        tx.execute(
            "CREATE UNIQUE INDEX app_identity ON app (identity_tag, identity_text0)",
            params![],
        )
        .context("create unique index app_identity")?;

        tx.commit().context("commit transaction")?;
        Ok(())
    }

    fn down(&mut self, _conn: &mut Connection) -> Result<()> {
        bail!("cannot down the base migration")
    }
}
