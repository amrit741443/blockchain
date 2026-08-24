use crypto::Address;

use crate::{
    account::Account,
    error::StateError,
    state::{State, StateDiff},
    transaction::Transaction,
};

pub struct ExecutionContext<'a> {
    state: &'a State,
    diff: StateDiff,
}

impl<'a> ExecutionContext<'a> {
    pub fn new(state: &'a State) -> Self {
        Self {
            state,
            diff: StateDiff::new(),
        }
    }

    pub fn get_account(&self, address: &Address) -> Option<Account> {
        self.diff
            .get(address)
            .copied()
            .or_else(|| self.state.get_account(address).copied())
    }

    fn get_required_account(&self, address: &Address) -> Result<Account, StateError> {
        self.get_account(address).ok_or(StateError::AccountNotFound)
    }

    pub fn set_account(&mut self, address: Address, account: Account) {
        self.diff.insert(address, account);
    }

    pub fn credit(&mut self, address: Address, amount: u64) -> Result<(), StateError> {
        let mut account = self.get_account(&address).unwrap_or_default();
        account.deposit(amount)?;
        self.set_account(address, account);

        Ok(())
    }

    fn validate_transaction(&self, tx: &Transaction) -> Result<(), StateError> {
        tx.verify()?;

        let sender_address = Address::from(*tx.sender());
        let receiver_address = Address::from(*tx.receiver());

        //Imp
        //This reads stateDiff first, the State,
        let sender = self.get_required_account(&sender_address)?;

        if sender.nonce() != tx.nonce() {
            return Err(StateError::InvalidNonce {
                expected: sender.nonce(),
                got: tx.nonce(),
            });
        }

        //it see in state diff
        sender.can_withdraw(tx.amount(), tx.fee())?;

        let receiver = self.get_account(&receiver_address).unwrap_or_default();

        receiver.can_deposit(tx.amount())?;

        Ok(())
    }

    pub fn execute_transaction(&mut self, tx: &Transaction) -> Result<u64, StateError> {
        self.validate_transaction(tx)?;

        let sender_address = Address::from(*tx.sender());
        let receiver_address = Address::from(*tx.receiver());

        let mut sender = self.get_required_account(&sender_address)?;
        let mut receiver = self.get_account(&receiver_address).unwrap_or_default();

        //apply state transaction to local copies.
        sender.withdraw(tx.amount(), tx.fee())?;
        sender.increment_nonce()?;

        receiver.deposit(tx.amount())?;

        //only write the changes into the temporary diff.
        self.set_account(sender_address, sender);
        self.set_account(receiver_address, receiver);

        Ok(tx.fee())
    }

    pub fn execute_transactions(
        &mut self,
        transactions: &[Transaction],
    ) -> Result<u64, StateError> {
        let mut total_fee = 0u64;
        for tx in transactions {
            let fee = self.execute_transaction(tx)?;
            total_fee = total_fee.checked_add(fee).ok_or(StateError::FeeOverflow)?;
        }

        Ok(total_fee)
    }

    pub fn into_diff(self) -> StateDiff {
        self.diff
    }
}
