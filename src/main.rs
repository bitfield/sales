use anyhow::Result;
use clap::Parser;

use std::path::PathBuf;

use sales::Report;

#[derive(Parser)]
/// Summarises sales data from a CSV file.
struct Args {
    #[arg(short, long)]
    /// Groups related line items using this config file.
    groups: Option<PathBuf>,
    /// Path(s) to the CSV sales data file(s).
    #[arg(required(true))]
    csv: Vec<PathBuf>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let mut report = Report::new();
    if let Some(path) = args.groups {
        report.read_groups(path)?;
    }
    for csv in args.csv {
        report.read_csv(csv)?;
    }
    print!("{report}");
    Ok(())
}
