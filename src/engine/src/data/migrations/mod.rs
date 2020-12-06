use rusqlite::{Connection, NO_PARAMS};
use util::*;

mod migration1;

pub struct Migrator;

impl Migrator {
    pub fn migrate(conn: &mut Connection) -> Result<()> {
        let version = Migrator::get_version(conn).with_context(|| "Get Version of Migrations")?;
        if version < 1 {
            migration1::run(conn).with_context(|| "Running migration v1")?;
        }
        /* uncomment when we have a version 2 of migrations
        if version < 2 {
            migration2::run(conn).with_context(|| "Running migration v1")?;
        }
        */
        Ok(())
    }

    fn get_version(conn: &mut Connection) -> rusqlite::Result<i64> {
        match conn.query_row("select max(version) from Migrations", NO_PARAMS, |f| {
            f.get(0)
        }) {
            Ok(version) => Ok(version),
            Err(rusqlite::Error::SqliteFailure(_, Some(text)))
                if text == "no such table: Migrations" =>
            {
                Ok(0)
            }
            Err(e) => Err(e),
        }
    }
}
