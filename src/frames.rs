//! Frames module for DingTalk Stream SDK
//!
//! Contains all message types and structures for communication with DingTalk

use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use serde::de::DeserializeOwned;

/// Headers for all message types
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Headers {
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
    pub topic: Option<String>,
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

impl Headers {
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

/// Base message structure for downstream messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientDownStream {
    #[serde(rename = "specVersion")]
    pub spec_version: Option<String>,
    #[serde(rename = "type")]
    pub msg_type: String,
    pub headers: Headers,
    pub data: Option<String>,
    #[serde(flatten)]
    pub extensions: HashMap<String, serde_json::Value>,
}

impl ClientDownStream {
    /// Get the message type
    pub fn msg_type(&self) -> &str {
        &self.msg_type
    }

    /// Parse the data field as JSON
    pub fn parse_data<T: for<'de> Deserialize<'de>>(&self) -> Option<T> {
        self.data.as_ref().and_then(|d| serde_json::from_str(d).ok())
    }
}

/// Event message type
pub const MSG_TYPE_EVENT: &str = "EVENT";
/// Callback message type
pub const MSG_TYPE_CALLBACK: &str = "CALLBACK";
/// System message type
pub const MSG_TYPE_SYSTEM: &str = "SYSTEM";

/// System message topic: connection established
pub const TOPIC_CONNECTED: &str = "CONNECTED";
/// System message topic: registered
pub const TOPIC_REGISTERED: &str = "REGISTERED";
/// System message topic: disconnect
pub const TOPIC_DISCONNECT: &str = "disconnect";
/// System message topic: keep alive
pub const TOPIC_KEEPALIVE: &str = "KEEPALIVE";
/// System message topic: ping
pub const TOPIC_PING: &str = "ping";

/// Content type for JSON
pub const CONTENT_TYPE_APPLICATION_JSON: &str = "application/json";

/// Event message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMessage {
    #[serde(rename = "specVersion")]
    pub spec_version: Option<String>,
    #[serde(rename = "type")]
    pub msg_type: String,
    pub headers: Headers,
    pub data: Option<serde_json::Value>,
    #[serde(flatten)]
    pub extensions: HashMap<String, serde_json::Value>,
}

impl EventMessage {
    pub fn new() -> Self {
        Self {
            spec_version: None,
            msg_type: MSG_TYPE_EVENT.to_string(),
            headers: Headers::new(),
            data: None,
            extensions: HashMap::new(),
        }
    }

    pub fn from_stream(msg: ClientDownStream) -> Self {
        Self {
            spec_version: msg.spec_version,
            msg_type: msg.msg_type,
            headers: msg.headers,
            data: msg.data.as_ref().and_then(|d| serde_json::from_str(d).ok()),
            extensions: msg.extensions,
        }
    }
}

/// Callback message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallbackMessage {
    #[serde(rename = "specVersion")]
    pub spec_version: Option<String>,
    #[serde(rename = "type")]
    pub msg_type: String,
    pub headers: Headers,
    pub data: Option<serde_json::Value>,
    #[serde(flatten)]
    pub extensions: HashMap<String, serde_json::Value>,
}

impl CallbackMessage {
    pub fn new() -> Self {
        Self {
            spec_version: None,
            msg_type: MSG_TYPE_CALLBACK.to_string(),
            headers: Headers::new(),
            data: None,
            extensions: HashMap::new(),
        }
    }

    pub fn from_stream(msg: ClientDownStream) -> Self {
        Self {
            spec_version: msg.spec_version,
            msg_type: msg.msg_type,
            headers: msg.headers,
            data: msg.data.as_ref().and_then(|d| serde_json::from_str(d).ok()),
            extensions: msg.extensions,
        }
    }
}

/// System message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMessage {
    #[serde(rename = "specVersion")]
    pub spec_version: Option<String>,
    #[serde(rename = "type")]
    pub msg_type: String,
    pub headers: Headers,
    pub data: Option<serde_json::Value>,
    #[serde(flatten)]
    pub extensions: HashMap<String, serde_json::Value>,
}

impl SystemMessage {
    pub fn new() -> Self {
        Self {
            spec_version: None,
            msg_type: MSG_TYPE_SYSTEM.to_string(),
            headers: Headers::new(),
            data: None,
            extensions: HashMap::new(),
        }
    }

    pub fn from_stream(msg: ClientDownStream) -> Self {
        Self {
            spec_version: msg.spec_version,
            msg_type: msg.msg_type,
            headers: msg.headers,
            data: msg.data.as_ref().and_then(|d| serde_json::from_str(d).ok()),
            extensions: msg.extensions,
        }
    }
}

/// ACK message status codes
pub mod ack_status {
    pub const OK: i32 = 200;
    pub const BAD_REQUEST: i32 = 400;
    pub const NOT_IMPLEMENTED: i32 = 404;
    pub const SYSTEM_EXCEPTION: i32 = 500;
}

/// ACK status constants for convenience
pub const OK: i32 = ack_status::OK;
pub const BAD_REQUEST: i32 = ack_status::BAD_REQUEST;
pub const NOT_IMPLEMENTED: i32 = ack_status::NOT_IMPLEMENTED;
pub const SYSTEM_EXCEPTION: i32 = ack_status::SYSTEM_EXCEPTION;

