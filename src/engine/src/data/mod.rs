pub mod db;
pub mod migrations;
pub mod model;

pub use db::*;

use migrations::Migrator;
use util::*;
use std::mem::MaybeUninit;
use rusqlite::{ffi::sqlite3, Connection};

#[no_mangle]
pub extern "C" fn migrate(hdl: *mut sqlite3, err: &mut MaybeUninit<String>) {
    fn inner(hdl: *mut sqlite3) -> Result<()> {
        let mut conn =
            unsafe { Connection::from_handle(hdl).with_context(|| "Create connection from handle")? };
        Migrator::migrate(&mut conn).with_context(|| "Run migrations on database")
    }

    match inner(hdl) {
        Ok(_) => *err = MaybeUninit::zeroed(),
        Err(e) => *err = MaybeUninit::new(e.to_string())
    }
}
