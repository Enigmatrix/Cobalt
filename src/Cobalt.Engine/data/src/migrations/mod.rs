use rusqlite::{Connection, Error};
use util::error::{Context, Result};

mod migration1;

pub trait Migration {
    fn version(&self) -> u64;
    fn up(&self, conn: &Connection) -> Result<()>;
    #[allow(dead_code)]
    fn down(&self, conn: &Connection) -> Result<()>;
}

pub struct Migrator<'a> {
    conn: &'a Connection,
    migrations: Vec<Box<dyn Migration>>,
}

impl<'a> Migrator<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        let mut migrations: Vec<Box<dyn Migration>> = vec![Box::new(migration1::Migration1)];
        // should already by sorted, but just in case
        migrations.sort_by_key(|migration| migration.version());
        Self { conn, migrations }
    }

    pub fn current_version(&self) -> Result<u64> {
        let version = self
            .conn
            .pragma_query_value(None, "user_version", |r| r.get(0));
        match version {
            Ok(v) => Ok(v),
            Err(Error::QueryReturnedNoRows) => Ok(0),
            Err(e) => Err(e).context("get current version"),
        }
    }

    pub fn set_current_version(&self, version: u64) -> Result<()> {
        self.conn
            .pragma_update(None, "user_version", version)
            .context("set current version")?;
        Ok(())
    }

    pub fn migrate(&self) -> Result<()> {
        let current_version = self.current_version()?;

        for migration in &self.migrations {
            let version = migration.version();
            if version > current_version {
                migration
                    .up(self.conn)
                    .with_context(|| format!("migration {version} up"))?;
                self.set_current_version(version)?;
            }
        }
        Ok(())
    }
}
