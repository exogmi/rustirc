
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, AsyncBufReadExt};
use crate::commands::parser::parse_command;
use crate::commands::handler::handle_command;
use crate::models::user::User;
use std::sync::Arc;
use crate::server::listener::SharedState;

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

    pub async fn handle(&mut self, shared_state: Arc<SharedState>) -> Result<(), Box<dyn std::error::Error>> {
        let (reader, mut writer) = self.stream.split();
        let mut reader = BufReader::new(reader).lines();

        while let Some(line) = reader.next_line().await? {
            if let Some(command) = parse_command(&line) {
                let responses = handle_command(command, self.id, &shared_state).await?;
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
