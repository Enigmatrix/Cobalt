use crate::migrator::Migration;

pub struct Migration1;

impl Migration for Migration1 {
    fn version(&self) -> u64 {
        1
    }

    fn up(&mut self, conn: &mut rusqlite::Connection) -> utils::errors::Result<()> {
        todo!()
    }

    fn down(&mut self, conn: &mut rusqlite::Connection) -> utils::errors::Result<()> {
        todo!()
    }
}
