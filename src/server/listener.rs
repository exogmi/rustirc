
use tokio::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use crate::models::user::User;
use crate::models::channel::Channel;
use crate::utils::generate_client_id;
use std::net::SocketAddr;
use crate::server::client::Client;
use log::LevelFilter;

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

pub async fn start_server(address: &str, log_level: LevelFilter) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(address).await?;
    log::info!("Server listening on {}", address);

    let shared_state = Arc::new(SharedState::new());

    loop {
        let (socket, addr) = listener.accept().await?;
        let state = Arc::clone(&shared_state);

        tokio::spawn(async move {
            if let Err(e) = handle_client(socket, state, addr, log_level).await {
                log::error!("Error handling client {}: {}", addr, e);
            }
        });
    }
}

pub async fn handle_client(socket: tokio::net::TcpStream, state: Arc<SharedState>, addr: SocketAddr, log_level: LevelFilter) -> Result<(), Box<dyn std::error::Error>> {
    let client_id = generate_client_id();
    let mut client = Client::new(client_id, socket, addr.ip());
    
    // Initialize client state
    {
        let mut users = state.users.lock().unwrap();
        users.insert(client_id, client.user.clone());
    }

    log::info!("New client connected: {}", addr);

    client.handle(state.clone(), log_level).await?;

    // Clean up client state
    {
        let mut users = state.users.lock().unwrap();
        users.remove(&client_id);
    }

    log::info!("Client disconnected: {}", addr);
    Ok(())
}
