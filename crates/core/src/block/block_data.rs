use std::time::{SystemTime, UNIX_EPOCH};

use crypto::Hash;
use serde::{Deserialize, Serialize};

use crate::{error::BlockError, transaction::Transaction};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockHeader {
    pub index: u64,
    pub timestamp: u64,
    pub previous_hash: Hash,
    pub merkle_root: Hash,
    pub nonce: u64,
    pub difficulty: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
    pub hash: Hash,
}

impl BlockHeader {
    pub fn to_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).expect("Serializationfailed")
    }
    pub fn calculate_hash(&self) -> Hash {
        Hash::digest(&self.to_bytes())
    }
}

impl Block {
    pub fn new(
        index: u64,
        previous_hash: Hash,
        transactions: Vec<Transaction>,
        difficulty: usize,
        nonce: u64,
    ) -> Result<Self, BlockError> {
        let merkle_root = Self::calculate_merkle_root(&transactions)?;

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| BlockError::InvalidTimestamp)?
            .as_secs();

        let header = BlockHeader {
            index,
            timestamp,
            previous_hash,
            merkle_root,
            difficulty,
            nonce,
        };

        let mut block = Self {
            header,
            transactions,
            hash: Hash::default(),
        };
        block.mine(difficulty);

        Ok(block)
    }

    pub fn calculate_merkle_root(transactions: &[Transaction]) -> Result<Hash, BlockError> {
        if transactions.is_empty() {
            return Ok(Hash::digest(b"empty_block"));
        }

        let mut combined_bytes = Vec::with_capacity(transactions.len() * 32);

        for tx in transactions {
            let tx_hash = tx.signing_hash()?;

            combined_bytes.extend_from_slice(tx_hash.as_bytes());
        }

        Ok(Hash::digest(&combined_bytes))
    }

    pub fn satisfies_difficulty(hash: &Hash, difficulty: usize) -> Result<(), BlockError> {
        if hash.iter().take(difficulty).all(|&byte| byte == 0) {
            Ok(())
        } else {
            Err(BlockError::InvalidProofOfWork)
        }
    }

    pub fn genesis(difficulty: usize) -> Result<Self, BlockError> {
        Self::new(0, Hash::default(), Vec::new(), difficulty, 0)
    }

    pub fn is_valid(&self, difficulty: usize) -> Result<(), BlockError> {
        let expected_merkle = Self::calculate_merkle_root(&self.transactions)?;

        if self.header.merkle_root != expected_merkle {
            return Err(BlockError::InvalidMerkleRoot);
        }

        let calculated_hash = self.header.calculate_hash();

        if calculated_hash != self.hash {
            return Err(BlockError::InvalidBlockHash);
        }

        Self::satisfies_difficulty(&self.hash, difficulty)?;

        Ok(())
    }

    pub fn mine(&mut self, difficulty: usize) {
        loop {
            let hash = self.header.calculate_hash();

            if Self::satisfies_difficulty(&hash, difficulty).is_ok() {
                self.hash = hash;
                break;
            }

            self.header.nonce += 1;
        }
    }
}
