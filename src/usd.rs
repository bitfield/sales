use serde_with::DeserializeFromStr;

use std::{
    fmt::{Debug, Display},
    ops::{AddAssign, Mul},
    str::FromStr,
};

/// Represents an amount of money in USD currency.
///
/// The amount is stored internally as an integer number of cents, but the
/// [`Display`] implementation formats it for display as dollars to 2 decimal
/// places.
#[derive(Clone, Copy, Default, DeserializeFromStr, Eq, PartialEq, Ord, PartialOrd)]
pub struct Usd(i32);

impl Debug for Usd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl Display for Usd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let dollars = f64::from(self.0) / 100.0;
        write!(f, "{dollars:>12.2}",)
    }
}

impl FromStr for Usd {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(Self(s.replace(['.', ','], "").parse()?))
    }
}

impl AddAssign for Usd {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl Mul<i32> for Usd {
    type Output = Self;

    fn mul(self, rhs: i32) -> Self::Output {
        Self(self.0 * rhs)
    }
}
