use anyhow::{bail, Context, Result};
use regex::Regex;
use serde::Deserialize;

use std::{
    collections::BTreeMap,
    fmt::{Debug, Display},
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use crate::usd::Usd;

#[derive(Debug)]
struct Group {
    name: String,
    regex: Regex,
}

/// Holds sales data.
///
/// To create a new, empty `Report`, use [`Report::new`].
///
/// To add group configuration, use [`Report::add_group`] or [`Report::read_groups`].
///
/// To add sales data, use [`Report::read_csv`].
///
/// To get a printable version of the report, use its [`Display`] implementation.
#[derive(Debug, Default)]
pub struct Report {
    groups: Vec<Group>,
    products: BTreeMap<String, Product>,
    units: i32,
    revenue: Usd,
    pub sort_by_revenue: bool,
}

impl Report {
    /// Creates a new, empty report with no data or group configuration.
    #[must_use]
    pub fn new() -> Report {
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
        let mut rdr = csv::Reader::from_path(&path)?;
        for result in rdr.deserialize() {
            let record: Record = result.with_context(|| format!("{}", path.as_ref().display()))?;
            let display_name = self.product_group(&record.name).unwrap_or(record.name);
            let prod = self.products.entry(display_name.clone()).or_default();
            let units = record.qty;
            prod.units += units;
            self.units += units;
            let revenue = record.price * record.qty;
            prod.revenue += revenue;
            self.revenue += revenue;
        }
        Ok(())
    }

    #[must_use]
    /// Returns product names sorted by unit sales, descending.
    ///
    /// The name of the best-selling product (by units, as opposed to revenue)
    /// is given first, and then the remaining names in descending order of unit
    /// sales. Products with identical sales are sorted alphabetically.
    ///
    /// # Panics
    ///
    /// If a product is removed from the map during sorting.
    pub fn products_by_unit_sales(&self) -> Vec<&str> {
        let mut products: Vec<_> = self.products.keys().map(String::as_ref).collect();
        products.sort_by(|a, b| {
            let prod_a = self.products.get(*a).unwrap();
            let prod_b = self.products.get(*b).unwrap();
            prod_b.units.cmp(&prod_a.units)
        });
        products
    }

    #[must_use]
    /// Returns product names sorted by revenue, descending.
    ///
    /// The name of the best-selling product (by revenue, as opposed to units)
    /// is given first, and then the remaining names in descending order of
    /// revenue. Products with identical sales are sorted alphabetically.
    ///
    /// # Panics
    ///
    /// If a product is removed from the map during sorting.
    pub fn products_by_revenue(&self) -> Vec<&str> {
        let mut products: Vec<_> = self.products.keys().map(String::as_ref).collect();
        products.sort_by(|a, b| {
            let prod_a = self.products.get(*a).unwrap();
            let prod_b = self.products.get(*b).unwrap();
            prod_b.revenue.cmp(&prod_a.revenue)
        });
        products
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
        writeln!(
            f,
            "{:width$} {:>6} {:>12}",
            "Product / Group", "Units", "Revenue"
        )?;
        let length = width + 20;
        writeln!(f, "{:-<length$}", "")?;
        let rows = if self.sort_by_revenue {
            self.products_by_revenue()
        } else {
            self.products_by_unit_sales()
        };
        for name in rows {
            let prod = self.products.get(name).unwrap();
            writeln!(f, "{name:width$} {:6} {:>12}", prod.units, prod.revenue)?;
        }
        writeln!(f, "{:-<length$}", "")?;
        writeln!(f, "{:width$} {:6} {}", "Total", self.units, self.revenue)?;
        Ok(())
    }
}

/// Holds sales data on a specific product.
#[derive(Debug, Default)]
pub struct Product {
    pub units: i32,
    pub revenue: Usd,
}

/// Defines the CSV format for sales data.
#[derive(Debug, Deserialize)]
pub struct Record {
    #[serde(rename = "Lineitem quantity", alias = "Quantity")]
    pub qty: i32,
    #[serde(rename = "Lineitem name", alias = "Item Name")]
    pub name: String,
    #[serde(rename = "Lineitem price", alias = "Item Price ($)")]
    pub price: Usd,
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn read_groups_fn_correctly_parses_group_data() {
        let mut report = Report::new();
        report.read_groups("testdata/groups").unwrap();
        assert_eq!(
            report
                .product_group("The Power of Go: Tests (Go 1.22 edition)")
                .unwrap(),
            "The Power of Go"
        );
        assert_eq!(
            report
                .product_group("For the Love of Go (Go 1.23 edition)")
                .unwrap(),
            "For the Love of Go"
        );
        assert_eq!(report.product_group("bogus product"), None);
    }

    #[test]
    fn read_groups_fn_returns_error_for_bad_line_format() {
        let mut report = Report::new();
        assert!(report.read_groups("testdata/groups.bad").is_err());
    }

    #[test]
    fn add_group_fn_adds_group_to_report() {
        let mut report = Report::new();
        report.add_group("Foo", "foo").unwrap();
        assert_eq!(report.product_group("foo variant 1"), Some("Foo".into()));
    }

    #[test]
    fn read_csv_fn_correctly_parses_squarespace_data() {
        let mut report = Report::new();
        report.read_csv("testdata/squarespace.csv").unwrap();
        assert_eq!(report.units, 17, "wrong units");
        assert_eq!(report.revenue, Usd::from_str("3,409.15").unwrap());
    }

    #[test]
    fn read_csv_fn_correctly_parses_gumroad_data() {
        let mut report = Report::new();
        report.read_csv("testdata/gumroad.csv").unwrap();
        assert_eq!(report.units, 7, "wrong units");
        assert_eq!(report.revenue, Usd::default());
    }

    #[test]
    fn products_by_unit_sales_fn_sorts_prods_by_units() {
        let mut report = Report::new();
        report.read_csv("testdata/squarespace.csv").unwrap();
        assert_eq!(
            report.products_by_revenue(),
            vec![
                "Go mentoring",
                "Code For Your Life",
                "For the Love of Go: Video/Book Bundle (2023 edition)",
                "For the Love of Go (2023)",
                "The Power of Go: Tests",
                "The Power of Go: Tools",
            ]
        );
    }

    #[test]
    fn products_by_revenue_fn_sorts_prods_by_revenue() {
        let mut report = Report::new();
        report.read_csv("testdata/squarespace.csv").unwrap();
        assert_eq!(
            report.products_by_unit_sales(),
            vec![
                "Go mentoring",
                "Code For Your Life",
                "For the Love of Go (2023)",
                "For the Love of Go: Video/Book Bundle (2023 edition)",
                "The Power of Go: Tests",
                "The Power of Go: Tools",
            ]
        );
    }
}
