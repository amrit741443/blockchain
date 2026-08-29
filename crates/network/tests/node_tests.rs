use blockchain_core::{blockchain::Blockchain, transaction::Transaction};

use crypto::{Address, Keypair};

use network::{NetworkMessage, Node, Peer};

use tokio::net::{TcpListener, TcpStream};

#[tokio::test]
async fn test_node_accepts_connections() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();

    let addr = listener.local_addr().unwrap();

    println!("this is {:?}", addr);

    // Simulate another node connecting.
    let client = tokio::spawn(async move {
        TcpStream::connect(addr).await.unwrap();
    });

    let (stream, peer_addr) = listener.accept().await.unwrap();

    let peer = Peer::new(peer_addr, stream);

    println!("this is {:?}", peer.address());

    assert_eq!(*peer.address(), peer_addr);

    client.await.unwrap();
}

#[tokio::test]
async fn test_node_connects_to_peer() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();

    let server_address = listener.local_addr().unwrap();

    let server = tokio::spawn(async move {
        let (stream, peer_address) = listener.accept().await.unwrap();

        let mut peer = Peer::new(peer_address, stream);

        let message = peer.receive().await.unwrap();

        assert!(matches!(message, NetworkMessage::Ping));

        peer.send(&NetworkMessage::Pong).await.unwrap();
    });

    let node = Node::new("127.0.0.1:0".parse().unwrap());

    let mut peer = node.connect_to_peer(server_address).await.unwrap();

    peer.send(&NetworkMessage::Ping).await.unwrap();

    let response = peer.receive().await.unwrap();

    assert!(matches!(response, NetworkMessage::Pong));

    server.await.unwrap();
}

#[tokio::test]
async fn test_transaction_message() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();

    let address = listener.local_addr().unwrap();

    let server = tokio::spawn(async move {
        let (stream, peer_address) = listener.accept().await.unwrap();

        let mut peer = Peer::new(peer_address, stream);

        let message = peer.receive().await.unwrap();

        match message {
            NetworkMessage::NewTransaction(tx) => {
                assert_eq!(tx.nonce(), 0);
            }

            _ => {
                panic!("Expected transaction message");
            }
        }
    });

    let client = TcpStream::connect(address).await.unwrap();

    // `address` is the remote/server address.
    let mut peer = Peer::new(address, client);

    let transaction = get_transaction();

    peer.send(&NetworkMessage::NewTransaction(Box::new(transaction)))
        .await
        .unwrap();

    server.await.unwrap();
}

fn get_transaction() -> Transaction {
    let mut blockchain = Blockchain::new(2).unwrap();

    let alice = Keypair::generate();

    let bob = Keypair::generate();

    let alice_addr = Address::from(alice.public_key());

    blockchain.state_mut().credit(alice_addr, 5000).unwrap();

    Transaction::new(&alice, bob.public_key(), 100, 0, 5).unwrap()
}
