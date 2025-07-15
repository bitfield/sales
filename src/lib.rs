#![doc = include_str!("../README.md")]
use anyhow::{bail, Result};
use regex::Regex;
use serde::Deserialize;
use serde_with::DeserializeFromStr;

use std::{
    collections::BTreeMap,
    fmt::{Debug, Display},
    fs::File,
    io::{BufRead, BufReader},
    ops::{AddAssign, Mul},
    path::Path,
    str::FromStr,
};

#[derive(Debug)]
struct Group {
    name: String,
    regex: Regex,
}

/// Holds sales data.
///
/// To create a new, empty `Report`, use [`Self::new`].
///
/// To add group configuration, use [`Self::add_group`] or [`Self::read_groups`].
///
/// To add sales data, use [`Self::read_csv`].
///
/// To get a printable version of the report, use its [`Display`] implementation.
#[derive(Debug, Default)]
pub struct Report {
    groups: Vec<Group>,
    products: BTreeMap<String, Product>,
    units: i32,
    revenue: USD,
}

impl Report {
    /// Creates a new, empty report with no data or group configuration.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Reads product group configuration from `path`.
    ///
    /// The configuration file consists of group specifications, one per line,
    /// in the following format:
    ///
    /// ```txt
    /// GROUP_NAME | GROUP_REGEX
    /// ```
    ///
    /// # Examples
    ///
    /// A very simple example:
    ///
    /// ```txt
    /// Foo | foo
    /// ```
    ///
    /// With this group defined, when analysing the sales data, all products
    /// whose name contains `foo` will be counted as a single product named
    /// `Foo`.
    ///
    /// `GROUP_REGEX` can be any regular expression supported by [`regex::Regex`].
    ///
    /// # Errors
    ///
    /// Returns errors if:
    /// * The file cannot be opened
    /// * The file cannot be read
    /// * There is a line with an invalid format (no `|` character)
    /// * `GROUP_REGEX` is an invalid regular expression
    pub fn read_groups(&mut self, path: impl AsRef<Path>) -> Result<()> {
        let file = BufReader::new(File::open(&path)?);
        for line in file.lines() {
            let line = line?;
            let Some((name, regex_str)) = line.split_once(" | ") else {
                bail!(
                    "reading {:?}: bad line format (missing |): {line}",
                    path.as_ref(),
                );
            };
            self.add_group(name, regex_str)?;
        }
        Ok(())
    }

    /// Returns the product group for `line_item`, if any.
    ///
    /// If the product name `line_item` matches the regular expression for any
    /// defined product group, then this function returns the name of that
    /// group.
    ///
    /// # Examples
    ///
    /// ```
    /// # use regex::Regex;
    /// # use sales::Report;
    /// let mut report = Report::new();
    /// report.add_group("Foo", "foo").unwrap();
    /// assert_eq!(report.product_group("foo variant 1"), Some("Foo".into()));
    /// assert_eq!(report.product_group("ungrouped product"), None);
    /// ```
    #[must_use]
    pub fn product_group(&self, line_item: &str) -> Option<String> {
        self.groups
            .iter()
            .find(|g| g.regex.is_match(line_item))
            .map(|g| g.name.clone())
    }

    /// Adds a new group configuration.
    ///
    /// Products whose name matches `regex_str` will be reported as part of
    /// product group `name`, rather than their own line item names.
    ///
    /// # Errors
    ///
    /// Returns any errors from compiling `regex_str` with [`Regex::new`].
    pub fn add_group(&mut self, name: &str, regex_str: &str) -> Result<()> {
        self.groups.push(Group {
            name: name.to_string(),
            regex: Regex::new(regex_str)?,
        });
        Ok(())
    }

