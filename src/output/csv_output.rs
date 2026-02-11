use std::io::Write;

use crate::{
    engine::account::Account,
    error::{app_error::AppError, output_error::OutputError},
    util::amount::format_amount_to_string,
};

pub fn write_accounts_csv<'a, W: Write>(
    writer: W,
    accounts: impl Iterator<Item = &'a Account>,
) -> Result<(), AppError> {
    let mut writer: csv::Writer<W> = csv::Writer::from_writer(writer);

    writer
        .write_record(["client", "available", "held", "total", "locked"])
        .map_err(OutputError::from)?;

    for acc in accounts {
        writer
            .write_record([
                acc.client.0.to_string(),
                format_amount_to_string(acc.available),
                format_amount_to_string(acc.held),
                format_amount_to_string(acc.total()),
                acc.locked.to_string(),
            ])
            .map_err(OutputError::from)?;
    }

    writer.flush()?;

    Ok(())
}
