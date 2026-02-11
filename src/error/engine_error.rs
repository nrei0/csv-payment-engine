use thiserror::Error;

#[derive(Debug, Error)]
pub enum EngineError {
    #[error("duplicate transaction: {0}")]
    DuplicateTransaction(String),

    #[error("missing amount for transaction: {0}")]
    MissingAmount(String),

    #[error("invalid amount for transaction: {0}")]
    InvalidAmount(String),

    #[error("account is locked for transaction: {0}")]
    AccountLocked(String),

    #[error("account not found for transaction: {0}")]
    AccountNotFound(String),

    #[error("insufficient funds for transaction: {0}")]
    InsufficientFunds(String),

    #[error("insufficient held funds for transaction: {0}")]
    InsufficientHeldFunds(String),

    #[error("invalid dispute target for transaction: {0}")]
    InvalidDisputeTarget(String),

    #[error("invalid resolve target for transaction: {0}")]
    InvalidResolveTarget(String),

    #[error("invalid chargeback target for transaction: {0}")]
    InvalidChargebackTarget(String),

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
