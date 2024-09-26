
use tokio::net::TcpStream;
use tokio::io::{AsyncWriteExt, BufReader, AsyncBufReadExt};
use crate::commands::parser::parse_command;
use crate::commands::handler::{handle_command, SharedState as HandlerSharedState};
use crate::models::user::User;
use std::sync::Arc;
use crate::server::listener::SharedState as ListenerSharedState;

pub struct Client {
    pub id: usize,
    pub stream: TcpStream,
    pub user: User,
}

impl Client {
    pub fn new(id: usize, stream: TcpStream, ip: std::net::IpAddr) -> Self {
        Client {
            id,
            stream,
            user: User::new(id, ip),
        }
    }

    pub async fn handle(&mut self, shared_state: Arc<ListenerSharedState>) -> Result<(), Box<dyn std::error::Error>> {
        let (reader, mut writer) = self.stream.split();
        let mut reader = BufReader::new(reader).lines();

        let handler_shared_state = HandlerSharedState {
            users: Arc::clone(&shared_state.users),
            channels: Arc::clone(&shared_state.channels),
        };

        while let Some(line) = reader.next_line().await? {
            log::trace!("Received from client {}: {}", self.id, line);

            if let Some(command) = parse_command(&line) {
                log::debug!("Parsed command from client {}: {:?}", self.id, command);

                match handle_command(command, self.id, &handler_shared_state).await {
                    Ok(responses) => {
                        for response in responses {
                            match writer.write_all(response.as_bytes()).await {
                                Ok(_) => {
                                    if let Err(e) = writer.write_all(b"\r\n").await {
                                        log::error!("Error writing newline to client {}: {}", self.id, e);
                                        return Err(Box::new(e));
                                    }
                                    if let Err(e) = writer.flush().await {
                                        log::error!("Error flushing writer for client {}: {}", self.id, e);
                                        return Err(Box::new(e));
                                    }
                                    log::trace!("Sent to client {}: {}", self.id, response);
                                }
                                Err(e) => {
                                    log::error!("Error writing to client {}: {}", self.id, e);
                                    return Err(Box::new(e));
                                }
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("Error handling command for client {}: {}", self.id, e);
                        if let Err(write_err) = writer.write_all(format!("ERROR :{}\r\n", e).as_bytes()).await {
                            log::error!("Error writing error message to client {}: {}", self.id, write_err);
                            return Err(Box::new(write_err));
                        }
                    }
                }
            } else {
                log::warn!("Unable to parse command from client {}: {}", self.id, line);
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
