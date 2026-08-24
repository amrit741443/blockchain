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
        Address::from(self.keypair.public_key())
    }
}
