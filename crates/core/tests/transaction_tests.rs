// use core::{
//     account::Account,
//     error::{StateError, TransactionError},
//     interfaces::{Executable, IAccount, Verifiable},
//     transaction::Transaction,
// };

// use crypto::Keypair;

// #[test]
// fn test_valid_transaction_creation_and_execution() {
//     let alice_keypair = Keypair::generate();
//     let bob_keypair = Keypair::generate();

//     let mut alice_acc = Account::new(1000, 0);
//     let mut bob_acc = Account::new(200, 0);

//     let tx = Transaction::new(&alice_keypair, bob_keypair.public_key(), 300, 0, 2).unwrap();

//     //verify crypto signature
//     assert!(tx.verify().is_ok());

//     //Execute transaction
//     assert!(tx.execute(&mut alice_acc, &mut bob_acc).is_ok());

//     // Assert balance & nonce updates
//     assert_eq!(alice_acc.balance(), 700);
//     assert_eq!(alice_acc.nonce(), 1);
//     assert_eq!(bob_acc.balance(), 500);
// }

// #[test]
// fn test_tampered_transaction_fails_verification() {
//     let alice_keypair = Keypair::generate();
//     let bob_keypair = Keypair::generate();

//     let mut tx = Transaction::new(&alice_keypair, bob_keypair.public_key(), 100, 0, 2).unwrap();

//     tx.amount = 10000;

//     assert!(matches!(
//         tx.verify(),
//         Err(TransactionError::InvalidSignature(_))
//     ));
// }

// #[test]
// fn test_zero_amount_and_self_transfer_rejected() {
//     let alice_keypair = Keypair::generate();
//     let bob_keypair = Keypair::generate();

//     // Zero amount rejection
//     let zero_res = Transaction::new(&alice_keypair, bob_keypair.public_key(), 0, 0, 5);
//     assert_eq!(zero_res, Err(TransactionError::ZeroAmount));

//     let self_res = Transaction::new(&alice_keypair, alice_keypair.public_key(), 100, 0, 5);
//     assert_eq!(self_res, Err(TransactionError::SelfTransfer));
// }

// #[test]
// fn test_insufficient_balance_rejection() {
//     let alice_keypair = Keypair::generate();
//     let bob_keypair = Keypair::generate();

//     let mut alice_acc = Account::new(50, 0); // Only 50 balance
//     let mut bob_acc = Account::new(0, 0);

//     let tx = Transaction::new(&alice_keypair, bob_keypair.public_key(), 500, 0, 5).unwrap();

//     let res = tx.execute(&mut alice_acc, &mut bob_acc);

//     assert!(matches!(res, Err(StateError::InsufficientBalance { .. })));
// }

// #[test]
// fn test_invalid_nonce_rejection() {
//     let alice_keypair = Keypair::generate();
//     let bob_keypair = Keypair::generate();

//     let mut alice_acc = Account::new(1000, 3); // Current nonce is 3
//     let mut bob_acc = Account::new(0, 0);

//     // Transaction attempts to use nonce 0
//     let tx = Transaction::new(&alice_keypair, bob_keypair.public_key(), 100, 0, 2).unwrap();

//     let res = tx.execute(&mut alice_acc, &mut bob_acc);
//     assert!(matches!(res, Err(StateError::InvalidNonce { .. })));
// }
