use crate::error::AccountError;
use crypto::CryptoError;
use thiserror::Error;

// transaction.rs

#[derive(Debug, Error)]
pub enum TransactionError {
    #[error("transaction amount must be greater than zero")]
    ZeroAmount,
    #[error("account operation failed: {0}")]
    Account(#[from] AccountError),

    #[error("sender and receiver cannot be the same")]
    SelfTransfer,

    #[error("invalid cryptographic signature: {0}")]
    InvalidSignature(#[source] CryptoError),

    #[error("duplicate transaction")]
    Duplicate,

    #[error("invalid nonce: expected:{expected} got:{got}")]
    InvalidNonce { expected: u64, got: u64 },

    #[error("failed to serialize transaction payload: {0}")]
    Serialization(#[from] bincode::Error),
}
