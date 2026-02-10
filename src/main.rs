use std::fs::File;
use std::io::Write;

use csv::Writer;
use engine::{account::Account, engine::Engine};
use error::{app_error::AppError, source_error::SourceError};
use rust_decimal::{Decimal, prelude::FromPrimitive};
use source::csv_source::CsvSource;

mod dto;
mod engine;
mod error;
mod source;

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

// CSV runner.
fn run() -> Result<(), AppError> {
    // Read arguments and open file for reading.
    let path = std::env::args().nth(1).ok_or(AppError::MissingInput)?;
    let file = File::open(path)?;

    // Source impl. iterator over transactions, so we can process them one by one without loading the whole file into memory.
    let source = CsvSource::new(file);

    let mut engine = Engine::new();

    // Process transactions one by one by engine.
    for tx in source {
        // @andy What if one transaction is invalid? Should we skip it and continue with the next one, or should we stop processing and report the error?
        //       Or should we consider that all transactions are valid and just panic if we encounter an invalid one?
        let tx = tx?;
        if let Err(_e) = engine.apply(&tx) {
            // @andy There are some errors which could be ignored vs some which are okay.
            //       Fow now just ignore.
            continue;
        }
    }

    write_accounts_csv(std::io::stdout(), engine.accounts()).map_err(SourceError::from)?;

    Ok(())
}

// @andy move to utils.
fn format_amount(v: i64) -> String {
    let mut d = Decimal::from_i64(v).unwrap();
    d.set_scale(4).unwrap();
    d /= Decimal::new(10_000, 0);
    d.to_string()
}

// @andy move closer to csv source (output).
fn write_accounts_csv<'a, W: Write>(
    writer: W,
    accounts: impl Iterator<Item = &'a Account>,
) -> Result<(), csv::Error> {
    let mut writer = Writer::from_writer(writer);

    writer.write_record(["client", "available", "held", "total", "locked"])?;

    for acc in accounts {
        writer.write_record([
            acc.client.0.to_string(),
            format_amount(acc.available),
            format_amount(acc.held),
            format_amount(acc.total()),
            acc.locked.to_string(),
        ])?;
    }

    writer.flush()?;
    Ok(())
}
