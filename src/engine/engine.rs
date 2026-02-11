use std::collections::HashMap;

use crate::error::engine_error::EngineError;

use super::{
    account::Account,
    id::{ClientId, TransactionId},
    ledger_entry::LedgerEntry,
    transaction::{Transaction, TxKind},
};

pub struct Engine {
    accounts: HashMap<ClientId, Account>,
    ledger: HashMap<TransactionId, LedgerEntry>,
}

impl Engine {
    pub fn new() -> Self {
        Engine {
            accounts: HashMap::new(),
            ledger: HashMap::new(),
        }
    }

    pub fn accounts(&self) -> impl Iterator<Item = &Account> {
        self.accounts.values()
    }

    fn account_mut_or_create(&mut self, client: ClientId) -> &mut Account {
        self.accounts.entry(client).or_insert_with(|| Account {
            client,
            available: 0,
            held: 0,
            locked: false,
        })
    }

    fn account_mut_existing(&mut self, client: ClientId) -> Option<&mut Account> {
        self.accounts.get_mut(&client)
    }

    pub fn apply(&mut self, transaction: &Transaction) -> Result<(), EngineError> {
        match transaction.kind {
            TxKind::Deposit => self.apply_deposit(transaction),
            TxKind::Withdrawal => self.apply_withdrawal(transaction),
            TxKind::Dispute => self.apply_dispute(transaction),
            TxKind::Resolve => self.apply_resolve(transaction),
            TxKind::Chargeback => self.apply_chargeback(transaction),
        }
    }

    fn apply_deposit(&mut self, transaction: &Transaction) -> Result<(), EngineError> {
        // Check for duplicate transaction.
        if self.ledger.contains_key(&transaction.tx) {
            return Err(EngineError::DuplicateTransaction(
                transaction.tx.to_string(),
            ));
        }

        // Amount is required for deposits and withdrawals.
        let amount = transaction
            .amount
            .ok_or_else(|| EngineError::MissingAmount(transaction.tx.to_string()))?;

        // Amount should be > 0.
        if amount <= 0 {
            return Err(EngineError::InvalidAmount(transaction.tx.to_string()));
        }

        // Account is locked.
        let account = self.account_mut_or_create(transaction.client);
        if account.locked {
            return Err(EngineError::AccountLocked(transaction.tx.to_string()));
        }

        // Apply the deposit.
        account.available += amount;

        // Write to ledger to prevent duplicates.
        self.ledger.insert(
            transaction.tx,
            LedgerEntry {
                tx: transaction.tx,
                client: transaction.client,
                kind: transaction.kind,
                amount,
                disputed: false,
                charged_back: false,
            },
        );

        Ok(())
    }

    fn apply_withdrawal(&mut self, transaction: &Transaction) -> Result<(), EngineError> {
        // Check for duplicate transaction.
        if self.ledger.contains_key(&transaction.tx) {
            return Err(EngineError::DuplicateTransaction(
                transaction.tx.to_string(),
            ));
        }

        // Amount is required for deposits and withdrawals.
        let amount = transaction
            .amount
            .ok_or_else(|| EngineError::MissingAmount(transaction.tx.to_string()))?;

        // Amount should be > 0.
        if amount <= 0 {
            return Err(EngineError::InvalidAmount(transaction.tx.to_string()));
        }

        // Account is locked.
        let account = self.account_mut_or_create(transaction.client);
        if account.locked {
            return Err(EngineError::AccountLocked(transaction.tx.to_string()));
        }

        // Insufficient funds.
        if account.available < amount {
            return Err(EngineError::InsufficientFunds(transaction.tx.to_string()));
        }

        // Apply the withdrawal.
        account.available -= amount;

        // Write to ledger to prevent duplicates.
        self.ledger.insert(
            transaction.tx,
            LedgerEntry {
                tx: transaction.tx,
                client: transaction.client,
                kind: transaction.kind,
                amount,
                disputed: false,
                charged_back: false,
            },
        );

        Ok(())
    }

    fn apply_dispute(&mut self, transaction: &Transaction) -> Result<(), EngineError> {
        // Validate that the transaction exists and belongs to the same client, and is not already disputed or charged back.
        // Retrieve the amount and original kind from the ledger entry for the disputed transaction.
        let (amount, original_kind) = {
            let entry = self.ledger.get(&transaction.tx).ok_or_else(|| {
                EngineError::DisputeNonExistentTransaction(transaction.tx.to_string())
            })?;

            // Check that the transaction belongs to the same client.
            if entry.client != transaction.client {
                return Err(EngineError::TransactionClientMismatch(
                    transaction.tx.to_string(),
                ));
            }

            // Check that the transaction is not already disputed.
            if entry.disputed {
                return Err(EngineError::TransactionAlreadyDisputed(
                    transaction.tx.to_string(),
                ));
            }

            // Check that the transaction is not already charged back.
            if entry.charged_back {
                return Err(EngineError::TransactionAlreadyChargedBack(
                    transaction.tx.to_string(),
                ));
            }

            (entry.amount, entry.kind)
        };

        // Account must exist for dispute/resolve/chargeback.
        let account = match self.account_mut_existing(transaction.client) {
            Some(a) => a,
            None => {
                return Err(EngineError::AccountNotFound(transaction.client.to_string()));
            }
        };

        // Account is locked.
        if account.locked {
            return Err(EngineError::AccountLocked(transaction.tx.to_string()));
        }

        match original_kind {
            TxKind::Deposit => {
                // Disputing a deposit: move funds from available to held.
                if account.available < amount {
                    return Err(EngineError::InsufficientFunds(transaction.tx.to_string()));
                }

                account.available -= amount;
                account.held += amount;
            }
            TxKind::Withdrawal => {
                // Disputing a withdrawal: temporarily restore withdrawn funds into held.
                account.held += amount;
            }
            _ => {
                return Err(EngineError::InvalidDisputeTarget(
                    transaction.tx.to_string(),
                ));
            }
        }

        // Mark the transaction as disputed.
        if let Some(entry) = self.ledger.get_mut(&transaction.tx) {
            entry.disputed = true;
        }

        Ok(())
    }

