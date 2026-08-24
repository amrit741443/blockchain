use std::cmp::Ordering;

use crypto::{Address, Hash};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PriorityEntry {
    fee: u64,
    sender: Address,
    nonce: u64,
    tx_hash: Hash,
}

impl PriorityEntry {
    pub fn new(fee: u64, sender: Address, nonce: u64, tx_hash: Hash) -> Self {
        Self {
            fee,
            sender,
            nonce,
            tx_hash,
        }
    }

    pub fn fee(&self) -> u64 {
        self.fee
    }

    pub fn sender(&self) -> Address {
        self.sender
    }

    pub fn nonce(&self) -> u64 {
        self.nonce
    }

    pub fn tx_hash(&self) -> Hash {
        self.tx_hash
    }
}

impl Ord for PriorityEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.fee
            .cmp(&other.fee)
            .then_with(|| self.sender.cmp(&other.sender))
            .then_with(|| self.nonce.cmp(&other.nonce))
            .then_with(|| self.tx_hash.cmp(&other.tx_hash))
    }
}

impl PartialOrd for PriorityEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
