#![doc = include_str!("../README.md")]
use anyhow::{bail, Result};
use regex::Regex;
use serde::Deserialize;
use serde_with::DeserializeFromStr;

use std::{
    collections::BTreeMap,
    fmt::Display,
    fs::File,
    io::{BufRead, BufReader},
    ops::AddAssign,
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
    units: usize,
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
    pub fn read_csv(&mut self, paths: &[impl AsRef<Path>]) -> Result<()> {
        for path in paths {
            let mut rdr = csv::Reader::from_path(path)?;
            for result in rdr.deserialize() {
                let record: Record = result?;
                let display_name = self
                    .product_group(&record.line_item_name)
                    .unwrap_or(record.line_item_name);
                let prod = self.products.entry(display_name).or_default();
                prod.units += 1;
                self.units += 1;
                prod.revenue += record.line_item_price;
                self.revenue += record.line_item_price;
            }
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
    pub units: usize,
    pub revenue: USD,
}

/// Defines the CSV format for sales data.
#[derive(Debug, Deserialize)]
pub struct Record {
    #[serde(rename = "Order ID")]
    pub order_id: String,
    #[serde(rename = "Lineitem name")]
    pub line_item_name: String,
    #[serde(rename = "Lineitem price")]
    pub line_item_price: USD,
}

/// Represents an amount of money in USD currency.
///
/// The amount is stored internally as an integer number of cents, but the
/// [`Display`] implementation formats it for display as dollars to 2 decimal
/// places.
#[derive(Clone, Copy, Debug, Default, DeserializeFromStr)]
pub struct USD(i32);

impl FromStr for USD {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(Self(s.replace('.', "").parse()?))
    }
}

impl AddAssign for USD {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl Display for USD {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let dollars = f64::from(self.0) / 100.0;
        write!(f, "${:>8}", format!("{dollars:.2}"))
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
    fn add_group_fn_adds_group_to_reporter() {
        let mut reporter = Report::new();
        reporter.add_group("Foo", "foo").unwrap();
        assert_eq!(reporter.product_group("foo variant 1"), Some("Foo".into()));
    }
}
