use rusqlite::Connection;
use util::*;

mod migration1;

pub struct Migrator;

impl Migrator {
    pub fn migrate(conn: &mut Connection) -> Result<()> {
        migration1::run(conn).with_context(|| "Running migration v1")?;
        // place the others here
        Ok(())
    }
}