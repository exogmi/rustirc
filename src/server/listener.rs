
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use crate::models::user::User;
use crate::models::channel::Channel;
use crate::utils::generate_client_id;
use std::net::SocketAddr;

pub struct SharedState {
    pub users: Arc<Mutex<HashMap<usize, User>>>,
    pub channels: Arc<Mutex<HashMap<String, Channel>>>,
}

impl SharedState {
    pub fn new() -> Self {
        SharedState {
            users: Arc::new(Mutex::new(HashMap::new())),
            channels: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

pub async fn start_server(address: &str) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(address).await?;
    println!("Server listening on {}", address);

    let shared_state = Arc::new(SharedState::new());

    loop {
        let (socket, addr) = listener.accept().await?;
        let state = Arc::clone(&shared_state);

        tokio::spawn(async move {
            if let Err(e) = handle_client(socket, state, addr).await {
                eprintln!("Error handling client {}: {}", addr, e);
            }
        });
    }
}

async fn handle_client(mut socket: tokio::net::TcpStream, state: Arc<SharedState>, addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
    let client_id = generate_client_id();
    
    // Initialize client state
    {
        let mut users = state.users.lock().unwrap();
        users.insert(client_id, User::new(client_id, addr.ip()));
    }

    println!("New client connected: {}", addr);

    let mut buffer = [0; 1024];

    loop {
        let n = socket.read(&mut buffer).await?;
        if n == 0 {
            break; // Client disconnected
        }

        let message = String::from_utf8_lossy(&buffer[..n]);
        println!("Received from client {}: {}", addr, message);

        // Echo the message back to the client (temporary, replace with actual command handling later)
        let response = format!("Echo: {}", message);
        socket.write_all(response.as_bytes()).await?;
    }

    // Clean up client state
    {
        let mut users = state.users.lock().unwrap();
        users.remove(&client_id);
    }

    println!("Client disconnected: {}", addr);
    Ok(())
}
