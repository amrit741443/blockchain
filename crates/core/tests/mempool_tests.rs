use core::{
    blockchain::Blockchain, constants::DEFAULT_DIFFICULTY, mempool::Mempool,
    transaction::Transaction,
};

use crypto::{Address, Keypair};

#[test]
fn test_mempool_fee_priority_respects_nonce_order() {
    let mut blockchain = Blockchain::new(DEFAULT_DIFFICULTY).unwrap();

    let mut mempool = Mempool::new();

    let alice = Keypair::generate();
    let bob = Keypair::generate();

    let alice_addr = Address::from(alice.public_key());
    let bob_addr = Address::from(bob.public_key());

    // Give both accounts enough balance.
    blockchain.state_mut().credit(alice_addr, 5000);

    blockchain.state_mut().credit(bob_addr, 5000);

    // ---------------------------------------------------------
    // Alice:
    //
    // nonce 0 -> fee 5
    // nonce 1 -> fee 50
    //
    // Alice nonce 1 has the highest fee,
    // but it cannot execute before nonce 0.
    // ---------------------------------------------------------

    let alice_tx_0 = Transaction::new(&alice, bob.public_key(), 100, 0, 5).unwrap();

    let alice_tx_1 = Transaction::new(&alice, bob.public_key(), 100, 1, 50).unwrap();

    // ---------------------------------------------------------
    // Bob:
    //
    // nonce 0 -> fee 30
    // ---------------------------------------------------------

    let bob_tx_0 = Transaction::new(&bob, alice.public_key(), 100, 0, 30).unwrap();

    // Add Alice transactions.
    mempool
        .add_transaction(alice_tx_0, blockchain.state())
        .unwrap();

    mempool
        .add_transaction(alice_tx_1, blockchain.state())
        .unwrap();

    // Add Bob transaction.
    mempool
        .add_transaction(bob_tx_0, blockchain.state())
        .unwrap();

    assert_eq!(mempool.len(), 3);

    // ---------------------------------------------------------
    // Select transactions according to priority.
    // ---------------------------------------------------------

    let txs = mempool.get_prioritized_transactions(blockchain.state(), 3);

    assert_eq!(txs.len(), 3);

    // ---------------------------------------------------------
    // Expected order:
    //
    // 1. Bob   nonce 0 -> fee 30
    // 2. Alice nonce 0 -> fee 5
    // 3. Alice nonce 1 -> fee 50
    //
    // Alice nonce 1 cannot be selected first because
    // Alice nonce 0 must be executed first.
    // ---------------------------------------------------------

    assert_eq!(txs[0].fee(), 30);
    assert_eq!(txs[1].fee(), 5);
    assert_eq!(txs[2].fee(), 50);
}

#[test]
fn test_priority_selection_does_not_modify_mempool() {
    let mut blockchain = Blockchain::new(DEFAULT_DIFFICULTY).unwrap();

    let mut mempool = Mempool::new();

    let alice = Keypair::generate();
    let bob = Keypair::generate();

    let alice_addr = Address::from(alice.public_key());
    let bob_addr = Address::from(bob.public_key());

    blockchain.state_mut().credit(alice_addr, 5000);

    blockchain.state_mut().credit(bob_addr, 5000);

    let alice_tx_0 = Transaction::new(&alice, bob.public_key(), 100, 0, 5).unwrap();

    let alice_tx_1 = Transaction::new(&alice, bob.public_key(), 100, 1, 50).unwrap();

    mempool
        .add_transaction(alice_tx_0, blockchain.state())
        .unwrap();

    mempool
        .add_transaction(alice_tx_1, blockchain.state())
        .unwrap();

    assert_eq!(mempool.len(), 2);

    // First selection.
    let txs = mempool.get_prioritized_transactions(blockchain.state(), 2);

    assert_eq!(txs.len(), 2);

    // The actual mempool must NOT be modified.
    assert_eq!(mempool.len(), 2);

    // Select again.
    let txs_again = mempool.get_prioritized_transactions(blockchain.state(), 2);

    assert_eq!(txs_again.len(), 2);

    // Both selections should produce the same order.
    assert_eq!(txs[0].nonce(), txs_again[0].nonce());
    assert_eq!(txs[0].fee(), txs_again[0].fee());

    assert_eq!(txs[1].nonce(), txs_again[1].nonce());
    assert_eq!(txs[1].fee(), txs_again[1].fee());
}

