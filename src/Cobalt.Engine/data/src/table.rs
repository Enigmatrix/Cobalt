use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::ops::Deref;

use rusqlite::types::{FromSql, FromSqlResult, ToSqlOutput, ValueRef};
use rusqlite::{Result, ToSql};

/// Trait for mapping an Entity to a [Table] in the database.
pub trait Table {
    type Id: Default + Debug + Clone + Eq;

    /// Unique identifier for the [Table]
    fn id(&self) -> &Ref<Self>
    where
        Self: Sized;

    /// Name of the [Table]
    fn name() -> &'static str;
    /// Column names of the [Table]
    fn columns() -> &'static [&'static str];
}

/// Reference to a [Table] in the database via its unique identifier.
#[derive(Default, Debug, Clone)]
pub struct Ref<T: Table> {
    inner: T::Id,
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
    /// Create a new [Ref] with the given unique identifier
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

/// Basic unique identifier - autoincremented integer
pub type Id = u64;
/// Color in hexadecimal format
pub type Color = String;
/// Timestamp as Windows ticks
pub type Timestamp = u64;
/// Duration as Windows ticks
pub type Duration = u64;

/// Unique identifier with Version
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct VersionedId {
    pub id: u64,
    pub version: u64,
}
