
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
            if let Some(command) = parse_command(&line) {
                let responses = handle_command(command, self.id, &handler_shared_state).await?;
                for response in responses {
                    writer.write_all(response.as_bytes()).await?;
                    writer.write_all(b"\r\n").await?;
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
