//! Utility types and functions for the text system.
//!
//! This module contains helper types and utilities used throughout the crate,
//! including string handling optimizations.

#[cfg(feature = "serialization")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::hash::Hash;
use std::{ops::Deref, sync::Arc};

/// An efficient string type that can hold either borrowed static strings or owned arc strings.
///
/// This type is optimized for cases where strings are frequently either static string literals
/// or shared owned strings. It avoids unnecessary allocations when dealing with static strings
/// while providing reference counting for owned strings.
#[derive(Clone, Debug)]
pub enum ArcCowStr {
    /// A borrowed static string slice with 'static lifetime.
    Borrowed(&'static str),
    /// An owned string wrapped in an Arc for efficient cloning.
    Owned(Arc<str>),
}

impl From<&'static str> for ArcCowStr {
    /// Creates an `ArcCowStr` from a static string slice.
    ///
    /// This is the most efficient way to create an `ArcCowStr` from string literals.
    ///
    /// # Examples
    /// ```
    /// use protextinator::utils::ArcCowStr;
    ///
    /// let arc_str: ArcCowStr = "hello world".into();
    /// assert_eq!(&*arc_str, "hello world");
    /// ```
    fn from(s: &'static str) -> Self {
        ArcCowStr::Borrowed(s)
    }
}

impl From<String> for ArcCowStr {
    /// Creates an `ArcCowStr` from an owned String.
    ///
    /// The String is converted to an Arc<str> for efficient sharing.
    ///
    /// # Examples
    /// ```
    /// use protextinator::utils::ArcCowStr;
    ///
    /// let owned_string = String::from("hello world");
    /// let arc_str: ArcCowStr = owned_string.into();
    /// assert_eq!(&*arc_str, "hello world");
    /// ```
    fn from(s: String) -> Self {
        ArcCowStr::Owned(Arc::from(s))
    }
}

impl Deref for ArcCowStr {
    type Target = str;

    /// Dereferences to the underlying string slice.
    ///
    /// This allows `ArcCowStr` to be used wherever a `&str` is expected.
    fn deref(&self) -> &str {
        match self {
            ArcCowStr::Borrowed(s) => s,
            ArcCowStr::Owned(a) => a,
        }
    }
}

impl Hash for ArcCowStr {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.deref().hash(state);
    }
}

impl PartialEq for ArcCowStr {
    fn eq(&self, other: &Self) -> bool {
        self.deref() == other.deref()
    }
}

impl Eq for ArcCowStr {}

#[cfg(feature = "serialization")]
impl Serialize for ArcCowStr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            ArcCowStr::Borrowed(s) => serializer.serialize_str(s),
            ArcCowStr::Owned(a) => serializer.serialize_str(a),
        }
    }
}

#[cfg(feature = "serialization")]
impl<'de> Deserialize<'de> for ArcCowStr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(ArcCowStr::from(s))
    }
}

#[inline(always)]
pub fn srgb_to_linear_u8(c: u8) -> f32 {
    let x = c as f32 / 255.0;
    if x <= 0.04045 {
        x / 12.92
    } else {
        ((x + 0.055) / 1.055).powf(2.4)
    }
}

#[inline(always)]
pub fn linear_to_srgb_u8(x: f32) -> u8 {
    let x = x.clamp(0.0, 1.0);
    let y = if x <= 0.0031308 {
        x * 12.92
    } else {
        1.055 * x.powf(1.0 / 2.4) - 0.055
    };
    (y.clamp(0.0, 1.0) * 255.0 + 0.5).floor() as u8
}
