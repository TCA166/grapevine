use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize, ser::SerializeStruct};

/// General purpose message packet
pub struct Message {
    content: String,
    timestamp: DateTime<Utc>,
    theirs: bool,
}

impl Message {
    /// Create a new message packet, implicitly setting the timestamp, and
    /// identifying it as sent by us
    pub fn new(content: String) -> Self {
        let timestamp = Utc::now();
        Self {
            content,
            timestamp,
            theirs: false,
        }
    }

    /// Check if the message was sent by us
    pub fn is_theirs(&self) -> bool {
        self.theirs
    }

    /// Get the timestamp of the message
    pub fn timestamp(&self) -> &DateTime<Utc> {
        &self.timestamp
    }

    /// Get the content of the message
    pub fn content(&self) -> &String {
        &self.content
    }
}

impl Serialize for Message {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("Message", 2)?;
        s.serialize_field("content", &self.content)?;
        s.serialize_field("timestamp", &self.timestamp)?;
        s.end()
    }
}

impl<'de> Deserialize<'de> for Message {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct MessageHelper {
            content: String,
            timestamp: DateTime<Utc>,
        }
        let helper = MessageHelper::deserialize(deserializer)?;
        Ok(Message {
            content: helper.content,
            timestamp: helper.timestamp,
            theirs: true,
        })
    }
}
