// state.rs

use thiserror::Error;

use crate::error::{AccountError, BlockError, TransactionError};

#[derive(Debug, Error)]
pub enum StateError {
    #[error("account  not found")]
    AccountNotFound,

    #[error("invalid nonce for account: expected {expected}, got {got}")]
    InvalidNonce { expected: u64, got: u64 },

    #[error("transaction validation failed: {0}")]
    InvalidTransaction(#[from] TransactionError),

    #[error("account operation failed: {0}")]
    Account(#[from] AccountError),

    #[error("Block operation failed: {0}")]
    Block(#[from] BlockError),

    #[error("Total transaction fees overflowed")]
    FeeOverflow,

    #[error("Invalid block height: current {current}, expected {expected}, got {got}")]
    InvalidBlockHeight {
        current: u64,
        expected: u64,
        got: u64,
    },

    #[error("Invalid previous block hash: expected {expected}, got {got}")]
    InvalidPreviousHash { expected: String, got: String },
}
