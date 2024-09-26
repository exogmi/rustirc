use tokio::net::{TcpStream, TcpListener};
use tokio::io::{AsyncWriteExt, AsyncBufReadExt, BufReader};
use crate::server::listener::{start_server, SharedState};
use std::sync::Arc;

#[tokio::test]
async fn test_server_starts_and_accepts_connections() {
    // Start the server in a separate task
    let server_address = "127.0.0.1:8080";
    let _server_task = tokio::spawn(async move {
        if let Err(e) = start_server(server_address).await {
            eprintln!("Server error: {}", e);
        }
    });

    // Give the server a moment to start
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Try to connect to the server
    let result = TcpStream::connect(server_address).await;
    assert!(result.is_ok(), "Failed to connect to server");

    // If we've reached this point, the connection was successful
    println!("Successfully connected to server");

    // Clean up: close the connection and stop the server
    drop(result);
    // server_task.abort();
}

#[tokio::test]
async fn test_client_disconnection() {
    let server_address = "127.0.0.1:8082";
    let shared_state = Arc::new(SharedState::new());
    let server_state = shared_state.clone();

    let _server_task = tokio::spawn(async move {
        let listener = TcpListener::bind(server_address).await.unwrap();
        let (socket, addr) = listener.accept().await.unwrap();
        crate::server::listener::handle_client(socket, server_state, addr).await.unwrap();
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let mut stream = TcpStream::connect(server_address).await.unwrap();
    let (reader, mut writer) = stream.split();
    let mut reader = BufReader::new(reader).lines();

    writer.write_all(b"Hello\n").await.unwrap();
    let response = reader.next_line().await.unwrap().unwrap();
    assert_eq!(response, "Echo: Hello");

    // Simulate client disconnection
    drop(stream);

    // Wait for the server to process the disconnection
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Check if the user was removed from the shared state
    let users = shared_state.users.lock().unwrap();
    assert_eq!(users.len(), 0, "User should be removed after disconnection");

    // server_task.abort();
}

#[tokio::test]
async fn test_multiple_client_connections() {
    let server_address = "127.0.0.1:8081";
    let server_task = tokio::spawn(async move {
        if let Err(e) = start_server(server_address).await {
            eprintln!("Server error: {}", e);
        }
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let mut handles = vec![];

    for i in 0..3 {
        let handle = tokio::spawn(async move {
            let mut stream = TcpStream::connect(server_address).await.unwrap();
            let (reader, mut writer) = stream.split();
            let mut reader = BufReader::new(reader).lines();

            writer.write_all(format!("Hello from client {}!\n", i).as_bytes()).await.unwrap();
            let response = reader.next_line().await.unwrap().unwrap();
            assert_eq!(response, format!("Echo: Hello from client {}!", i));
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    server_task.abort();
}
