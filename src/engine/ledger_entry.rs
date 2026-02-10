use super::id::{ClientId, TransactionId};

#[derive(Debug, Clone)]
pub struct LedgerEntry {
    #[allow(dead_code)] // Normally should be used, but not in our example: useful for debugging.
    pub tx: TransactionId,
    pub client: ClientId,
    pub amount: i64,
    pub disputed: bool,
    pub charged_back: bool,
}
