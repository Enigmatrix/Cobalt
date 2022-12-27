use rusqlite::{Connection, Statement};
use utils::errors::*;

use crate::{migrations::default_migrations, migrator::Migrator, models};

macro_rules! prepare_stmt {
    ($conn: expr, $sql:expr) => {{
        let sql = $sql;
        $conn
            .prepare(&sql)
            .with_context(|| format!("prepare stmt: {sql}"))
    }};
}

macro_rules! insert_stmt {
    ($conn: expr, $tbl:expr, $cols:expr) => {{
        let sql = format!(
            "INSERT INTO {} VALUES (NULL{})",
            $tbl,
            ", ?".repeat($cols.len())
        );
        prepare_stmt!($conn, sql)
    }};
}

pub struct Database<'a> {
    pub conn: &'a Connection,
    pub find_app_stmt: Statement<'a>,
    pub insert_app_stmt: Statement<'a>,
    pub insert_session_stmt: Statement<'a>,
    pub insert_usage_stmt: Statement<'a>,
    pub insert_interaction_period_stmt: Statement<'a>,
}

impl<'a> Database<'a> {
    pub fn insert_app(&self, app: &mut models::App) {
        // self.insert_app_stmt
        todo!()
    }
}

pub struct DatabaseHolder {
    conn: Connection,
}

impl DatabaseHolder {
    // we borrow as mutably here so that no two databases can exist at the same time
    pub fn database(&mut self) -> Result<Database<'_>> {
        let conn = &self.conn;

        // TODO update app statement
        // TODO columns
        // TODO migrations
        // TODO insert null App (just to get the id)

        Ok(Database {
            conn,
            find_app_stmt: prepare_stmt!(
                conn,
                "SELECT Id FROM App WHERE Identity_Tag = ? AND Identity_Text1 = ?"
            )?,
            insert_app_stmt: insert_stmt!(conn, "App", ["", ""]).context("prep app insert stmt")?,
            insert_session_stmt: insert_stmt!(conn, "Session", ["", ""])
                .context("prep session insert stmt")?,
            insert_usage_stmt: insert_stmt!(conn, "Usage", ["", ""])
                .context("prep usage insert stmt")?,
            insert_interaction_period_stmt: insert_stmt!(conn, "InteractionPeriod", ["", ""])
                .context("interaction period insert stmt")?,
        })
    }
}

impl DatabaseHolder {
    pub fn new(conn_str: &str) -> Result<DatabaseHolder> {
        //TODO the other options can disable mutexes
        let mut conn = Connection::open(conn_str).context("open db using conn str")?;
        // TODO enable fkey
        // TODO turn on WAL

        // run migration
        Migrator::new(&mut conn, default_migrations())
            .migrate()
            .context("run default migration")?;

        Ok(DatabaseHolder { conn })
    }
}
