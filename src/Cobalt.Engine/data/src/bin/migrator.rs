use common::errors::*;
use common::settings::{ConnectionStrings, Settings};

use data::db::Database;
use data::migrator::Migrator;

fn main() -> Result<()> {
    let database_path = std::env::args().nth(1);
    if let Some(database_path) = database_path {
        let mut db = Database::new(&Settings {
            connection_strings: ConnectionStrings { database_path },
            ..Default::default()
        })
        .context("create database")?;
        let mut migrator = Migrator::new(&mut db);
        migrator.migrate().context("perform migration")?;
        println!("migrated!");
    } else {
        println!("no path argument provided!");
    }
    Ok(())
}
