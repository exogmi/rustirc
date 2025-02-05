
use tokio::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use crate::models::user::User;
use crate::models::channel::Channel;
use crate::utils::generate_client_id;
use std::net::SocketAddr;
use crate::server::client::Client;
use log::LevelFilter;
use tokio::sync::broadcast;

pub struct SharedState {
    pub users: Arc<Mutex<HashMap<usize, User>>>,
    pub channels: Arc<Mutex<HashMap<String, Channel>>>,
    pub tx: broadcast::Sender<String>,
}

impl SharedState {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(100);
        SharedState {
            users: Arc::new(Mutex::new(HashMap::new())),
            channels: Arc::new(Mutex::new(HashMap::new())),
            tx,
        }
    }
}

pub async fn start_server(address: &str, _log_level: LevelFilter) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(address).await?;
    log::info!("Server listening on {}", address);

    let shared_state = Arc::new(SharedState::new());

    loop {
        let (socket, addr) = listener.accept().await?;
        let state = Arc::clone(&shared_state);

        tokio::spawn(async move {
            let mut rx = state.tx.subscribe();
            let mut client = Client::new(generate_client_id(), socket, addr.ip());

            loop {
                tokio::select! {
                    result = client.handle(Arc::clone(&state)) => {
                        if let Err(e) = result {
                            log::error!("Error handling client {}: {}", addr, e);
                        }
                        break;
                    }
                    Ok(msg) = rx.recv() => {
                        if let Err(e) = client.send(&msg).await {
                            log::error!("Error sending message to client {}: {}", addr, e);
                            break;
                        }
                    }
                }
            }
        });
    }
}

pub async fn handle_client(socket: tokio::net::TcpStream, state: Arc<SharedState>, addr: SocketAddr) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client_id = generate_client_id();
    let mut client = Client::new(client_id, socket, addr.ip());
    
    // Initialize client state
    {
        let mut users = state.users.lock().unwrap();
        users.insert(client_id, client.user.clone());
    }

    log::info!("New client connected: {}", addr);

    let mut rx = state.tx.subscribe();

    loop {
        tokio::select! {
            result = client.handle(state.clone()) => {
                if let Err(e) = result {
                    log::error!("Error handling client {}: {}", addr, e);
                }
                break;
            }
            Ok(msg) = rx.recv() => {
                if let Some((recipient_id, message)) = msg.split_once(':') {
                    if recipient_id.parse::<usize>().ok() == Some(client_id) {
                        if let Err(e) = client.send(message).await {
                            log::error!("Error sending message to client {}: {}", addr, e);
                            break;
                        }
                    }
                }
            }
        }
    }

    // Clean up client state
    {
        let mut users = state.users.lock().unwrap();
        users.remove(&client_id);
    }

    log::info!("Client disconnected: {}", addr);
    Ok(())
}
