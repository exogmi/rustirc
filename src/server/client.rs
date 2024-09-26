
use tokio::net::TcpStream;
use tokio::io::{AsyncWriteExt, BufReader, AsyncBufReadExt};
use tokio::sync::broadcast;
use crate::commands::parser::parse_command;
use crate::commands::handler::{handle_command, SharedState as HandlerSharedState};
use crate::models::user::User;
use std::sync::Arc;
use crate::server::listener::SharedState as ListenerSharedState;
use log::LevelFilter;

pub struct Client {
    pub id: usize,
    pub stream: TcpStream,
    pub user: User,
    pub tx: broadcast::Sender<String>,
    pub rx: broadcast::Receiver<String>,
}

impl Client {
    pub fn new(id: usize, stream: TcpStream, ip: std::net::IpAddr, tx: broadcast::Sender<String>) -> Self {
        Client {
            id,
            stream,
            user: User::new(id, ip),
            tx: tx.clone(),
            rx: tx.subscribe(),
        }
    }

    pub async fn handle(&mut self, shared_state: Arc<ListenerSharedState>, log_level: LevelFilter) -> Result<(), Box<dyn std::error::Error>> {
        let (reader, mut writer) = self.stream.split();
        let mut reader = BufReader::new(reader);

        let handler_shared_state = HandlerSharedState {
            users: Arc::clone(&shared_state.users),
            channels: Arc::clone(&shared_state.channels),
        };

        loop {
            let mut line = String::new();
            tokio::select! {
                result = reader.read_line(&mut line) => {
                    match result {
                        Ok(0) => break, // EOF
                        Ok(_) => {
                            if log_level == LevelFilter::Trace {
                                log::trace!("Received from client {}: {}", self.id, line.trim());
                            }

                            if let Some(command) = parse_command(line.trim()) {
                                if log_level >= LevelFilter::Debug {
                                    log::debug!("Parsed command from client {}: {:?}", self.id, command);
                                }

                                let responses = handle_command(command, self.id, &handler_shared_state).await?;
                                for response in responses {
                                    // Send response directly to the client
                                    writer.write_all(response.as_bytes()).await?;
                                    writer.write_all(b"\r\n").await?;

                                    if log_level == LevelFilter::Trace {
                                        log::trace!("Sent to client {}: {}", self.id, response);
                                    }
                                }
                            }
                        }
                        Err(e) => return Err(Box::new(e)),
                    }
                }
                result = self.rx.recv() => {
                    match result {
                        Ok(msg) => {
                            writer.write_all(msg.as_bytes()).await?;
                            writer.write_all(b"\r\n").await?;
                        }
                        Err(e) => log::error!("Error receiving broadcast: {}", e),
                    }
                }
            }
        }

        Ok(())
    }

    pub async fn send(&mut self, message: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.stream.write_all(message.as_bytes()).await?;
        self.stream.write_all(b"\r\n").await?;
        Ok(())
    }
}
