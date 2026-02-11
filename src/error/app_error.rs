use thiserror::Error;

use super::{engine_error::EngineError, output_error::OutputError, source_error::SourceError};

#[derive(Debug, Error)]
pub enum AppError {
    #[error("missing input file path")]
    MissingInput,

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Source(#[from] SourceError),

    #[error(transparent)]
    Output(#[from] OutputError),

    #[error(transparent)]
    Engine(#[from] EngineError),
}
