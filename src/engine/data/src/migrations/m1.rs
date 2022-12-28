use crate::migrator::Migration;
use rusqlite::{params, Connection};

use utils::errors::*;

pub struct Migration1;

impl Migration for Migration1 {
    fn version(&self) -> u64 {
        1
    }

    fn up(&mut self, conn: &mut Connection) -> Result<()> {
        let tx = conn.transaction().context("create transaction")?;

        // all fields of app are nullable
        tx.execute(
            "CREATE TABLE app (
            id              INTEGER PRIMARY KEY NOT NULL,
            name            TEXT,
            description     TEXT,
            company         TEXT,
            color           TEXT,
            identity_tag    INTEGER,
            identity_text0  TEXT
        )",
            params![],
        )
        .context("create table app")?;

        // cmd_line can be NULL
        tx.execute(
            "CREATE TABLE session (
            id              INTEGER PRIMARY KEY NOT NULL,
            app             INTEGER NOT NULL,
            title           TEXT NOT NULL,
            cmd_line        TEXT,
            FOREIGN KEY(app) references app(id)
        )",
            params![],
        )
        .context("create table session")?;

        tx.execute(
            "CREATE TABLE usage (
            id              INTEGER PRIMARY KEY NOT NULL,
            session         INTEGER NOT NULL,
            start           INTEGER NOT NULL,
            end             INTEGER NOT NULL,
            FOREIGN KEY(session) references session(id)
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
            name            TEXT,
            color           TEXT
        )",
            params![],
        )
        .context("create table tag")?;

        tx.execute(
            "CREATE TABLE _app_tag (
            app             INTEGER NOT NULL,
            session         INTEGER NOT NULL,
            FOREIGN KEY(app) references app(id),
            FOREIGN KEY(session) references session(id),
            PRIMARY KEY (app, session)
        )",
            params![],
        )
        .context("create table _app_tag")?;

        tx.execute(
            "CREATE TABLE migration (
            version     INTEGER NOT NULL
        )",
            params![],
        )
        .context("create table migration")?;

        //TODO insert alert

        tx.execute(
            "CREATE INDEX usage_start_end ON usage (start ASC, end ASC)",
            params![],
        )
        .context("create index usage_start_end")?;

        tx.execute(
            "CREATE INDEX interaction_period_start_end ON interaction_period (start ASC, end ASC)",
            params![],
        )
        .context("create index interaction_period")?;

        tx.execute(
            "CREATE UNIQUE INDEX app_identity ON app (identity_tag, identity_text0)
                WHERE identity_tag IS NOT NULL AND identity_text0 IS NOT NULL",
            params![],
        )
        .context("create unique index app_identity")?;

        tx.execute("INSERT INTO migration VALUES (0)", params![])
            .context("insert default migration")?;

        tx.commit().context("commit transaction")?;
        Ok(())
    }

    fn down(&mut self, _conn: &mut Connection) -> Result<()> {
        bail!("cannot down the base migration")
    }
}
