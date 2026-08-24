use crypto::{Address, Keypair};

use crate::{block::Block, blockchain::Blockchain, error::StateError, mempool::Mempool};

pub struct Miner {
    keypair: Keypair,
}

impl Miner {
    pub fn new(keypair: Keypair) -> Self {
        Self { keypair }
    }
    pub fn address(&self) -> Address {
        Address(self.keypair.public_key())
    }

    pub fn mine_next_block(
        &self,
        blockchain: &mut Blockchain,
        mempool: &mut Mempool,
        limit: usize,
    ) -> Result<Block, StateError> {
        /*
         * Ask the mempool for highest-priority executable transactions
         */

        let transactions = mempool.get_prioritized_transactions(blockchain.state(), limit);

        /*
         * Get information needed to crate the next block.
         */

        let index = blockchain.height() + 1;
        let previous_hash = blockchain.latest_hash();
        let difficulty = blockchain.difficulty();

        /*
         * 3. Create the next block.
         * Block::new() calculates the merkle root and mines the block.
         */

        let block = Block::new(index, previous_hash, transactions, difficulty, 0)?;

        /*
         * 4. Submit the block to the blockchain.
         *  Blockchain is responsible for validating the block and adding it to the chain.
         */

        blockchain.add_block(block.clone(), &self.address())?;

        /*
         * Remove transaction that were successfully mined
         */

        mempool.remove_mined_transactions(&block.transactions, blockchain.state())?;

        Ok(block)
    }
}