#[test]
fn test_mempool_rejects_wrong_nonce() {
    let mut blockchain = Blockchain::new(DEFAULT_DIFFICULTY).unwrap();

    let mut mempool = Mempool::new();

    let alice = Keypair::generate();
    let bob = Keypair::generate();

    let alice_addr = Address::from(alice.public_key());

    blockchain.state_mut().credit(alice_addr, 5000);

    // Alice's state nonce starts at 0.
    //
    // Trying to submit nonce 1 first should fail.

    let tx = Transaction::new(&alice, bob.public_key(), 100, 1, 10).unwrap();

    let result = mempool.add_transaction(tx, blockchain.state());

    assert!(result.is_err());

    // Nothing should have entered the mempool.
    assert_eq!(mempool.len(), 0);
}

#[test]
fn test_mempool_accepts_sequential_nonces() {
    let mut blockchain = Blockchain::new(DEFAULT_DIFFICULTY).unwrap();

    let mut mempool = Mempool::new();

    let alice = Keypair::generate();
    let bob = Keypair::generate();

    let alice_addr = Address::from(alice.public_key());

    blockchain.state_mut().credit(alice_addr, 5000);

    // Alice nonce 0.
    let tx_0 = Transaction::new(&alice, bob.public_key(), 100, 0, 10).unwrap();

    // Alice nonce 1.
    let tx_1 = Transaction::new(&alice, bob.public_key(), 100, 1, 20).unwrap();

    assert!(mempool.add_transaction(tx_0, blockchain.state()).is_ok());

    assert!(mempool.add_transaction(tx_1, blockchain.state()).is_ok());

    assert_eq!(mempool.len(), 2);
}

#[test]
fn test_mempool_rejects_nonce_gap() {
    let mut blockchain = Blockchain::new(DEFAULT_DIFFICULTY).unwrap();

    let mut mempool = Mempool::new();

    let alice = Keypair::generate();
    let bob = Keypair::generate();

    let alice_addr = Address::from(alice.public_key());

    blockchain.state_mut().credit(alice_addr, 5000);

    // First submit nonce 0.
    let tx_0 = Transaction::new(&alice, bob.public_key(), 100, 0, 10).unwrap();

    mempool.add_transaction(tx_0, blockchain.state()).unwrap();

    // Now nonce 2 is invalid because nonce 1 is missing.
    let tx_2 = Transaction::new(&alice, bob.public_key(), 100, 2, 20).unwrap();

    let result = mempool.add_transaction(tx_2, blockchain.state());

    assert!(result.is_err());

    // Only nonce 0 should be present.
    assert_eq!(mempool.len(), 1);
}

#[test]
fn test_mempool_balance_reservation() {
    let mut blockchain = Blockchain::new(DEFAULT_DIFFICULTY).unwrap();

    let mut mempool = Mempool::new();

    let alice = Keypair::generate();
    let bob = Keypair::generate();

    let alice_addr = Address::from(alice.public_key());

    // Alice only has 150.
    blockchain.state_mut().credit(alice_addr, 150);

    // Cost = amount + fee = 100 + 10 = 110.
    let tx_0 = Transaction::new(&alice, bob.public_key(), 100, 0, 10).unwrap();

    mempool.add_transaction(tx_0, blockchain.state()).unwrap();

    // Another transaction:
    //
    // Cost = 100 + 10 = 110.
    //
    // Total would be:
    //
    // 110 + 110 = 220
    //
    // Alice only has 150, so this must fail.

    let tx_1 = Transaction::new(&alice, bob.public_key(), 100, 1, 10).unwrap();

    let result = mempool.add_transaction(tx_1, blockchain.state());

    assert!(result.is_err());

    // Only the first transaction remains.
    assert_eq!(mempool.len(), 1);
}
