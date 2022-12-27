use std::cmp;

use rusqlite::Connection;
use rusqlite::Error as SqliteError;
use utils::errors::*;
use utils::tracing::debug;

pub trait Migration {
    fn version(&self) -> u64;

    fn up(&mut self, conn: &mut Connection) -> Result<()>;
    fn down(&mut self, conn: &mut Connection) -> Result<()>;
}

impl PartialEq<dyn Migration> for dyn Migration {
    fn eq(&self, other: &dyn Migration) -> bool {
        self.version().eq(&other.version())
    }
}

impl Eq for dyn Migration {}

impl PartialOrd<dyn Migration> for dyn Migration {
    fn partial_cmp(&self, other: &dyn Migration) -> Option<cmp::Ordering> {
        self.version().partial_cmp(&other.version())
    }
}

impl Ord for dyn Migration {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.version().cmp(&other.version())
    }
}

pub struct Migrator<'a> {
    conn: &'a mut Connection,
    migrations: Vec<Box<dyn Migration>>,
}

impl<'a> Migrator<'a> {
    pub fn new(conn: &'a mut Connection, mut migrations: Vec<Box<dyn Migration>>) -> Self {
        migrations.sort();
        Migrator { conn, migrations }
    }

    pub fn get_current_version(&self) -> Result<u64> {
        match self
            .conn
            .query_row("SELECT Version FROM Migration", [], |row| row.get(0))
        {
            Ok(v) => Ok(v),
            Err(SqliteError::QueryReturnedNoRows) => Ok(0),
            Err(SqliteError::SqliteFailure(_, Some(ref v))) if v == "no such table: Migration" => {
                Ok(0)
            }
            Err(e) => Err(e).context("get current version query"),
        }
    }

    pub fn migrate(&mut self) -> Result<()> {
        let current_version = self.get_current_version().context("get current version")?;
        for m in self
            .migrations
            .iter_mut()
            .skip_while(|m| m.version() <= current_version)
        {
            let ver = m.version();
            debug!("running migration {}", ver);
            m.up(self.conn)
                .with_context(|| format!("error running migration {}", ver))?;
        }
        Ok(())
    }
}
