use thiserror::Error;

use super::{engine_error::EngineError, source_error::SourceError};

#[derive(Debug, Error)]
pub enum AppError {
    #[error("missing input file path")]
    MissingInput,

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Source(#[from] SourceError),

    #[error(transparent)]
    Engine(#[from] EngineError),
}
