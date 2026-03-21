//! Frames module for DingTalk Stream SDK
//!
//! Contains all message types and structures for communication with DingTalk

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

mod callback_message;
pub use callback_message::{
    CallbackMessage, Data as CallbackMessageData, File as CallbackMessagePayloadFile,
    Payload as CallbackMessagePayload, Picture as CallbackMessagePayloadPicture,
    RichText as CallbackMessagePayloadRichText, Text as CallbackMessagePayloadText,
};

mod down_stream_message;
pub use down_stream_message::{
    Data as DownStreamMessageData, DownStreamMessage, MessageHeaders, MessageTopic,
};

/// Event message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMessage {
    #[serde(rename = "specVersion")]
    pub spec_version: Option<String>,
    #[serde(rename = "type")]
    pub headers: MessageHeaders,
    pub data: Option<serde_json::Value>,
    #[serde(flatten)]
    pub extensions: HashMap<String, serde_json::Value>,
}

/// System message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMessage {
    #[serde(rename = "specVersion")]
    pub spec_version: Option<String>,
    #[serde(rename = "type")]
    pub headers: MessageHeaders,
    pub data: Option<serde_json::Value>,
    #[serde(flatten)]
    pub extensions: HashMap<String, serde_json::Value>,
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
    pub headers: MessageHeaders,
    #[serde(rename = "message")]
    pub message: String,
    pub data: Option<String>,
}

impl AckMessage {
    pub fn ok(message: &str) -> Self {
        Self {
            code: OK,
            headers: MessageHeaders::new(),
            message: message.to_string(),
            data: None,
        }
    }

    pub fn error(code: i32, message: &str) -> Self {
        Self {
            code,
            headers: MessageHeaders::new(),
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
            headers: MessageHeaders::new().with_content_type("application/json"),
            message: "OK".to_string(),
            data: Some(serde_json::to_string(&response).unwrap_or_default()),
        }
    }
}
