use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::ops::Deref;

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

#[derive(Default, Debug, Clone)]
pub struct Ref<T: Table> {
    pub inner: T::Id,
}

impl<T: Table<Id: PartialEq>> PartialEq for Ref<T> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<T: Table<Id: Eq>> Eq for Ref<T> {}

impl<T: Table<Id: Hash>> Hash for Ref<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inner.hash(state);
    }
}

impl<T: Table> Ref<T> {
    pub fn new(inner: T::Id) -> Self {
        Self { inner }
    }
}

impl<T: Table> Deref for Ref<T> {
    type Target = T::Id;

    fn deref(&self) -> &Self::Target {
        &self.inner
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
    pub id: u64,
    pub version: u64,
}
