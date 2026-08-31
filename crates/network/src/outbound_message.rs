use std::net::SocketAddr;

use crate::message::NetworkMessage;

//This is for who receives it and what they respond with
pub struct OutboundMessage {
    pub peer: SocketAddr,
    pub message: NetworkMessage,
}
