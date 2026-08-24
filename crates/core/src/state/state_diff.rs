use std::collections::HashMap;

use crypto::Address;

use crate::{account::Account, state::State};

#[derive(Debug, Default, Clone)]
pub struct StateDiff {
    accounts: HashMap<Address, Account>,
}

impl StateDiff {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, address: &Address) -> Option<&Account> {
        self.accounts.get(address)
    }

    pub fn insert(&mut self, address: Address, account: Account) {
        self.accounts.insert(address, account);
    }

    pub fn is_empty(&self) -> bool {
        self.accounts.is_empty()
    }

    pub fn commit_into(self, state: &mut State) {
        for (address, account) in self.accounts {
            state.set_account(address, account);
        }
    }
}
