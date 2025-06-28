use std::collections::{HashMap, HashSet};

// Currently there are just a few marker types which can be later
// substituted with the actual types from other crates like smallvec.
// Notably, they also serve as a hint that the collection is small.

/// A small vector, used for small collections that are not expected to grow beyond a certain size.
pub type SmallVec<T> = Vec<T>;
/// A small hash map, used for small collections that are not expected to grow beyond a certain size.
pub type SmallHashMap<K, V> = HashMap<K, V>;
/// A small hash set, used for small collections that are not expected to grow beyond a certain size.
pub type SmallHashSet<T> = HashSet<T>;
