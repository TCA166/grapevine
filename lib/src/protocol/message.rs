use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize, ser::SerializeStruct};

/// General purpose message packet
pub struct Message {
    content: String,
    timestamp: DateTime<Utc>,
    ours: bool,
}

impl Message {
    /// Create a new message packet, implicitly setting the timestamp, and
    /// identifying it as sent by us
    pub fn new(content: String) -> Self {
        let timestamp = Utc::now();
        Self {
            content,
            timestamp,
            ours: true,
        }
    }

    /// Check if the message was sent by us
    pub fn is_ours(&self) -> bool {
        self.ours
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
            ours: false,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_message_new_sets_fields() {
        let msg = Message::new("hello".to_string());
        assert_eq!(msg.content(), "hello");
        assert!(msg.is_ours());
        // Timestamp should be close to now
        let now = Utc::now();
        assert!((now - *msg.timestamp()).num_seconds().abs() < 5);
    }

    #[test]
    fn test_message_serialize_deserialize() {
        let msg = Message::new("test content".to_string());
        let serialized = serde_json::to_string(&msg).unwrap();
        let deserialized: Message = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.content(), "test content");
        assert!(
            !deserialized.is_ours(),
            "Deserialized message should not be ours"
        );
        assert_eq!(deserialized.timestamp(), msg.timestamp());
    }

    #[test]
    fn test_message_deserialize_sets_ours_false() {
        let msg = Message::new("abc".to_string());
        let serialized = serde_json::to_string(&msg).unwrap();
        let mut deserialized: Message = serde_json::from_str(&serialized).unwrap();
        assert!(!deserialized.is_ours());
        // Changing the content should not affect the 'ours' field
        deserialized.ours = true;
        assert!(deserialized.is_ours());
    }
}
