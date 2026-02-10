use csv::{DeserializeRecordsIntoIter, Trim};
use std::io::{BufReader, Read};

use crate::{
    dto::csv_transaction::CsvTransaction, engine::transaction::Transaction,
    error::source_error::SourceError,
};

pub struct CsvSource<R: Read> {
    inner_iter: DeserializeRecordsIntoIter<BufReader<R>, CsvTransaction>,
}

impl<R: Read> CsvSource<R> {
    pub fn new(reader: R) -> Self {
        let reader = csv::ReaderBuilder::new()
            .trim(Trim::All)
            .from_reader(BufReader::new(reader));

        Self {
            inner_iter: reader.into_deserialize(),
        }
    }
}

impl<R: Read> Iterator for CsvSource<R> {
    type Item = Result<Transaction, SourceError>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(
            self.inner_iter
                .next()?
                .map_err(SourceError::from)
                .and_then(|dto| dto.try_into()),
        )
    }
}
