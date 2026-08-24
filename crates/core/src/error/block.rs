use thiserror::Error;

use crate::error::TransactionError;

#[derive(Debug, Error)]
pub enum BlockError {
    #[error("failed to calculate transaction hash: {0}")]
    InvalidTransactionHash(#[from] TransactionError),

    #[error("invalid Merkle root")]
    InvalidMerkleRoot,

    #[error("invalid block hash")]
    InvalidBlockHash,

    #[error("invalid proof of work")]
    InvalidProofOfWork,

    #[error("difficulty exceeds hash size")]
    InvalidDifficulty,

    #[error("Invalid block timestamp")]
    InvalidTimestamp,
}
