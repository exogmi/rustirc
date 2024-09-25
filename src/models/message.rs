
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub enum Recipient {
    User(usize),
    Channel(String),
}

#[derive(Debug)]
pub struct Message {
    pub sender_id: usize,
    pub recipient: Recipient,
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

impl Message {
    pub fn new(sender_id: usize, recipient: Recipient, content: String) -> Self {
        Message {
            sender_id,
            recipient,
            content,
            timestamp: Utc::now(),
        }
    }
}
