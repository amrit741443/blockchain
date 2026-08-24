use std::collections::BTreeMap;

use crypto::Hash;

use crate::error::{AccountError, TransactionError};

#[derive(Debug, Default, Clone)]
pub struct AccountQueue {
    next_accepted_nonce: u64,
    pub transactions: BTreeMap<u64, Hash>,
}

impl AccountQueue {
    pub fn new(nonce: u64) -> Self {
        Self {
            next_accepted_nonce: nonce,
            transactions: BTreeMap::new(),
        }
    }

    pub fn next_accepted_nonce(&self) -> u64 {
        self.next_accepted_nonce
    }

    pub fn transactions(&self) -> &BTreeMap<u64, Hash> {
        &self.transactions
    }

    /*
     * Return the first transaction in nonce order.
     * this is the next executable transaction for this account.
     *
     */
    pub fn front(&self) -> Option<(u64, Hash)> {
        self.transactions
            .first_key_value()
            .map(|(nonce, hash)| (*nonce, hash.clone()))
    }

    /*
     * Removes and returns the first transaction in nonce order.
     * IMPORTANT: This does not modify 'next_accepted_nonce'
     * 'next accepted nonce' represent the next nonce that can be accepted from the user, not the next transaction to execute.
     */
    pub fn pop_front(&mut self) -> Option<(u64, Hash)> {
        self.transactions.pop_first()
    }

    /*
     * Adds a transaction to this account's queue.
     */
    pub fn push(&mut self, nonce: u64, tx_hash: Hash) -> Result<(), TransactionError> {
        if nonce != self.next_accepted_nonce {
            return Err(TransactionError::InvalidNonce {
                expected: self.next_accepted_nonce,
                got: nonce,
            });
        }

        self.transactions.insert(nonce, tx_hash);

        self.next_accepted_nonce = self
            .next_accepted_nonce
            .checked_add(1)
            .ok_or(AccountError::NonceOverflow)?;

        Ok(())
    }
}
