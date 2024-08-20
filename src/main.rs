use std::{collections::HashMap, env, fmt::Display, ops::AddAssign, str::FromStr};

use anyhow::Result;
use serde::Deserialize;
use serde_with::DeserializeFromStr;

#[derive(Debug, Default, DeserializeFromStr)]
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
        write!(f, "${dollars:.2}")
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

#[derive(Debug, Default)]
pub struct Product {
    pub units: usize,
    pub revenue: USD,
}

fn main() -> Result<()> {
    let mut products: HashMap<String, Product> = HashMap::new();
    let mut units = 0;
    let mut revenue = USD(0);
    let mut rdr = csv::Reader::from_path(env::args().nth(1).unwrap()).unwrap();
    for result in rdr.deserialize() {
        let record: Record = result?;
        let prod = products.entry(record.line_item_name).or_default();
        prod.units += 1;
        prod.revenue += record.line_item_price;
    }
    for (name, prod) in products {
        println!("{name:+20} {} {}", prod.units, prod.revenue);
        units += prod.units;
        revenue += prod.revenue;
    }
    println!("Total revenue {revenue}");
    println!("Total units {units}");
    Ok(())
}
