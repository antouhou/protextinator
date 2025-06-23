use std::{ops::Deref, sync::Arc};
#[cfg(feature = "serialization")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ArcCowStr {
    Borrowed(&'static str),
    Owned(Arc<str>),
}

impl From<&'static str> for ArcCowStr {
    fn from(s: &'static str) -> Self {
        ArcCowStr::Borrowed(s)
    }
}

impl From<String> for ArcCowStr {
    fn from(s: String) -> Self {
        ArcCowStr::Owned(Arc::from(s))
    }
}

impl Deref for ArcCowStr {
    type Target = str;
    fn deref(&self) -> &str {
        match self {
            ArcCowStr::Borrowed(s) => s,
            ArcCowStr::Owned(a) => a,
        }
    }
}

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