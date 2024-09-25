
use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Channel {
    pub name: String,
    pub members: HashSet<usize>,
    pub topic: Option<String>,
    pub key: Option<String>,
    pub state_path: Option<PathBuf>,
}

impl Channel {
    pub fn new(name: String) -> Self {
        Channel {
            name,
            members: HashSet::new(),
            topic: None,
            key: None,
            state_path: None,
        }
    }

    pub fn add_member(&mut self, user_id: usize) {
        self.members.insert(user_id);
    }

    pub fn remove_member(&mut self, user_id: &usize) {
        self.members.remove(user_id);
    }

    pub fn set_topic(&mut self, topic: String) {
        self.topic = Some(topic);
    }

    pub fn set_key(&mut self, key: Option<String>) {
        self.key = key;
    }

    pub fn read_state(&self) -> Result<(), std::io::Error> {
        // Implement state reading logic here
        // This is a placeholder implementation
        Ok(())
    }

    pub fn write_state(&self) -> Result<(), std::io::Error> {
        // Implement state writing logic here
        // This is a placeholder implementation
        Ok(())
    }
}
