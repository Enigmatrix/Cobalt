use std::cmp;

use common::{errors::*, tracing::debug};
use rusqlite::{Connection, Error as SqliteError};

use crate::{db::Database, migrations::default_migrations};

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
    pub fn new(db: &'a mut Database) -> Self {
        let mut migrations = default_migrations();
        migrations.sort();
        Migrator {
            conn: &mut db.conn,
            migrations,
        }
    }

    pub fn get_current_version(&self) -> Result<u64> {
        let version = self
            .conn
            .pragma_query_value(None, "user_version", |r| r.get(0));
        match version {
            Ok(v) => Ok(v),
            Err(SqliteError::QueryReturnedNoRows) => Ok(0),
            Err(e) => Err(e).context("get migration version query"),
        }
    }

    fn set_current_version(&self, version: u64) -> Result<()> {
        self.conn
            .pragma_update(None, "user_version", version)
            .context("set migration version")?;
        Ok(())
    }

    pub fn migrate(&mut self) -> Result<()> {
        let current_version = self.get_current_version().context("get current version")?;
        let mut new_version = None;

        for m in self
            .migrations
            .iter_mut()
            .skip_while(|m| m.version() <= current_version)
        {
            let ver = m.version();
            debug!("running migration {ver}");
            m.up(self.conn)
                .with_context(|| format!("error running migration {ver}"))?;

            let new_version = new_version.get_or_insert(ver);
            *new_version = (*new_version).max(ver);
        }

        if let Some(new_version) = new_version {
            self.set_current_version(new_version)
                .context("set current version")?;
        }

        Ok(())
    }
}
