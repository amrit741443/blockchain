use crate::{
    error::{AccountError, TransactionError},
    mempool::{AccountQueue, PriorityEntry},
    state::State,
    transaction::Transaction,
};
use crypto::{Address, Hash};
use std::collections::{BinaryHeap, HashMap};

#[derive(Clone, Debug, Default)]
pub struct Mempool {
    // *All transactions currently waiting in the mempool.
    pending_transactions: HashMap<Hash, Transaction>,

    // * One nonce-ordered queue for every sender.
    accounts: HashMap<Address, AccountQueue>,

    //* Highest-fee executable transactions.
    priority_queue: BinaryHeap<PriorityEntry>,

    // * Total funds reserved by pending transactions.
    pending_balances: HashMap<Address, u64>,
}

impl Mempool {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.pending_transactions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.pending_transactions.is_empty()
    }

    /*
     * ADD TRANSACTION
     */
    pub fn add_transaction(
        &mut self,
        tx: Transaction,
        state: &State,
    ) -> Result<(), TransactionError> {
        // *1. Verify signature
        tx.verify()?;
        let sender = Address::from(*tx.sender());

        // *2 Get committed account state.
        let account = state
            .get_account(&sender)
            .ok_or(AccountError::AccountNotFound)?;

        let state_nonce = account.nonce();
        let balance = account.balance();

        /*
         * 3. Determine which nonce this account should accept next.
         *
         * if this account has no pending transctions
         * start from blockchain state nonce.
         * Otherwise use this AccountQueue's next accepted nonce.
         *
         */
        let expected_nonce = self
            .accounts
            .get(&sender)
            .map(|q| q.next_accepted_nonce())
            .unwrap_or(state_nonce);

        // *4. Nonce must be exactly the expected nonce.
        if tx.nonce() != expected_nonce {
            return Err(TransactionError::InvalidNonce {
                expected: expected_nonce,
                got: tx.nonce(),
            });
        }

        /*
         * 5.  Calculate transaction cost.
         * cost = amount + fee
         */
        let cost = tx
            .amount()
            .checked_add(tx.fee())
            .ok_or(AccountError::CostOverflow)?;

        // *6. Calculate fund already reserved by pending transactions.
        let pending_const = self.pending_balances.get(&sender).copied().unwrap_or(0);

        let total_required = pending_const
            .checked_add(cost)
            .ok_or(AccountError::CostOverflow)?;

        // *7. Check account balance.
        if balance < total_required {
            return Err(AccountError::InsufficientBalance {
                required: total_required,
                available: balance,
            }
            .into());
        }

        // *8. clculate transaction hash
        let tx_hash = tx.tx_id()?;

        // *9. store transaction in pending transactions
        self.pending_transactions.insert(tx_hash, tx);

        // *10. Add transaction to sender nonce queue
        let queue = self
            .accounts
            .entry(sender)
            .or_insert_with(|| AccountQueue::new(state_nonce));

        queue.push(expected_nonce, tx_hash)?;

        // *11. Reserve funds
        self.pending_balances.insert(sender, total_required);

        Ok(())
    }

    /*
     * REMOVE MINED TRANSACTIONS
     */
    pub fn remove_mined_transactions(
        &mut self,
        mined_txs: &[Transaction],
        state: &State,
    ) -> Result<(), TransactionError> {
        for tx in mined_txs {
            let tx_hash = tx.tx_id()?;
            self.pending_transactions.remove(&tx_hash);
        }

        /*
         * Rebuld all account queues and reservations from the transaction that remains.
         */
        self.rebuild_reservations(state)?;

        Ok(())
    }

    /*
     * REBUILD RESERVATIONS
     */
    pub fn rebuild_reservations(&mut self, state: &State) -> Result<(), TransactionError> {
        self.accounts.clear();
        self.pending_balances.clear();
        self.priority_queue.clear();

        // *Group pending transaction by sender
        let mut transaction_by_sender: HashMap<Address, Vec<&Transaction>> = HashMap::new();

        for tx in self.pending_transactions.values() {
            let sender = Address::from(*tx.sender());

            transaction_by_sender.entry(sender).or_default().push(tx);
        }

        // *Rebuild every sender independently

        for (sender, mut transactions) in transaction_by_sender {
            let account = state
                .get_account(&sender)
                .ok_or(AccountError::AccountNotFound)?;

            let state_nonce = account.nonce();
            let balance = account.balance();

            //Nonces must be processed in ascending order
            transactions.sort_by_key(|tx| tx.nonce());

            let mut queue = AccountQueue::new(state_nonce);
            let mut reserved = 0u64;

            for tx in transactions {
                let expected_nonce = queue.next_accepted_nonce();

                /*
                 * Only accept contiguous nonce sequences
                 * Example
                 * state nonce 5
                 * 5-6 accepted
                 * 8 rejected because 7 was not accepted
                 */

                if tx.nonce() != expected_nonce {
                    continue;
                }

                let cost = tx
                    .amount()
                    .checked_add(tx.fee())
                    .ok_or(AccountError::CostOverflow)?;

                let new_reserved = reserved
                    .checked_add(cost)
                    .ok_or(AccountError::CostOverflow)?;

                if new_reserved > balance {
                    continue;
                }

                let tx_hash = tx.tx_id()?;

                queue.push(tx.nonce(), tx_hash)?;
                reserved = new_reserved;
            }

            if !queue.transactions().is_empty() {
                self.accounts.insert(sender, queue);
            }

            if reserved > 0 {
                self.pending_balances.insert(sender, reserved);
            }
        }

        Ok(())
    }

    /*
     * PRIORITY TRANSACTIONS
     */
    pub fn get_prioritized_transactions(&self, _state: &State, limit: usize) -> Vec<Transaction> {
        let mut result = Vec::with_capacity(limit);

        /*
         * Clone the account queues because we are only simulating
         * transaction execution.
         *
         * We MUST mutate these temporary queues while selecting
         * transactions.
         */
        let mut queues = self.accounts.clone();

        let mut heap: BinaryHeap<PriorityEntry> = BinaryHeap::new();

        /*
         * STEP 1
         *
         * Add the first executable transaction from each account.
         */
        for (sender, queue) in &queues {
            let Some((nonce, tx_hash)) = queue.front() else {
                continue;
            };

            let Some(tx) = self.pending_transactions.get(&tx_hash) else {
                continue;
            };

            heap.push(PriorityEntry::new(tx.fee(), *sender, nonce, tx_hash));
        }

        /*
         * STEP 2
         *
         * Repeatedly select the highest-fee executable transaction.
         */
        while result.len() < limit {
            let Some(entry) = heap.pop() else {
                break;
            };

            let sender = entry.sender();
            let nonce = entry.nonce();
            let tx_hash = entry.tx_hash();

            /*
             * Get the temporary account queue.
             *
             * We need mutable access because the selected transaction
             * will be removed from this temporary queue.
             */
            let Some(queue) = queues.get_mut(&sender) else {
                continue;
            };

            /*
             * Check for a stale heap entry.
             */
            let Some((current_nonce, current_hash)) = queue.front() else {
                continue;
            };

            if current_nonce != nonce || current_hash != tx_hash {
                continue;
            }

            /*
             * Find the actual transaction.
             */
            let Some(tx) = self.pending_transactions.get(&tx_hash) else {
                continue;
            };

            /*
             * Select transaction.
             */
            result.push(tx.clone());

            /*
             * IMPORTANT:
             *
             * Remove the selected transaction from the
             * TEMPORARY account queue.
             *
             * We do NOT modify self.accounts.
             */
            queue.transactions.remove(&nonce);

            /*
             * Now the next transaction from this account
             * may become executable.
             */
            if let Some((next_nonce, next_hash)) = queue.front()
                && let Some(next_tx) = self.pending_transactions.get(&next_hash)
            {
                heap.push(PriorityEntry::new(
                    next_tx.fee(),
                    sender,
                    next_nonce,
                    next_hash,
                ));
            }
        }

        result
    }

    /*
     * NONCE ORDERED TRANSACTIONS
     */
    pub fn get_nonce_ordered_transactions(&self, limit: usize) -> Vec<Transaction> {
        let mut result = Vec::with_capacity(limit);

        for queue in self.accounts.values() {
            for tx_hash in queue.transactions().values() {
                if result.len() < limit {
                    return result;
                }
                if let Some(tx) = self.pending_transactions.get(tx_hash) {
                    result.push(tx.clone());
                }
            }
        }

        result
    }
}
