use thiserror::Error;

#[derive(Debug, Error)]
pub enum OutputError {
    #[error("csv output error: {0}")]
    Csv(#[from] csv::Error),
}
