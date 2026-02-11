use serde::Deserialize;

use crate::{
    engine::{
        id::{ClientId, TransactionId},
        transaction::{Transaction, TxKind},
    },
    error::source_error::SourceError, util::amount::parse_amount_fixed_decimal,
};

#[derive(Debug, Deserialize)]
pub struct CsvTransaction {
    #[serde(rename = "type")]
    pub kind: String,
    pub client: u16,
    pub tx: u32,
    pub amount: Option<String>,
}

impl TryFrom<CsvTransaction> for Transaction {
    type Error = SourceError;

    fn try_from(row: CsvTransaction) -> Result<Self, Self::Error> {
        let kind = match row.kind.trim() {
            "deposit" => TxKind::Deposit,
            "withdrawal" => TxKind::Withdrawal,
            "dispute" => TxKind::Dispute,
            "resolve" => TxKind::Resolve,
            "chargeback" => TxKind::Chargeback,
            other => return Err(SourceError::InvalidRecord(format!("unknown type: {other}"))),
        };

        let amount = match kind {
            TxKind::Deposit | TxKind::Withdrawal => {
                // For deposit and withdrawal transactions, amount is required. For other transaction types, amount should be ignored.
                let s = row.amount.ok_or_else(|| {
                    SourceError::InvalidRecord(format!("missing amount for tx {}", row.tx))
                })?;

                // Get amount's mantissa (i128).
                let amount = parse_amount_fixed_decimal(&s).map_err(
                    |e| SourceError::InvalidRecord(format!("invalid amount for tx {}: {e}", row.tx))
                )?;

                // Since we now that amount is scaled by 10^4, we can safely convert it to i64, but we should check for overflow.
                Some(amount.try_into().map_err(|_| {
                    SourceError::InvalidRecord(format!("amount overflow for tx {}", row.tx))
                })?)
            }
            _ => None,
        };

        Ok(Transaction {
            kind,
            client: ClientId::from_u16(row.client),
            tx: TransactionId::from_u32(row.tx),
            amount,
        })
    }
}