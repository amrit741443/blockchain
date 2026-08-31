use network::{NetworkMessage, Peer};
use std::net::SocketAddr;
use tokio::{io::AsyncWriteExt, net::TcpListener};

#[tokio::test]
async fn test_peer_send_receive() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();

    let address: SocketAddr = listener.local_addr().unwrap();

    let client_task = tokio::spawn(async move {
        let mut peer = Peer::connect(address).await.unwrap();

        peer.send(&NetworkMessage::Ping).await.unwrap();
    });

    let (stream, peer_address) = listener.accept().await.unwrap();

    let mut peer = Peer::new(peer_address, stream);

    let message = peer.receive().await.unwrap();

    assert!(matches!(message, NetworkMessage::Ping));

    client_task.await.unwrap();
}

#[tokio::test]
async fn test_peer_rejects_oversized_message() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();

    let address = listener.local_addr().unwrap();

    let server = tokio::spawn(async move {
        let (stream, peer_address) = listener.accept().await.unwrap();

        let mut peer = Peer::new(peer_address, stream);

        let result = peer.receive().await;

        assert!(result.is_err());
    });

    let mut stream = tokio::net::TcpStream::connect(address).await.unwrap();

    // Larger than the allowed message size.
    let length: usize = 2 * 1024 * 1024;

    stream.write_all(&length.to_be_bytes()).await.unwrap();

    server.await.unwrap();
}
