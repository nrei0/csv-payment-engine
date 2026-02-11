use std::fs::File;

use engine::engine::Engine;
use error::app_error::AppError;
use output::csv_output::write_accounts_csv;
use source::csv_source::CsvSource;

mod engine;
mod error;
mod output;
mod source;
mod util;

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
        let tx = tx?;
        if let Err(_e) = engine.apply(&tx) {
            // Out assumptions that partners would provide us with correct transactions.
            // So ignore engine errors, but if needed, we can handle it there.
            continue;
        }
    }

    write_accounts_csv(std::io::stdout(), engine.accounts())?;

    Ok(())
}
