use std::fmt::{Debug, Display};

use serde::Deserialize;

/// Represents an amount of money in USD currency.
///
/// The amount is stored internally as an integer number of cents, but the
/// [`Display`] implementation formats it for display as dollars to 2 decimal
/// places.
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
