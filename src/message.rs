use chrono::{DateTime, Utc};
use std::{fmt::Display, io::Read};

pub struct Message {
    sender: String,
    content: String,
    timestamp: DateTime<Utc>,
}

impl Message {
    pub fn read_from<R: Read>(reader: &mut R) -> Self {}
}

impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}: {}", self.timestamp, self.sender, self.content)
    }
}
