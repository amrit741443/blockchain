// account.rs

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AccountError {
    #[error("insufficient balance: required {required}, available {available}")]
    InsufficientBalance { required: u64, available: u64 },

    #[error("transaction cost overflow")]
    CostOverflow,

    #[error("account balance overflow")]
    BalanceOverflow,

    #[error("account nonce overflow")]
    NonceOverflow,

    #[error("account not found")]
    AccountNotFound,
}
