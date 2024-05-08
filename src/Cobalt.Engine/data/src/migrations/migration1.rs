use super::Migration;
use rusqlite::Connection;
use util::error::{bail, Result};

pub struct Migration1;

impl Migration for Migration1 {
    fn version(&self) -> u64 {
        1
    }

    fn up(&self, conn: &Connection) -> Result<()> {
        todo!("up on migration 1")
    }

    fn down(&self, conn: &Connection) -> Result<()> {
        bail!("down on base migration")
    }
}
