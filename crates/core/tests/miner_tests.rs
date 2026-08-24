use core::{
    blockchain::Blockchain, constants::DEFAULT_DIFFICULTY, mempool::Mempool, miner::Miner,
    transaction::Transaction,
};

use crypto::{Address, Keypair};

#[test]
fn test_miner_mines_prioritized_transactions() {
    let mut blockchain = Blockchain::new(DEFAULT_DIFFICULTY).unwrap();

    let mut mempool = Mempool::new();

    let miner_keys = Keypair::generate();
    let miner = Miner::new(miner_keys);

    let alice = Keypair::generate();
    let bob = Keypair::generate();

    let alice_addr = Address::from(alice.public_key());

    // Give Alice funds.
    let _result = blockchain.state_mut().credit(alice_addr, 5000);

    // Alice nonce 0, low fee.
    let tx1 = Transaction::new(&alice, bob.public_key(), 100, 0, 5).unwrap();

    // Alice nonce 1, high fee.
    let tx2 = Transaction::new(&alice, bob.public_key(), 100, 1, 50).unwrap();

    mempool.add_transaction(tx1, blockchain.state()).unwrap();

    mempool.add_transaction(tx2, blockchain.state()).unwrap();

    assert_eq!(mempool.len(), 2);

    // Miner creates the next block.
    let block = miner
        .mine_next_block(&mut blockchain, &mut mempool, 2)
        .unwrap();

    // Both transactions should be included.
    assert_eq!(block.transactions.len(), 2);

    // Nonce order must be respected.
    assert_eq!(block.transactions[0].nonce(), 0);
    assert_eq!(block.transactions[1].nonce(), 1);

    // Blockchain should now contain the mined block.
    assert_eq!(blockchain.height(), 1);

    // Mined transactions should be removed.
    assert_eq!(mempool.len(), 0);
}
