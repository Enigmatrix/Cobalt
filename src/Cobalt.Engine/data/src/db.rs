use rusqlite::{Connection, Statement};
use util::error::{Context, Result};

use crate::{
    entities::{InteractionPeriod, Session, Usage},
    migrations::Migrator,
};

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
    pub fn new(path: &str) -> Result<Self> {
        let conn = Connection::open(path).context("open conn")?;
        conn.pragma_update(None, "journal_mode", "WAL")
            .context("enable WAL")?;
        Migrator::new(&conn).migrate().context("migrate")?;
        Ok(Self { conn })
    }
}

pub struct UsageWriter<'a> {
    conn: &'a Connection,
    insert_session: Statement<'a>,
    insert_usage: Statement<'a>,
    insert_interaction_period: Statement<'a>,
}

impl<'a> UsageWriter<'a> {
    pub fn new(db: &'a Database) -> Result<Self> {
        let conn = &db.conn;
        let insert_session = insert_stmt!(conn, Session)?;
        let insert_usage = insert_stmt!(conn, Usage)?;
        let insert_interaction_period = insert_stmt!(conn, InteractionPeriod)?;
        // TODO prep stmts
        Ok(Self {
            conn,
            insert_session,
            insert_usage,
            insert_interaction_period,
        })
    }
}
