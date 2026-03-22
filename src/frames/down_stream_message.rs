use crate::{CallbackMessage, EventMessage};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;

/// Base message structure for downstream messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownStreamMessage {
    #[serde(rename = "specVersion")]
    pub spec_version: Option<String>,
    pub headers: MessageHeaders,
    #[serde(rename = "type")]
    pub r#type: Type,
    pub data: Option<String>,
    #[serde(flatten)]
    pub extensions: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Type {
    #[serde(rename = "SYSTEM")]
    System,
    #[serde(rename = "EVENT")]
    Event,
    #[serde(rename = "CALLBACK")]
    Callback,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Data {
    System(super::callback_message::Data),
    Event(EventMessage),
    Callback(CallbackMessage),
}

/// Headers for all message types
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MessageHeaders {
    #[serde(rename = "appId", skip_serializing_if = "Option::is_none")]
    pub app_id: Option<String>,
    #[serde(rename = "connectionId", skip_serializing_if = "Option::is_none")]
    pub connection_id: Option<String>,
    #[serde(rename = "contentType", skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    #[serde(rename = "messageId", skip_serializing_if = "Option::is_none")]
    pub message_id: Option<String>,
    #[serde(rename = "time", skip_serializing_if = "Option::is_none")]
    pub time: Option<String>,
    #[serde(rename = "topic", skip_serializing_if = "Option::is_none")]
    pub topic: Option<MessageTopic>,
    // Event fields
    #[serde(rename = "eventBornTime", skip_serializing_if = "Option::is_none")]
    pub event_born_time: Option<i64>,
    #[serde(rename = "eventCorpId", skip_serializing_if = "Option::is_none")]
    pub event_corp_id: Option<String>,
    #[serde(rename = "eventId", skip_serializing_if = "Option::is_none")]
    pub event_id: Option<String>,
    #[serde(rename = "eventType", skip_serializing_if = "Option::is_none")]
    pub event_type: Option<String>,
    #[serde(rename = "eventUnifiedAppId", skip_serializing_if = "Option::is_none")]
    pub event_unified_app_id: Option<String>,
    /// Additional extension fields
    #[serde(flatten)]
    pub extensions: HashMap<String, serde_json::Value>,
}

impl MessageHeaders {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_message_id(mut self, message_id: String) -> Self {
        self.message_id = Some(message_id);
        self
    }

    pub fn with_content_type(mut self, content_type: &str) -> Self {
        self.content_type = Some(content_type.to_string());
        self
    }
}

/// System message topic enum
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Hash)]
#[serde(untagged)]
pub enum MessageTopic {
    #[serde(rename = "CONNECTED")]
    Connected,
    #[serde(rename = "REGISTERED")]
    Registered,
    #[serde(rename = "disconnect")]
    Disconnect,
    #[serde(rename = "KEEPALIVE")]
    KeepAlive,
    #[serde(rename = "ping")]
    Ping,
    Callback(String),
}

impl From<String> for MessageTopic {
    fn from(s: String) -> Self {
        let uppercase = s.to_uppercase();
        match uppercase.as_str() {
            "CONNECTED" => MessageTopic::Connected,
            "REGISTERED" => MessageTopic::Registered,
            "DISCONNECT" => MessageTopic::Disconnect,
            "KEEPALIVE" => MessageTopic::KeepAlive,
            "PING" => MessageTopic::Ping,
            _ => MessageTopic::Callback(s),
        }
    }
}

impl<'de> Deserialize<'de> for MessageTopic {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let str = String::deserialize(deserializer)?;
        Ok(Self::from(str))
    }
}

impl std::fmt::Display for MessageTopic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            MessageTopic::Connected => "CONNECTED",
            MessageTopic::Registered => "REGISTERED",
            MessageTopic::Disconnect => "disconnect",
            MessageTopic::KeepAlive => "KEEPALIVE",
            MessageTopic::Ping => "ping",
            MessageTopic::Callback(s) => s,
        };
        write!(f, "{str}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_message_topic_deserialize() {
        let json = include_str!("../../test_resources/client_downstream_msg.json");
        let message: DownStreamMessage = serde_json::from_str(json).unwrap();
        let CallbackMessage { data, .. } = CallbackMessage::try_from(message).unwrap();
        let data = data.unwrap();
        assert_eq!(data.msg_id.unwrap().as_str(), "msgtH7As/bwsnWfBS0olMV5tA==");
    }
}
