use blockchain_core::transaction::Transaction;
use serde::{Deserialize, Serialize};

/*
* NetworkMessage
    ↓
WHAT was sent over TCP?

NetworkEvent
    ↓
WHAT happened to our Node?
*/

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkMessage {
    Ping,
    Pong,
    NewTransaction(Box<Transaction>),
}
