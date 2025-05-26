use std::fmt;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Id(pub u64);

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Id {
    pub const NULL: Self = Id(0);

    pub fn new(id: impl Hash) -> Self {
        let mut hasher = ahash::AHasher::default();
        id.hash(&mut hasher);
        Self(hasher.finish())
    }

    pub fn with(&self, id: impl Hash) -> Self {
        Self::new((self.0, id))
    }
}
