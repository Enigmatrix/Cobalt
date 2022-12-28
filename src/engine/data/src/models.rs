use rusqlite::types::ToSqlOutput;
use rusqlite::{Result, ToSql};
use std::fmt::Debug;

pub type Timestamp = u64;

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct App {
    pub id: Ref<Self>,
    // pub initialized: bool,
    pub name: String,
    pub description: String,
    pub company: String,
    pub color: Option<String>,
    pub identity: AppIdentity,
    // pub icon:
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Session {
    pub id: Ref<Self>,
    pub app: Ref<App>,
    pub title: String,
    pub cmd_line: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Usage {
    pub id: Ref<Self>,
    pub session: Ref<Session>,
    pub start: Timestamp,
    pub end: Timestamp,
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct InteractionPeriod {
    pub id: Ref<Self>,
    pub start: Timestamp,
    pub end: Timestamp,
    pub mouseclicks: u64,
    pub keystrokes: u64,
}

/// Unique identity of an App, outside of the Database (on the FileSystem/Registry)
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AppIdentity {
    Win32 { path: String },
    UWP { aumid: String },
}

impl Default for AppIdentity {
    fn default() -> Self {
        Self::Win32 {
            path: Default::default(),
        }
    }
}

pub trait Table {
    type Id: Default + Debug + Clone + Eq;

    fn id(&self) -> &Ref<Self>
    where
        Self: Sized;

    fn name() -> &'static str;
    fn columns() -> &'static [&'static str];
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Ref<T: Table> {
    pub inner: T::Id,
}

impl<T: Table> Ref<T> {
    pub fn new(inner: T::Id) -> Self {
        Self { inner }
    }
}

impl<Id: ToSql + 'static, T: Table<Id = Id>> ToSql for Ref<T> {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>> {
        self.inner.to_sql()
    }
}

macro_rules! table {
    ($t:ty, $name:expr, $fld:ident : $id:ty, $cols:expr) => {
        impl Table for $t {
            type Id = $id;
            fn id(&self) -> &Ref<Self> {
                &self.$fld
            }
            fn name() -> &'static str {
                $name
            }
            fn columns() -> &'static [&'static str] {
                &$cols
            }
        }
    };
}

table!(
    App,
    "app",
    id: u64,
    [
        "id",
        "initialized",
        "name",
        "description",
        "company",
        "color",
        "identity_tag",
        "identity_text0",
        // "icon", // TODO icon
    ]
);
table!(
    Session,
    "session",
    id: u64,
    ["id", "app", "title", "cmd_line",]
);
table!(Usage, "usage", id: u64, ["id", "session", "start", "end"]);
table!(
    InteractionPeriod,
    "interaction_period",
    id: u64,
    ["id", "start", "end", "mouseclicks", "keystrokes"]
);
