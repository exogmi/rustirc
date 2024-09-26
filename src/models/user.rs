
use std::collections::HashSet;
use std::net::IpAddr;

#[derive(Debug, Clone, PartialEq)]
pub enum UserStatus {
    Online,
    Away(Option<String>),
}

#[derive(Debug, Clone)]
pub struct User {
    pub id: usize,
    pub nickname: Option<String>,
    pub username: Option<String>,
    pub realname: Option<String>,
    pub host: IpAddr,
    pub channels: HashSet<String>,
    pub status: UserStatus,
}

impl User {
    pub fn get_host(&self) -> &IpAddr {
        &self.host
    }
}

impl User {
    pub fn new(id: usize, host: IpAddr) -> Self {
        User {
            id,
            nickname: None,
            username: None,
            realname: None,
            host,
            channels: HashSet::new(),
            status: UserStatus::Online,
        }
    }

    pub fn set_nickname(&mut self, nickname: String) -> Result<(), &'static str> {
        // Basic nickname validation
        if nickname.is_empty() || nickname.len() > 20 || !nickname.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            return Err("Invalid nickname");
        }
        self.nickname = Some(nickname);
        Ok(())
    }

    pub fn join_channel(&mut self, channel: String) {
        self.channels.insert(channel);
    }

    pub fn leave_channel(&mut self, channel: &str) {
        self.channels.remove(channel);
    }

    pub fn set_away(&mut self, message: Option<String>) {
        self.status = UserStatus::Away(message);
    }

    pub fn set_online(&mut self) {
        self.status = UserStatus::Online;
    }
}
