use std::{collections::HashMap, error::Error, net::SocketAddr};

use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc,
};

use crate::{NetworkEvent, NetworkMessage, OutboundMessage, Peer};

pub struct Node {
    address: SocketAddr,

    // Peer -> Node
    event_tx: mpsc::Sender<NetworkEvent>,
    event_rx: mpsc::Receiver<NetworkEvent>,

    // Node -> Peer
    outbound_tx: mpsc::Sender<OutboundMessage>,
    outbound_rx: mpsc::Receiver<OutboundMessage>,

    // Peer address -> command channel owned by that peer task
    peer_channels: HashMap<SocketAddr, mpsc::Sender<NetworkMessage>>,
}

impl Node {
    pub fn new(
        address: SocketAddr,
        event_tx: mpsc::Sender<NetworkEvent>,
        event_rx: mpsc::Receiver<NetworkEvent>,
        outbound_tx: mpsc::Sender<OutboundMessage>,
        outbound_rx: mpsc::Receiver<OutboundMessage>,
    ) -> Self {
        Self {
            address,
            event_tx,
            event_rx,
            outbound_tx,
            outbound_rx,
            peer_channels: HashMap::new(),
        }
    }

    pub fn address(&self) -> &SocketAddr {
        &self.address
    }

    pub fn event_tx(&self) -> &mpsc::Sender<NetworkEvent> {
        &self.event_tx
    }

    pub fn event_rx(&mut self) -> &mut mpsc::Receiver<NetworkEvent> {
        &mut self.event_rx
    }

    pub fn outbound_tx(&self) -> &mpsc::Sender<OutboundMessage> {
        &self.outbound_tx
    }

    pub async fn start(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let listener = TcpListener::bind(self.address).await?;

        loop {
            tokio::select! {
                // ---------------------------------------------------------
                // 1. New inbound connection
                // ---------------------------------------------------------
                result = listener.accept() => {
                    let (stream, peer_address) = result?;

                    self.register_peer(stream, peer_address).await;
                }

                // ---------------------------------------------------------
                // 2. Event from a peer
                // ---------------------------------------------------------
                Some(event) = self.event_rx.recv() => {
                    self.handle_event(event).await?;
                }

                // ---------------------------------------------------------
                // 3. Outbound message from Node
                // ---------------------------------------------------------
                Some(outbound) = self.outbound_rx.recv() => {
                    if let Some(peer_tx) =
                        self.peer_channels.get(&outbound.peer).cloned()
                    {
                        if peer_tx.send(outbound.message).await.is_err() {
                            self.peer_channels.remove(&outbound.peer);
                        }
                    } else {
                        eprintln!(
                            "Cannot send message: peer {} is not connected",
                            outbound.peer
                        );
                    }
                }

                else => {
                    break;
                }
            }
        }

        Ok(())
    }

    async fn register_peer(&mut self, stream: TcpStream, peer_address: SocketAddr) {
        if self.peer_channels.contains_key(&peer_address) {
            eprintln!("Peer already connected: {peer_address}");
            return;
        }

        let (peer_tx, mut peer_rx) = mpsc::channel::<NetworkMessage>(100);

        self.peer_channels.insert(peer_address, peer_tx);

        let event_tx = self.event_tx.clone();

        tokio::spawn(async move {
            let mut peer = Peer::new(peer_address, stream);

            loop {
                tokio::select! {
                    // TCP -> Peer -> Node
                    result = peer.receive() => {
                        match result {
                            Ok(message) => {
                                let event = NetworkEvent::Message {
                                    peer: peer_address,
                                    message: Box::new(message),
                                };

                                if event_tx.send(event).await.is_err() {
                                    break;
                                }
                            }

                            Err(error) => {
                                eprintln!(
                                    "Peer {} disconnected: {}",
                                    peer_address,
                                    error
                                );

                                let _ = event_tx
                                    .send(NetworkEvent::Disconnected {
                                        peer: peer_address,
                                    })
                                    .await;

                                break;
                            }
                        }
                    }

                    // Node -> Peer -> TCP
                    Some(message) = peer_rx.recv() => {
                        if let Err(error) = peer.send(&message).await {
                            eprintln!(
                                "Failed sending to {}: {}",
                                peer_address,
                                error
                            );

                            let _ = event_tx
                                .send(NetworkEvent::Disconnected {
                                    peer: peer_address,
                                })
                                .await;

                            break;
                        }
                    }

                    else => {
                        break;
                    }
                }
            }
        });
    }

    pub async fn connect_to_peer(
        &mut self,
        address: SocketAddr,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let stream = TcpStream::connect(address).await?;

        self.register_peer(stream, address).await;

        println!("Connected to peer {address}");

        Ok(())
    }

    pub async fn handle_event(
        &mut self,
        event: NetworkEvent,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        match event {
            NetworkEvent::Message { peer, message } => {
                match *message {
                    NetworkMessage::Ping => {
                        println!("Received Ping from {peer}");

                        self.outbound_tx
                            .send(OutboundMessage {
                                peer,
                                message: NetworkMessage::Pong,
                            })
                            .await?;
                    }

                    NetworkMessage::Pong => {
                        println!("Received Pong from {peer}");
                    }

                    NetworkMessage::NewTransaction(tx) => {
                        println!("Received transaction from {peer}: {:?}", tx);

                        // NEXT:
                        // validate transaction
                        // add transaction to mempool
                        // broadcast to other peers
                    }
                }
            }

            NetworkEvent::Disconnected { peer } => {
                println!("Peer disconnected: {peer}");

                self.peer_channels.remove(&peer);
            }
        }

        Ok(())
    }
    pub async fn send_to_peer(
        &self,
        peer: SocketAddr,
        message: NetworkMessage,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.outbound_tx
            .send(OutboundMessage { peer, message })
            .await?;

        Ok(())
    }
}
