
use tokio::net::TcpStream;
use crate::server::client::Client;
use crate::server::listener::SharedState;
use std::sync::Arc;
use tokio::io::{AsyncWriteExt, AsyncReadExt};

#[tokio::test]
#[ignore]
async fn test_client_handling() {
    // Start a mock server
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let shared_state = Arc::new(SharedState::new());
    let server_state = shared_state.clone();

    // Spawn the server task
    tokio::spawn(async move {
        let (socket, _) = listener.accept().await.unwrap();
        let mut client = Client::new(1, socket, addr.ip());
        client.handle(server_state, log::LevelFilter::Info).await.unwrap();
    });

    // Connect a mock client
    let mut stream = TcpStream::connect(addr).await.unwrap();

    // Send a NICK command
    stream.write_all(b"NICK testuser\r\n").await.unwrap();

    // Read the response
    let mut buffer = [0; 1024];
    let n = stream.read(&mut buffer).await.unwrap();
    let response = String::from_utf8_lossy(&buffer[..n]);

    assert!(response.contains("NICK :testuser"), "Unexpected response: {}", response);

    // Send a USER command
    stream.write_all(b"USER testuser 0 * :Test User\r\n").await.unwrap();

    // Read the response
    let n = stream.read(&mut buffer).await.unwrap();
    let response = String::from_utf8_lossy(&buffer[..n]);

    assert!(response.contains("Welcome to the IRC server!"), "Unexpected response: {}", response);

    // Close the connection
    drop(stream);

    // Wait for the server to process the disconnection
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Check if the user was removed from the shared state
    let users = shared_state.users.lock().unwrap();
    assert_eq!(users.len(), 0, "User should be removed after disconnection");
}
