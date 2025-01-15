use std::{collections::BTreeMap, fmt::Display, ops::AddAssign, path::Path, str::FromStr};

use serde::Deserialize;
use serde_with::DeserializeFromStr;

use crate::groups::Groups;

#[derive(Debug, Default)]
pub struct Product {
    pub units: usize,
    pub revenue: USD,
}

pub struct Report {
    pub products: BTreeMap<String, Product>,
    pub groups: Groups,
    pub total_units: usize,
    pub total_revenue: USD,
}

impl Report {
    /// Builds a `Report` from the given set of CSV files, optionally grouping them by the groups defined at `groups_path`.
    pub fn from_csv(
        paths: &[impl AsRef<Path>],
        groups_path: Option<impl AsRef<Path>>,
    ) -> anyhow::Result<Self> {
        let mut groups = Groups::default();
        if let Some(groups_path) = groups_path {
            groups = Groups::from_file(&groups_path)?;
        }
        let mut products: BTreeMap<String, Product> = BTreeMap::new();
        let mut total_units = 0;
        let mut total_revenue = USD(0);
        for path in paths {
            let mut rdr = csv::Reader::from_path(path)?;
            for result in rdr.deserialize() {
                let record: Record = result?;
                let display_name = groups
                    .product_group(&record.line_item_name)
                    .unwrap_or(record.line_item_name);
                let prod = products.entry(display_name).or_default();
                prod.units += 1;
                total_units += 1;
                prod.revenue += record.line_item_price;
                total_revenue += record.line_item_price;
            }
        }
        Ok(Self {
            products,
            groups,
            total_units,
            total_revenue,
        })
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
        writeln!(f, "Total revenue {:>}", self.total_revenue)?;
        writeln!(f, "Total units {}", self.total_units)?;
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct Record {
    #[serde(rename = "Order ID")]
    pub order_id: String,
    #[serde(rename = "Lineitem name")]
    pub line_item_name: String,
    #[serde(rename = "Lineitem price")]
    pub line_item_price: USD,
}

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
