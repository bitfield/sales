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
    paths: Vec<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let report = Report::from_csv(&args.paths, args.groups)?;
    print!("{report}");
    Ok(())
}
