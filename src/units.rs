use std::fmt::{Debug, Display};

use serde::Deserialize;

/// Represents a number of product units.
///
/// The amount is stored internally as a `usize`.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Ord, PartialOrd)]
#[must_use]
pub struct Units(pub usize);

impl Display for Units {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl Units {
    pub fn strict_add(&mut self, rhs: Self) -> Self {
        Self(self.0.checked_add(rhs.0).expect("overflow"))
    }
}
