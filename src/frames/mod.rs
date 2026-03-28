//! Frames module for DingTalk Stream SDK
//!
//! Contains all message types and structures for communication with DingTalk

use serde::{Deserialize, Serialize};
use std::ops::Deref;
use std::sync::Arc;
use crate::frames::down_message::MessageHeaders;

pub mod down_message;
pub mod up_message;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DingTalkUserId(pub String);

impl<S: Into<String>> From<S> for DingTalkUserId {
    fn from(value: S) -> Self {
        Self(value.into())
    }
}

impl Deref for DingTalkUserId {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DingTalkPrivateConversationId(pub String);

impl<S: Into<String>> From<S> for DingTalkPrivateConversationId {
    fn from(value: S) -> Self {
        Self(value.into())
    }
}
impl Deref for DingTalkPrivateConversationId {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DingTalkGroupConversationId(pub String);

impl<S: Into<String>> From<S> for DingTalkGroupConversationId {
    fn from(value: S) -> Self {
        Self(value.into())
    }
}
impl Deref for DingTalkGroupConversationId {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
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

pub type SendMessageCallbackFn =
    dyn Fn(Result<SendMessageCallbackData, anyhow::Error>) + Send + Sync + 'static;
#[derive(Clone)]
pub struct SendMessageCallback(Arc<SendMessageCallbackFn>);

impl Deref for SendMessageCallback {
    type Target = SendMessageCallbackFn;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl<F> From<F> for SendMessageCallback
where
    F: Fn(Result<SendMessageCallbackData, anyhow::Error>) + Send + Sync + 'static,
{
    fn from(value: F) -> Self {
        SendMessageCallback(Arc::new(value))
    }
}

#[derive(Clone, Default)]
pub struct OptionSendMessageCallback(Option<SendMessageCallback>);

impl<T: Into<SendMessageCallback>> From<T> for OptionSendMessageCallback {
    fn from(value: T) -> Self {
        Self(Some(value.into()))
    }
}

impl Deref for OptionSendMessageCallback {
    type Target = Option<SendMessageCallback>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct SendMessageCallbackData {
    pub http_status: u16,
    pub text: String,
}
