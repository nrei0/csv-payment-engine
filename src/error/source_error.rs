use thiserror::Error;

#[derive(Debug, Error)]
pub enum SourceError {
    #[error("csv error: {0}")]
    Csv(#[from] csv::Error),

    #[error("invalid record: {0}")]
    InvalidRecord(String),
}
