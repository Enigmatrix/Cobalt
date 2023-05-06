use crate::{
    table::{Ref, Table},
    *,
};
use common::errors::*;
use rusqlite::{params, Connection, OptionalExtension, Statement};

macro_rules! prepare_stmt {
    ($conn: expr, $sql:expr) => {{
        let sql = $sql;
        $conn
            .prepare(&sql)
            .with_context(|| format!("prepare stmt: {sql}"))
    }};
}

macro_rules! insert_stmt {
    ($conn: expr, $mdl:ty) => {{
        let name = <$mdl as $crate::table::Table>::name();
        let sql = format!(
            "INSERT INTO {} ({}) VALUES (NULL{})",
            name,
            <$mdl as $crate::table::Table>::columns().join(", "),
            ", ?".repeat(<$mdl as $crate::table::Table>::columns().len() - 1)
        );
        prepare_stmt!($conn, sql)
    }};
}

pub enum FoundOrInserted<T: Table> {
    Found(Ref<T>),
    Inserted(Ref<T>),
}

/// Represents a connection to the database
pub struct Database {
    pub(crate) conn: Connection,
}

impl Database {
    /// Create a new [Database] from a connection string
    pub fn new(conn_str: &str) -> Result<Database> {
        // TODO check if arbitrary conn_str works
        let conn = Connection::open(conn_str).context("open connection")?;
        Ok(Database { conn })
    }
}

/// Reference to hold statements regarding entity insertion
pub struct EntityInserter<'a> {
    conn: &'a Connection,
    find_or_insert_app_stmt: Statement<'a>,
    insert_session: Statement<'a>,
    insert_usage: Statement<'a>,
    insert_interaction_period: Statement<'a>,
}

impl<'a> EntityInserter<'a> {
    /// Initialize a [EntityInserter] from a given [Database]
    pub fn from(db: &'a mut Database) -> Result<EntityInserter<'a>> {
        let conn = &db.conn;
        Ok(Self {
            find_or_insert_app_stmt: prepare_stmt!(
                conn,
                "INSERT INTO app (identity_tag, identity_text0) VALUES (?, ?) ON CONFLICT DO NOTHING RETURNING id"
            )
            .context("find or insert app stmt")?,
            insert_session: insert_stmt!(conn, Session).context("session insert stmt")?,
            insert_usage: insert_stmt!(conn, Usage).context("usage insert stmt")?,
            insert_interaction_period: insert_stmt!(conn, InteractionPeriod)
                .context("interaction period insert stmt")?,
            conn
        })
    }

    /// Find or insert a [App] by its [AppIdentity]
    pub fn find_or_insert_app(&mut self, identity: &AppIdentity) -> Result<FoundOrInserted<App>> {
        let (tag, text0) = Self::destructure_identity(&identity);
        let found_app = self
            .find_or_insert_app_stmt
            .query_row(params![tag, text0], |r| r.get(0))
            .optional()
            .context("find or insert app")?;

        Ok(match found_app {
            Some(id) => FoundOrInserted::Found(id),
            None => FoundOrInserted::Inserted(Ref::new(self.conn.last_insert_rowid() as u64)),
        })
    }

    fn destructure_identity(identity: &AppIdentity) -> (u64, &str) {
        match identity {
            AppIdentity::Win32 { path } => (0, path),
            AppIdentity::Uwp { aumid } => (1, aumid),
        }
    }
}

pub struct AppUpdater<'a> {
    conn: &'a mut Connection,
    update_app: Statement<'a>,
}

#[test]
fn feature() -> Result<()> {
    {
        let db = Database::new("test.db")?;
        db.conn.execute("create table app (id int primary key, gg text unique);", params![])?;
        db.conn.execute("insert into app values (NULL, 'lmao');", params![])?;
        db.conn.execute("insert into app values (NULL, 'lmao') on conflict do nothing returning id;", params![])?;
    }
    std::fs::remove_file("test.db")?;
    Ok(())
}