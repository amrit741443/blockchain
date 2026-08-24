use crate::error::AccountError;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Account {
    balance: u64,
    nonce: u64,
}

impl Account {
    pub fn new(balance: u64, nonce: u64) -> Self {
        Self { balance, nonce }
    }

    pub fn nonce(&self) -> u64 {
        self.nonce
    }
    pub fn balance(&self) -> u64 {
        self.balance
    }

    pub fn increment_nonce(&mut self) -> Result<(), AccountError> {
        self.nonce = self
            .nonce
            .checked_add(1)
            .ok_or(AccountError::NonceOverflow)?;
        Ok(())
    }

    pub fn deposit(&mut self, amount: u64) -> Result<(), AccountError> {
        self.balance = self
            .balance
            .checked_add(amount)
            .ok_or(AccountError::BalanceOverflow)?;
        Ok(())
    }

    pub fn can_deposit(&self, amount: u64) -> Result<(), AccountError> {
        self.balance
            .checked_add(amount)
            .ok_or(AccountError::BalanceOverflow)?;

        Ok(())
    }

    pub fn can_withdraw(&self, amount: u64, fee: u64) -> Result<(), AccountError> {
        let total = amount.checked_add(fee).ok_or(AccountError::CostOverflow)?;

        if self.balance < total {
            return Err(AccountError::InsufficientBalance {
                required: total,
                available: self.balance,
            });
        }

        Ok(())
    }

    /// Deducts funds from the account if available
    pub fn withdraw(&mut self, amount: u64, fee: u64) -> Result<(), AccountError> {
        self.can_withdraw(amount, fee)?;

        let total = amount.checked_add(fee).ok_or(AccountError::CostOverflow)?;

        self.balance -= total;
        Ok(())
    }
}