    fn apply_resolve(&mut self, transaction: &Transaction) -> Result<(), EngineError> {
        // Validate that the transaction exists and belongs to the same client, and is currently disputed and not charged back.
        // Retrieve the amount and original kind from the ledger entry for the resolved transaction.
        let (amount, original_kind) = {
            let entry = self.ledger.get(&transaction.tx).ok_or_else(|| {
                EngineError::ResolveNonExistentTransaction(transaction.tx.to_string())
            })?;

            // Check that the transaction belongs to the same client.
            if entry.client != transaction.client {
                return Err(EngineError::TransactionClientMismatch(
                    transaction.tx.to_string(),
                ));
            }

            // Check that the transaction is currently disputed.
            if !entry.disputed {
                return Err(EngineError::TransactionNotDisputed(
                    transaction.tx.to_string(),
                ));
            }

            // Check that the transaction is not already charged back.
            if entry.charged_back {
                return Err(EngineError::TransactionAlreadyChargedBack(
                    transaction.tx.to_string(),
                ));
            }

            (entry.amount, entry.kind)
        };

        // Account must exist for dispute/resolve/chargeback.
        let account = match self.account_mut_existing(transaction.client) {
            Some(a) => a,
            None => {
                return Err(EngineError::AccountNotFound(transaction.client.to_string()));
            }
        };

        // Account is locked.
        if account.locked {
            return Err(EngineError::AccountLocked(transaction.tx.to_string()));
        }

        // Must have enough held funds to resolve.
        if account.held < amount {
            return Err(EngineError::InsufficientHeldFunds(
                transaction.tx.to_string(),
            ));
        }

        match original_kind {
            TxKind::Deposit => {
                // Resolving a deposit dispute: move held back to available.
                account.held -= amount;
                account.available += amount;
            }
            TxKind::Withdrawal => {
                // Resolving a withdrawal dispute: remove the temporarily restored held funds.
                account.held -= amount;
            }
            _ => {
                return Err(EngineError::InvalidResolveTarget(
                    transaction.tx.to_string(),
                ));
            }
        }

        // Mark the transaction as no longer disputed.
        if let Some(entry) = self.ledger.get_mut(&transaction.tx) {
            entry.disputed = false;
        }

        Ok(())
    }

    fn apply_chargeback(&mut self, transaction: &Transaction) -> Result<(), EngineError> {
        // Validate that the transaction exists and belongs to the same client, and is currently disputed and not charged back.
        // Retrieve the amount and original kind from the ledger entry for the charged back transaction.
        let (amount, original_kind) = {
            let entry = self.ledger.get(&transaction.tx).ok_or_else(|| {
                EngineError::ChargebackNonExistentTransaction(transaction.tx.to_string())
            })?;

            // Check that the transaction belongs to the same client.
            if entry.client != transaction.client {
                return Err(EngineError::TransactionClientMismatch(
                    transaction.tx.to_string(),
                ));
            }

            // Check that the transaction is currently disputed.
            if !entry.disputed {
                return Err(EngineError::TransactionNotDisputed(
                    transaction.tx.to_string(),
                ));
            }

            // Check that the transaction is not already charged back.
            if entry.charged_back {
                return Err(EngineError::TransactionAlreadyChargedBack(
                    transaction.tx.to_string(),
                ));
            }

            (entry.amount, entry.kind)
        };

        // Account must exist for dispute/resolve/chargeback.
        let account = match self.account_mut_existing(transaction.client) {
            Some(a) => a,
            None => {
                return Err(EngineError::AccountNotFound(transaction.client.to_string()));
            }
        };

        // Account is locked.
        if account.locked {
            return Err(EngineError::AccountLocked(transaction.tx.to_string()));
        }

        // Must have enough held funds to charge back.
        if account.held < amount {
            return Err(EngineError::InsufficientHeldFunds(
                transaction.tx.to_string(),
            ));
        }

        match original_kind {
            TxKind::Deposit => {
                // Chargeback a deposit: held funds are withdrawn from the system.
                account.held -= amount;
            }

            TxKind::Withdrawal => {
                // Chargeback a withdrawal: reverse the withdrawal permanently.
                // The disputed amount is currently held (temporary restoration). Finalize by moving it to available.
                account.held -= amount;
                account.available += amount;
            }

            _ => {
                return Err(EngineError::InvalidChargebackTarget(
                    transaction.tx.to_string(),
                ));
            }
        }

        // Lock the account on chargeback.
        account.locked = true;

        // Mark the transaction as charged back and no longer disputed.
        if let Some(entry) = self.ledger.get_mut(&transaction.tx) {
            entry.disputed = false;
            entry.charged_back = true;
        }

        Ok(())
    }
}
