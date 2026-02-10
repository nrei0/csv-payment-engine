// @andy will it be better just to move closer to source?

use std::str::FromStr;

use rust_decimal::Decimal;
use serde::Deserialize;

use crate::{
    engine::{
        id::{ClientId, TransactionId},
        transaction::{Transaction, TxKind},
    },
    error::source_error::SourceError,
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
                let s = row.amount.ok_or_else(|| {
                    SourceError::InvalidRecord(format!("missing amount for tx {}", row.tx))
                })?;
                Some(parse_amount_fixed_decimal(&s)?)
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

// @andy move to separate mod.
fn parse_amount_fixed_decimal(s: &str) -> Result<i64, SourceError> {
    let s = s.trim();

    let mut d = Decimal::from_str(s)
        .map_err(|_| SourceError::InvalidRecord(format!("invalid amount: {s}")))?;

    d.rescale(4);

    d.mantissa()
        .try_into()
        .map_err(|_| SourceError::InvalidRecord(format!("amount out of range: {s}")))
}
