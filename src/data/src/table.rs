use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::ops::Deref;

use serde::{Deserialize, Serialize};
use sqlx::prelude::{FromRow, Type};
use sqlx::sqlite::SqliteRow;
use util::num::N64;

/// Trait for mapping an Entity to a [Table] in the database.
pub trait Table {
    type Id: Default + Debug + Clone + Hash + PartialEq + Eq + Serialize;

    /// Unique identifier for the [Table]
    fn id(&self) -> &Ref<Self>
    where
        Self: Sized;

    /// Name of the [Table]
    fn name() -> &'static str;
}

/// Reference to a [Table] in the database via its unique identifier.
#[derive(Default, Debug, Clone, Type, Serialize, Deserialize)]
#[sqlx(transparent)]
pub struct Ref<T: Table>(pub T::Id);

impl<T: Table<Id: PartialEq>> PartialEq for Ref<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T: Table<Id: Eq>> Eq for Ref<T> {}

impl<T: Table<Id: Hash>> Hash for Ref<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<T: Table> Ref<T> {
    /// Create a new [Ref] with the given unique identifier
    pub fn new(inner: T::Id) -> Self {
        Self(inner)
    }
}

impl<T: Table> Deref for Ref<T> {
    type Target = T::Id;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, T: Table> FromRow<'a, SqliteRow> for Ref<T>
where
    T::Id: FromRow<'a, SqliteRow>,
{
    fn from_row(row: &'a SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(Ref::new(<T::Id as FromRow<'a, SqliteRow>>::from_row(row)?))
    }
}

/// Basic unique identifier - autoincremented integer
pub type Id = i64;
/// Color in hexadecimal format
pub type Color = String;
/// Timestamp as Windows ticks
pub type Timestamp = i64;
/// Duration as Windows ticks
pub type Duration = i64;

/// Time Period for grouping usages
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Period {
    /// Hour
    Hour,
    /// Day
    Day,
    /// Week
    Week,
    /// Month
    Month,
    /// Year
    Year,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Real(N64);

impl sqlx::Type<sqlx::Sqlite> for Real {
    fn type_info() -> sqlx::sqlite::SqliteTypeInfo {
        <f64 as sqlx::Type<sqlx::Sqlite>>::type_info()
    }
}

impl sqlx::Encode<'_, sqlx::Sqlite> for Real {
    fn encode_by_ref(
        &self,
        buf: &mut <sqlx::Sqlite as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        <f64 as sqlx::Encode<'_, sqlx::Sqlite>>::encode_by_ref(&self.0.into(), buf)
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Sqlite> for Real {
    fn decode(
        value: <sqlx::Sqlite as sqlx::Database>::ValueRef<'r>,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        <f64 as sqlx::Decode<'r, sqlx::Sqlite>>::decode(value).map(Real::from)
    }
}

impl From<f64> for Real {
    fn from(value: f64) -> Self {
        Real(N64::new(value))
    }
}

impl From<Real> for f64 {
    fn from(value: Real) -> Self {
        value.0.into()
    }
}
