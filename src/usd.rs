use serde_with::DeserializeFromStr;

use std::{
    fmt::{Debug, Display},
    str::FromStr,
};

use crate::units::Units;

/// Represents an amount of money in USD currency.
///
/// The amount is stored internally as an integer number of cents, but the
/// [`Display`] implementation formats it for display as dollars to 2 decimal
/// places.
#[derive(Clone, Copy, Default, DeserializeFromStr, Eq, PartialEq, Ord, PartialOrd)]
#[must_use = "Don't leave money on the table"]
pub struct Usd(usize);

impl Debug for Usd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl Display for Usd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let dollars = self.0.strict_div(100); // can't fail unless divisor is zero
        let cents = self.0.strict_rem(100); // can't fail unless divisor is zero
        write!(f, "{dollars:>12}.{cents:2}",)
    }
}

impl FromStr for Usd {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(Self(s.replace(['.', ','], "").parse()?))
    }
}

impl Usd {
    pub fn strict_add(&mut self, rhs: Self) -> Self {
        Self(self.0.checked_add(rhs.0).expect("overflow"))
    }

    pub fn strict_mul(self, rhs: Units) -> Self {
        Self(self.0.checked_mul(rhs.0).expect("overflow"))
    }
}
