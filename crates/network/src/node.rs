use std::{collections::HashMap, error::Error, net::SocketAddr};

use tokio::{net::TcpListener, sync::mpsc};

use crate::{NetworkEvent, Peer};

pub struct Node {
    address: SocketAddr,
    peers: HashMap<SocketAddr, Peer>,
}

impl Node {
    pub fn new(address: SocketAddr) -> Self {
        Self {
            address,
            peers: HashMap::new(),
        }
    }

    pub fn address(&self) -> &SocketAddr {
        &self.address
    }

    pub fn peers(&self) -> &HashMap<SocketAddr, Peer> {
        &self.peers
    }

    pub async fn start(&mut self) -> Result<(), Box<dyn Error>> {
        let listener = TcpListener::bind(&self.address).await?;
        println!("Node listening on {}", self.address);

        let (event_tx, mut event_rx) = mpsc::channel::<NetworkEvent>(100);

        loop {
            tokio::select! {
                result =  listener.accept()=>{
                    let (stream, peer_address) = result?;

                    println!("Incomming connection from {}", peer_address);

                    let mut peer = Peer::new(peer_address, stream);

                    let peer_tx = event_tx.clone();

                    tokio::spawn(async move {
                                  loop {
                                      match peer.receive().await {
                                          Ok(message) => {
                                              let event =
                                                  NetworkEvent::Message {
                                                      peer: *peer.address(),
                                                      message: Box::new(message),
                                                  };

                                              if peer_tx.send(event).await.is_err() {
                                                  break;
                                              }
                                          }

                                          Err(_) => {
                                              let _ = peer_tx
                                                  .send(
                                                      NetworkEvent::Disconnected {
                                                          peer: *peer.address(),
                                                      }
                                                  )
                                                  .await;

                                              break;
                                          }
                                      }
                                  }
                              });


                }

                Some(event) = event_rx.recv() =>{
                  match event {
                      NetworkEvent::Message { peer, message } => {
                          println!("Received message from {}", peer);
                          println!("Message: {:?}", message);
                      }
                      NetworkEvent::Disconnected { peer } => {
                          println!("Disconnected from {}", peer);
                      }
                  }
                }
            }
        }
    }

    pub async fn connect_to_peer(
        &self,
        address: SocketAddr,
    ) -> Result<Peer, Box<dyn Error + Send + Sync>> {
        let peer = Peer::connect(address).await?;

        println!("Connected to peer {}", peer.address());

        Ok(peer)
    }
}
