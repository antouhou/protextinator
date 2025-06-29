//! Unique identifier type for text states and other resources.
//!
//! This module provides a hash-based ID system for efficiently identifying
//! and tracking text states and other resources in the system.

use std::fmt;
use std::hash::{Hash, Hasher};

/// A unique identifier based on hash values.
///
/// `Id` provides a way to create stable, unique identifiers from any hashable data.
/// This is particularly useful for identifying text states, UI elements, and other
/// resources that need consistent identification across frames.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Id(pub u64);

impl fmt::Display for Id {
    /// Formats the ID as its underlying u64 value.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Id {
    /// A null/empty ID with value 0.
    ///
    /// This can be used as a default or placeholder value.
    pub const NULL: Self = Id(0);

    /// Creates a new ID by hashing the provided value.
    ///
    /// The same input will always produce the same ID, making this suitable
    /// for stable identification across application runs.
    ///
    /// # Arguments
    /// * `id` - Any hashable value to create an ID from
    ///
    /// # Examples
    /// ```
    /// use protextinator::Id;
    /// 
    /// let id1 = Id::new("my_text_element");
    /// let id2 = Id::new("my_text_element");
    /// assert_eq!(id1, id2); // Same input produces same ID
    /// 
    /// let id3 = Id::new(42);
    /// let id4 = Id::new((42, "suffix"));
    /// assert_ne!(id3, id4); // Different inputs produce different IDs
    /// ```
    pub fn new(id: impl Hash) -> Self {
        let mut hasher = ahash::AHasher::default();
        id.hash(&mut hasher);
        Self(hasher.finish())
    }

    /// Creates a new ID by combining this ID with another hashable value.
    ///
    /// This is useful for creating hierarchical or composite IDs.
    ///
    /// # Arguments
    /// * `id` - Any hashable value to combine with this ID
    ///
    /// # Returns
    /// A new ID that represents the combination of this ID and the provided value
    ///
    /// # Examples
    /// ```
    /// use protextinator::Id;
    /// 
    /// let base_id = Id::new("window");
    /// let button_id = base_id.with("button");
    /// let label_id = base_id.with("label");
    /// 
    /// assert_ne!(button_id, label_id);
    /// ```
    pub fn with(&self, id: impl Hash) -> Self {
        Self::new((self.0, id))
    }
}
