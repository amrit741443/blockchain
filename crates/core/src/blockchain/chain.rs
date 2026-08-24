use crypto::{Address, Hash};
use serde::{Deserialize, Serialize};

use crate::{
    block::Block,
    error::StateError,
    state::{ExecutionContext, State},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Blockchain {
    chain: Vec<Block>,
    state: State,
    difficulty: usize,
}

impl Blockchain {
    pub fn new(difficulty: usize) -> Result<Self, StateError> {
        let gensis_block = Block::genesis(difficulty)?;

        let state = State::new();

        let blockchain = Self {
            chain: vec![gensis_block],
            state,
            difficulty,
        };

        Ok(blockchain)
    }

    pub fn blocks(&self) -> &[Block] {
        &self.chain
    }

    pub fn latest_block(&self) -> &Block {
        self.chain
            .last()
            .expect("Chain must contain at least genesis block")
    }

    pub fn height(&self) -> u64 {
        self.state.block_height()
    }

    pub fn latest_hash(&self) -> Hash {
        self.latest_block().hash
    }

    pub fn state(&self) -> &State {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut State {
        &mut self.state
    }

    pub fn difficulty(&self) -> usize {
        self.difficulty
    }

    fn validate_block(&self, block: &Block) -> Result<(), StateError> {
        let latest_hash = self.latest_hash();

        if block.header.previous_hash != latest_hash {
            return Err(StateError::InvalidPreviousHash {
                expected: latest_hash.to_string(),
                got: block.header.previous_hash.to_string(),
            });
        }

        let expected_height = self.height() + 1;

        if block.header.index != expected_height {
            return Err(StateError::InvalidBlockHeight {
                current: self.height(),
                expected: expected_height,
                got: block.header.index,
            });
        }

        block.is_valid(self.difficulty)?;

        Ok(())
    }

    //---Block Processing
    pub fn add_block(&mut self, block: Block, miner_address: &Address) -> Result<(), StateError> {
        self.validate_block(&block)?;

        let mut context = ExecutionContext::new(&self.state);

        let total_fee = context.execute_transactions(&block.transactions)?;

        context.credit(*miner_address, total_fee)?;

        //commit
        let diff = context.into_diff();
        diff.commit_into(&mut self.state);

        //update blockchain metadata
        self.state
            .set_block_metadata(block.header.index, block.hash);

        //add block to chain

        self.chain.push(block);

        Ok(())
    }
}
