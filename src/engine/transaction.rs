use super::id::{ClientId, TransactionId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TxKind {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Debug, Clone, Copy)]
pub struct Transaction {
    pub kind: TxKind,
    pub client: ClientId,
    pub tx: TransactionId,
    pub amount: Option<i64>,
}
