use std::{net::SocketAddr, time::Duration};

use tokio::{net::TcpListener, sync::mpsc, time::timeout};

use network::{NetworkEvent, NetworkMessage, Node, OutboundMessage};

async fn free_port() -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("failed to bind temorary listener");

    let address = listener.local_addr().expect("failed to get local adress");

    drop(listener);

    address
}

fn create_node(
    address: SocketAddr,
) -> (
    Node,
    mpsc::Sender<NetworkEvent>,
    mpsc::Sender<OutboundMessage>,
) {
    let (event_tx, event_rx) = mpsc::channel(100);
    let (outbound_tx, outbound_rx) = mpsc::channel(100);

    let node = Node::new(
        address,
        event_tx.clone(),
        event_rx,
        outbound_tx.clone(),
        outbound_rx,
    );

    (node, event_tx, outbound_tx)
}

#[tokio::test]
async fn test_node_accepts_connection() {
    let node_address = free_port().await;

    let (mut node, _event_tx, _outbound_tx) = create_node(node_address);

    let node_task = tokio::spawn(async move { node.start().await });

    //Give the listner time to start
    tokio::time::sleep(Duration::from_millis(50)).await;

    let _stream = tokio::net::TcpStream::connect(node_address)
        .await
        .expect("failed to connect to node");

    // If the TCP connection succeeds, the Node accepted/listened
    // on the expected address.
    println!("ip address: {}", node_address);
    assert!(node_address.ip().is_loopback());

    node_task.abort();
}

#[tokio::test]
async fn test_node_receives_ping_and_responds_pong() {
    let node_address = free_port().await;

    let (event_tx, event_rx) = mpsc::channel(100);
    let (outbound_tx, outbound_rx) = mpsc::channel(100);

    let mut node = Node::new(node_address, event_tx, event_rx, outbound_tx, outbound_rx);

    let node_task = tokio::spawn(async move { node.start().await });

    tokio::time::sleep(Duration::from_millis(50)).await;

    let stream = tokio::net::TcpStream::connect(node_address)
        .await
        .expect("failed to connect");

    let peer_address = stream.local_addr().expect("failed to get local address");

    let mut test_peer = network::Peer::new(peer_address, stream);

    // Test Peer → Node
    test_peer
        .send(&NetworkMessage::Ping)
        .await
        .expect("failed to send ping");

    // Node → Test Peer
    let response = timeout(Duration::from_secs(2), test_peer.receive())
        .await
        .expect("timed out waiting for pong")
        .expect("failed to receive pong");

    assert!(matches!(response, NetworkMessage::Pong));

    node_task.abort();
}
