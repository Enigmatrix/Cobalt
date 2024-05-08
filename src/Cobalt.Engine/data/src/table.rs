use std::fmt::Debug;

use rusqlite::types::{FromSql, FromSqlResult, ToSqlOutput, ValueRef};
use rusqlite::{Result, ToSql};

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

impl<Id: FromSql + 'static, T: Table<Id = Id>> FromSql for Ref<T> {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        Id::column_result(value).map(|id| Ref::new(id))
    }
}

pub type Id = u64;
pub type Color = String;
pub type Timestamp = u64;
pub type Duration = u64;

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct VersionedId {
    pub guid: uuid::Uuid,
    pub version: u64,
}
