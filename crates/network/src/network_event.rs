use std::net::SocketAddr;

use crate::message::NetworkMessage;

#[derive(Debug)]
pub enum NetworkEvent {
    Message {
        peer: SocketAddr,
        message: Box<NetworkMessage>,
    },
    Disconnected {
        peer: SocketAddr,
    },
}
