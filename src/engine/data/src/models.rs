use rusqlite::types::ToSqlOutput;
use rusqlite::{Result, ToSql};

pub type Timestamp = u64;

#[derive(Default, Debug, PartialEq, Eq)]
pub struct App {
    pub id: Ref<Self>,
    pub name: String,
    pub description: String,
    pub company: String,
    pub color: Option<String>,
    pub identity: AppIdentity,
}

#[derive(Default, Debug, PartialEq, Eq)]
pub struct Session {
    pub id: Ref<Self>,
    pub app: Ref<App>,
    pub title: String,
    pub cmd_line: Option<String>,
}

#[derive(Default, Debug, PartialEq, Eq)]
pub struct Usage {
    pub id: Ref<Self>,
    pub session: Ref<Session>,
    pub start: Timestamp,
    pub end: Timestamp,
}

#[derive(Default, Debug, PartialEq, Eq)]
pub struct InteractionPeriod {
    pub id: Ref<Self>,
    pub start: Timestamp,
    pub end: Timestamp,
    pub mouseclicks: u64,
    pub keystrokes: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// Unique identity of an App, outside of the Database (on the FileSystem/Registry)
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
    type Id: Default;

    fn id(&self) -> &Ref<Self>
    where
        Self: Sized;

    fn columns() -> &'static [&'static str];
}

#[derive(Default, Debug)]
pub struct Ref<T: Table> {
    pub inner: T::Id,
}

impl<T: Table> Ref<T> {
    pub fn new(inner: T::Id) -> Self {
        Self { inner }
    }
}

impl<Id: Clone, T: Table<Id = Id>> Clone for Ref<T> {
    fn clone(&self) -> Self {
        Self::new(self.inner.clone())
    }
}

impl<Id: PartialEq, T: Table<Id = Id>> PartialEq for Ref<T> {
    fn eq(&self, other: &Self) -> bool {
        self.inner.eq(&other.inner)
    }
}

impl<Id: Eq, T: Table<Id = Id>> Eq for Ref<T> {}

impl<Id: ToSql + 'static, T: Table<Id = Id>> ToSql for Ref<T> {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>> {
        self.inner.to_sql()
    }
}

macro_rules! table {
    ($t:ty, $fld:ident : $id:ty, $cols: expr) => {
        impl Table for $t {
            type Id = $id;
            fn id(&self) -> &Ref<Self> {
                &self.$fld
            }
            fn columns() -> &'static [&'static str] {
                &$cols
            }
        }
    };
}

table!(
    App,
    id: u64,
    [
        "id",
        "name",
        "description",
        "company",
        "color",
        "identity_tag",
        "identity_text0"
    ]
);
table!(Session, id: u64, ["id", "app", "title", "cmd_line",]);
table!(Usage, id: u64, ["id", "session", "start", "end"]);
table!(
    InteractionPeriod,
    id: u64,
    ["id", "start", "end", "mouseclicks", "keystrokes"]
);
