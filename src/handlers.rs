//! Handlers module for DingTalk Stream SDK
//!
//! Provides trait-based handlers for different message types

use crate::frames::{CallbackMessage, CallbackWebhookMessage, EventMessage, SystemMessage};
use crate::MessageTopic;
use async_trait::async_trait;
use std::fmt::{Display, Formatter};
use tokio::sync::mpsc::Sender;

/// Callback handler trait for handling callback messages
#[async_trait]
pub trait CallbackHandler: Send + Sync {
    /// Process a callback message
    async fn process(&self, message: &CallbackMessage, cb_webhook_msg_sender: Option<Sender<CallbackWebhookMessage>>) -> Result<Resp, Error>;

    /// Pre-start hook
    fn pre_start(&self) {}

    /// Get the topic this handler handles
    fn topic(&self) -> &MessageTopic;
}

/// Event handler trait for handling event messages
#[async_trait]
pub trait EventHandler: Send + Sync {
    /// Process an event message
    async fn process(&self, message: &EventMessage) -> Result<Resp, Error>;

    /// Pre-start hook
    fn pre_start(&self) {}
}

/// System handler trait for handling system messages
#[async_trait]
pub trait SystemHandler: Send + Sync {
    /// Process a system message
    async fn process(&self, message: &SystemMessage) -> Result<Resp, Error>;

    /// Pre-start hook
    fn pre_start(&self) {}
}

#[derive(Debug, Clone)]
pub enum Resp {
    Text(String),
    Json(serde_json::Value),
}

impl Display for Resp {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Resp::Text(text) => write!(f, "Text: {}", text),
            Resp::Json(json) => write!(f, "JSON: {}", json),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Error {
    pub msg: String,
    pub code: ErrorCode,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("Error: {} (Code: {})", self.msg, self.code))
    }
}

impl std::error::Error for Error {}

#[derive(Debug, Clone, Copy)]
#[repr(i32)]
pub enum ErrorCode {
    BadRequest = 400i32,
    Unauthorized = 401,
    Forbidden = 403,
    NotFound = 404,
    MethodNotAllowed = 405,
    TooManyRequests = 429,
    InternalServerError = 500,
    BadGateway = 502,
    ServiceUnavailable = 503,
    GatewayTimeout = 504,
}

impl Display for ErrorCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{}", *self as i32))
    }
}

/// Default no-op callback handler
pub struct DefaultCallbackHandler {
    pub topic: MessageTopic,
}

impl DefaultCallbackHandler {
    pub fn new(topic: &str) -> Self {
        Self {
            topic: MessageTopic::Callback(topic.to_string()),
        }
    }
}
