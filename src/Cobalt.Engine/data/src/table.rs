use std::fmt::Debug;

use rusqlite::types::ToSqlOutput;
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
