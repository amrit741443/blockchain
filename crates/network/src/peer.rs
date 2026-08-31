use std::{error::Error, net::SocketAddr};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::NetworkMessage;

#[derive(Debug)]
pub struct Peer {
    address: SocketAddr,
    stream: TcpStream,
}

const MAX_MESSAGE_SIZE: usize = 1024 * 1024;

impl Peer {
    pub fn new(address: SocketAddr, stream: TcpStream) -> Self {
        Self { address, stream }
    }

    pub fn address(&self) -> &SocketAddr {
        &self.address
    }

    pub async fn connect(address: SocketAddr) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let stream = TcpStream::connect(address).await?;

        Ok(Self { address, stream })
    }

    pub async fn send(
        &mut self,
        message: &NetworkMessage,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let payload = bincode::serialize(message)?;

        let length = payload.len() as u32;

        self.stream.write_all(&length.to_be_bytes()).await?;

        self.stream.write_all(&payload).await?;

        Ok(())
    }

    pub async fn receive(&mut self) -> Result<NetworkMessage, Box<dyn Error + Send + Sync>> {
        let mut length_bytes = [0u8; 4];

        self.stream.read_exact(&mut length_bytes).await?;

        let length = u32::from_be_bytes(length_bytes) as usize;

        if length > MAX_MESSAGE_SIZE {
            return Err("Network message too large".into());
        }

        let mut payload = vec![0u8; length];

        self.stream.read_exact(&mut payload).await?;

        let message = bincode::deserialize(&payload)?;

        Ok(message)
    }
}
