use network::{NetworkMessage, Peer};
use tokio::net::TcpListener;

#[tokio::test]
async fn test_peer_send_receive() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();

    let address = listener.local_addr().unwrap();

    let server = tokio::spawn(async move {
        let (stream, peer_address) = listener.accept().await.unwrap();

        let mut peer = Peer::new(peer_address, stream);

        let message = peer.receive().await.unwrap();

        assert!(matches!(message, NetworkMessage::Ping));

        peer.send(&NetworkMessage::Pong).await.unwrap();
    });

    let mut client = Peer::connect(address).await.unwrap();

    client.send(&NetworkMessage::Ping).await.unwrap();

    let response = client.receive().await.unwrap();

    assert!(matches!(response, NetworkMessage::Pong));

    server.await.unwrap();
}