    /// Reads sales data from the CSV files at `paths`, and updates the report.
    ///
    /// # Errors
    ///
    /// Returns any errors from opening or parsing a CSV file.
    pub fn read_csv(&mut self, path: impl AsRef<Path>) -> Result<()> {
        let mut rdr = csv::Reader::from_path(path)?;
        for result in rdr.deserialize() {
            let record: Record = result?;
            let display_name = self
                .product_group(&record.line_item_name)
                .unwrap_or(record.line_item_name);
            let prod = self.products.entry(display_name.clone()).or_default();
            let units = record.line_item_qty;
            prod.units += units;
            self.units += units;
            let revenue = record.line_item_price * record.line_item_qty;
            prod.revenue += revenue;
            self.revenue += revenue;
        }
        Ok(())
    }
}

impl Display for Report {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let width = self
            .products
            .keys()
            .max_by_key(|name| name.len())
            .unwrap()
            .len();
        for (name, prod) in &self.products {
            writeln!(f, "{name:width$} {}\t{}", prod.units, prod.revenue)?;
        }
        writeln!(f, "Total revenue {:>}", self.revenue)?;
        writeln!(f, "Total units {}", self.units)?;
        Ok(())
    }
}

/// Holds sales data on a specific product.
#[derive(Debug, Default)]
pub struct Product {
    pub units: i32,
    pub revenue: USD,
}

/// Defines the CSV format for sales data.
#[derive(Debug, Deserialize)]
pub struct Record {
    #[serde(rename = "Lineitem quantity", alias = "Quantity")]
    pub line_item_qty: i32,
    #[serde(rename = "Lineitem name", alias = "Item Name")]
    pub line_item_name: String,
    #[serde(rename = "Lineitem price", alias = "Item Price ($)")]
    pub line_item_price: USD,
}

/// Represents an amount of money in USD currency.
///
/// The amount is stored internally as an integer number of cents, but the
/// [`Display`] implementation formats it for display as dollars to 2 decimal
/// places.
#[derive(Clone, Copy, Default, DeserializeFromStr, Eq, PartialEq)]
pub struct USD(i32);

impl Debug for USD {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl Display for USD {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let dollars = f64::from(self.0) / 100.0;
        write!(f, "${:>8}", format!("{dollars:.2}"))
    }
}

impl FromStr for USD {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(Self(s.replace(['.', ','], "").parse()?))
    }
}

impl AddAssign for USD {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl Mul<i32> for USD {
    type Output = Self;

    fn mul(self, rhs: i32) -> Self::Output {
        Self(self.0 * rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_groups_fn_correctly_parses_group_data() {
        let mut reporter = Report::new();
        reporter.read_groups("testdata/groups").unwrap();
        assert_eq!(
            reporter
                .product_group("The Power of Go: Tests (Go 1.22 edition)")
                .unwrap(),
            "The Power of Go"
        );
        assert_eq!(
            reporter
                .product_group("For the Love of Go (Go 1.23 edition)")
                .unwrap(),
            "For the Love of Go"
        );
        assert_eq!(reporter.product_group("bogus product"), None);
    }

    #[test]
    fn read_groups_fn_returns_error_for_bad_line_format() {
        let mut reporter = Report::new();
        assert!(reporter.read_groups("testdata/groups.bad").is_err());
    }

    #[test]
    fn add_group_fn_adds_group_to_reporter() {
        let mut reporter = Report::new();
        reporter.add_group("Foo", "foo").unwrap();
        assert_eq!(reporter.product_group("foo variant 1"), Some("Foo".into()));
    }

    #[test]
    fn read_csv_fn_correctly_parses_squarespace_data() {
        let mut reporter = Report::new();
        reporter.read_csv("testdata/squarespace.csv").unwrap();
        assert_eq!(reporter.units, 17, "wrong units");
        assert_eq!(reporter.revenue, USD::from_str("3,409.15").unwrap());
    }

    #[test]
    fn read_csv_fn_correctly_parses_gumroad_data() {
        let mut reporter = Report::new();
        reporter.read_csv("testdata/gumroad.csv").unwrap();
        assert_eq!(reporter.units, 7, "wrong units");
        assert_eq!(reporter.revenue, USD::default());
    }
}
