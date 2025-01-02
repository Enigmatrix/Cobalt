use std::sync::Arc;

use async_trait::async_trait;
use sqlx::{query, query_as};
use util::error::{Context, Result};

use crate::db::Database;

mod migration1;

/// Trait for defining a database migration.
#[async_trait]
pub trait Migration {
    fn version(&self) -> i64;
    async fn up(&self, db: &mut Database) -> Result<()>;
    #[allow(dead_code)]
    async fn down(&self, db: &mut Database) -> Result<()>;
}

/// Migrator for applying database migrations.
pub struct Migrator<'a> {
    db: &'a mut Database,
    migrations: Vec<Arc<dyn Migration + Send + Sync>>,
}

impl<'a> Migrator<'a> {
    /// Create a new [Migrator] with the given database connection.
    pub fn new(db: &'a mut Database) -> Self {
        let mut migrations: Vec<Arc<dyn Migration + Send + Sync>> =
            vec![Arc::new(migration1::Migration1)];
        // should already by sorted, but just in case
        migrations.sort_by_key(|migration| migration.version());
        Self { db, migrations }
    }

    /// Get the current version of the database based on last run migration.
    pub async fn current_version(&mut self) -> Result<i64> {
        let version: (i64,) = query_as("PRAGMA user_version")
            .persistent(false)
            .fetch_optional(self.db.executor())
            .await
            .context("get current version")?
            .unwrap_or((0,));
        Ok(version.0)
    }

    /// Set the current version of the database based on last run migration.
    pub async fn set_current_version(&mut self, version: i64) -> Result<()> {
        query(&format!("PRAGMA user_version = {version}"))
            .persistent(false)
            .execute(self.db.executor())
            .await
            .context("set current version")?;
        Ok(())
    }

    /// Migrate the database to the latest version.
    pub async fn migrate(mut self) -> Result<()> {
        let current_version = self.current_version().await?;
        let migrations = self.migrations.clone();

        for migration in migrations {
            let version = migration.version();
            if version > current_version {
                migration
                    .up(self.db)
                    .await
                    .with_context(|| format!("migration {version} up"))?;
                self.set_current_version(version).await?;
            }
        }
        Ok(())
    }
}
