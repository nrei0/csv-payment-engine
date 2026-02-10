use thiserror::Error;

#[derive(Debug, Error)]
pub enum EngineError {
    #[error("missing amount for transaction: {0}")]
    MissingAmount(String),

    #[error("account is locked for transaction: {0}")]
    AccountLocked(String),

    #[error("account not found for transaction: {0}")]
    AccountNotFound(String),

    #[error("insufficient funds for transaction: {0}")]
    InsufficientFunds(String),

    #[error("insufficient held funds for transaction: {0}")]
    InsufficientHeldFunds(String),

    #[error("dispute on non-existent transaction: {0}")]
    DisputeNonExistentTransaction(String),

    #[error("resolve on non-existent transaction: {0}")]
    ResolveNonExistentTransaction(String),

    #[error("chargeback on non-existent transaction: {0}")]
    ChargebackNonExistentTransaction(String),

    #[error("dispute on transaction from different client: {0}")]
    TransactionClientMismatch(String),

    #[error("transaction already disputed: {0}")]
    TransactionAlreadyDisputed(String),

    #[error("transaction not disputed: {0}")]
    TransactionNotDisputed(String),

    #[error("transaction already charged back: {0}")]
    TransactionAlreadyChargedBack(String),
}
