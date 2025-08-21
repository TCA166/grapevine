use chrono::{DateTime, Utc};
use std::fmt::Display;

pub struct Message {
    sender: String,
    content: String,
    timestamp: DateTime<Utc>,
}

impl Message {
    pub fn new(sender: String, content: String) -> Self {
        let timestamp = Utc::now();
        Self {
            sender,
            content,
            timestamp,
        }
    }
}

impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}: {}", self.timestamp, self.sender, self.content)
    }
}
