//! Kafka message parser
//!
//! Parses Kafka messages and extracts topic, key, and partition information.

/// Kafka message representation
#[derive(Debug, Clone)]
pub struct KafkaMessage {
    pub topic: String,
    pub partition: i32,
    pub offset: i64,
    pub key: Option<Vec<u8>>,
    pub value: Option<Vec<u8>>,
}

/// Kafka message parser
pub struct KafkaParser;

impl KafkaParser {
    /// Parse a Kafka message from bytes
    ///
    /// # Examples
    ///
    /// ```
    /// use chaos_testing::parsers::kafka::KafkaParser;
    ///
    /// let message = KafkaParser::parse_message("users", 0, 100, None, Some(b"data".to_vec()));
    /// assert_eq!(message.topic, "users");
    /// ```
    pub fn parse_message(
        topic: &str,
        partition: i32,
        offset: i64,
        key: Option<Vec<u8>>,
        value: Option<Vec<u8>>,
    ) -> KafkaMessage {
        KafkaMessage {
            topic: topic.to_string(),
            partition,
            offset,
            key,
            value,
        }
    }

    /// Extract topic from Kafka metadata
    pub fn extract_topic(metadata: &str) -> Option<String> {
        metadata.split(':').nth(1).map(|s| s.to_string())
    }

    /// Classify message by topic pattern
    pub fn classify_by_topic(topic: &str) -> MessageType {
        if topic.contains("event") || topic.contains("events") {
            MessageType::Event
        } else if topic.contains("command") || topic.contains("commands") {
            MessageType::Command
        } else if topic.contains("query") || topic.contains("queries") {
            MessageType::Query
        } else if topic.ends_with("-dlq") || topic.contains("dead-letter") {
            MessageType::DeadLetter
        } else {
            MessageType::Data
        }
    }
}

/// Kafka message classification types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    Event,
    Command,
    Query,
    Data,
    DeadLetter,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_message() {
        let msg = KafkaParser::parse_message("test-topic", 0, 100, None, Some(b"data".to_vec()));
        assert_eq!(msg.topic, "test-topic");
        assert_eq!(msg.partition, 0);
        assert_eq!(msg.offset, 100);
    }

    #[test]
    fn test_extract_topic() {
        assert_eq!(
            KafkaParser::extract_topic("topic:users:metadata"),
            Some("users".to_string())
        );
    }

    #[test]
    fn test_classify_by_topic() {
        assert_eq!(
            KafkaParser::classify_by_topic("user-events"),
            MessageType::Event
        );
        assert_eq!(
            KafkaParser::classify_by_topic("create-order-command"),
            MessageType::Command
        );
        assert_eq!(
            KafkaParser::classify_by_topic("user-query"),
            MessageType::Query
        );
        assert_eq!(
            KafkaParser::classify_by_topic("users-dlq"),
            MessageType::DeadLetter
        );
        assert_eq!(KafkaParser::classify_by_topic("users"), MessageType::Data);
    }
}
