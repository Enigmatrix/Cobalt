use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::ops::Deref;

use sqlx::prelude::{FromRow, Type};

/// Trait for mapping an Entity to a [Table] in the database.
pub trait Table {
    type Id: Default + Debug + Clone + Eq;

    /// Unique identifier for the [Table]
    fn id(&self) -> &Ref<Self>
    where
        Self: Sized;

    /// Name of the [Table]
    fn name() -> &'static str;
}

/// Reference to a [Table] in the database via its unique identifier.
#[derive(Default, Debug, Clone, Type)]
#[sqlx(transparent)]
pub struct Ref<T: Table>(T::Id);

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

/// Basic unique identifier - autoincremented integer
pub type Id = u64;
/// Color in hexadecimal format
pub type Color = String;
/// Timestamp as Windows ticks
pub type Timestamp = u64;
/// Duration as Windows ticks
pub type Duration = u64;

/// Unique identifier with Version
#[derive(Default, Debug, Clone, PartialEq, Eq, FromRow)]
pub struct VersionedId {
    pub id: u64,
    pub version: u64,
}
