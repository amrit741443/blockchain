use crate::{account::Account, error::StateError};
use crypto::{Address, Hash};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct State {
    accounts: HashMap<Address, Account>,
    block_height: u64,
    last_block_hash: Hash,
}

impl State {
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
            block_height: 0,
            last_block_hash: Hash::default(),
        }
    }

    //---getters---
    pub fn block_height(&self) -> u64 {
        self.block_height
    }

    pub fn last_block_hash(&self) -> Hash {
        self.last_block_hash
    }

    pub fn get_account(&self, address: &Address) -> Option<&Account> {
        self.accounts.get(address)
    }

    pub fn get_balance(&self, address: &Address) -> Option<u64> {
        self.accounts.get(address).map(Account::balance)
    }

    pub fn get_nonce(&self, address: &Address) -> Option<u64> {
        self.get_account(address).map(Account::nonce)
    }

    pub(crate) fn set_account(&mut self, address: Address, account: Account) {
        self.accounts.insert(address, account);
    }

    pub(crate) fn set_block_metadata(&mut self, height: u64, hash: Hash) {
        self.block_height = height;
        self.last_block_hash = hash;
    }

    // ---STATE MUTATIONS--

    //Credits an address directly (used for genesis allocations or block miner rewards)
    pub fn credit(&mut self, address: Address, amount: u64) -> Result<(), StateError> {
        let account = self.accounts.entry(address).or_default();

        account.deposit(amount)?;

        Ok(())
    }

    /// Generates a deterministic State Root Hash across all account balances
    pub fn state_root(&self) -> Hash {
        let mut sorted_accounts: Vec<(&Address, &Account)> = self.accounts.iter().collect();
        sorted_accounts.sort_by(|a, b| a.0.cmp(b.0));

        let bytes = bincode::serialize(&sorted_accounts).expect("Failed to serialize");
        Hash::digest(&bytes)
    }
}