/// ACK message for responding to messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AckMessage {
    pub code: i32,
    pub headers: Headers,
    #[serde(rename = "message")]
    pub message: String,
    pub data: Option<String>,
}

impl AckMessage {
    pub fn ok(message: &str) -> Self {
        Self {
            code: OK,
            headers: Headers::new(),
            message: message.to_string(),
            data: None,
        }
    }

    pub fn error(code: i32, message: &str) -> Self {
        Self {
            code,
            headers: Headers::new(),
            message: message.to_string(),
            data: None,
        }
    }

    pub fn with_message_id(mut self, message_id: String) -> Self {
        self.headers.message_id = Some(message_id);
        self
    }

    pub fn with_content_type(mut self, content_type: &str) -> Self {
        self.headers.content_type = Some(content_type.to_string());
        self
    }

    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = Some(serde_json::to_string(&data).unwrap_or_default());
        self
    }

    pub fn response_data(data: serde_json::Value) -> Self {
        let response = serde_json::json!({ "response": data });
        Self {
            code: OK,
            headers: Headers::new().with_content_type("application/json"),
            message: "OK".to_string(),
            data: Some(serde_json::to_string(&response).unwrap_or_default()),
        }
    }
}

/// Robot message content - text
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextContent {
    pub content: String,
}

/// Robot message content - image
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageContent {
    #[serde(rename = "downloadCode")]
    pub download_code: String,
}

/// Robot message content - rich text
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RichTextContent {
    #[serde(rename = "richText")]
    pub rich_text: Vec<serde_json::Value>,
}

/// At user information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtUser {
    #[serde(rename = "dingtalkId")]
    pub dingtalk_id: Option<String>,
    #[serde(rename = "staffId")]
    pub staff_id: Option<String>,
}

/// Robot message (chatbot message)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RobotMessage {
    #[serde(rename = "conversationId")]
    pub conversation_id: Option<String>,
    #[serde(rename = "chatbotCorpId")]
    pub chatbot_corp_id: Option<String>,
    #[serde(rename = "chatbotUserId")]
    pub chatbot_user_id: Option<String>,
    #[serde(rename = "msgId")]
    pub msg_id: Option<String>,
    #[serde(rename = "senderNick")]
    pub sender_nick: Option<String>,
    #[serde(rename = "isAdmin")]
    pub is_admin: Option<bool>,
    #[serde(rename = "senderStaffId")]
    pub sender_staff_id: Option<String>,
    #[serde(rename = "sessionWebhookExpiredTime")]
    pub session_webhook_expired_time: Option<i64>,
    #[serde(rename = "createAt")]
    pub create_at: Option<i64>,
    #[serde(rename = "senderCorpId")]
    pub sender_corp_id: Option<String>,
    #[serde(rename = "conversationType")]
    pub conversation_type: Option<String>,
    #[serde(rename = "senderId")]
    pub sender_id: Option<String>,
    #[serde(rename = "sessionWebhook")]
    pub session_webhook: Option<String>,
    #[serde(rename = "robotCode")]
    pub robot_code: Option<String>,
    #[serde(rename = "msgtype")]
    pub msgtype: Option<String>,
    #[serde(rename = "text")]
    pub text: Option<TextContent>,
    #[serde(rename = "content")]
    pub content: Option<serde_json::Value>,
    #[serde(rename = "atUsers")]
    pub at_users: Option<Vec<AtUser>>,
    #[serde(rename = "isInAtList")]
    pub is_in_at_list: Option<bool>,
    #[serde(rename = "conversationTitle")]
    pub conversation_title: Option<String>,
}

impl RobotMessage {
    /// Get text content from the message
    pub fn get_text(&self) -> Option<String> {
        if let Some(text) = &self.text {
            return Some(text.content.clone());
        }
        None
    }

    /// Get text list from the message (supports rich text)
    pub fn get_text_list(&self) -> Vec<String> {
        let mut result = Vec::new();

        if let Some(text) = &self.text {
            result.push(text.content.clone());
        } else if let Some(content) = &self.content {
            if let Some(rich_text) = content.get("richText").and_then(|v| v.as_array()) {
                for item in rich_text {
                    if let Some(text) = item.get("text").and_then(|v| v.as_str()) {
                        result.push(text.to_string());
                    }
                }
            }
        }

        result
    }

    /// Get image download codes from the message
    pub fn get_image_list(&self) -> Vec<String> {
        let mut result = Vec::new();

        if let Some(content) = &self.content {
            if let Some(download_code) = content.get("downloadCode").and_then(|v| v.as_str()) {
                result.push(download_code.to_string());
            } else if let Some(rich_text) = content.get("richText").and_then(|v| v.as_array()) {
                for item in rich_text {
                    if let Some(code) = item.get("downloadCode").and_then(|v| v.as_str()) {
                        result.push(code.to_string());
                    }
                }
            }
        }

        result
    }
}

/// Graph API message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMessage {
    #[serde(rename = "requestLine")]
    pub request_line: Option<GraphRequestLine>,
    pub headers: Option<serde_json::Value>,
    pub body: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphRequestLine {
    pub method: Option<String>,
    pub uri: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphResponse {
    #[serde(rename = "statusLine")]
    pub status_line: Option<GraphStatusLine>,
    pub headers: Option<serde_json::Value>,
    pub body: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphStatusLine {
    pub code: Option<i32>,
    #[serde(rename = "reasonPhrase")]
    pub reason_phrase: Option<String>,
}