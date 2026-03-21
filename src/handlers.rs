//! Handlers module for DingTalk Stream SDK
//!
//! Provides trait-based handlers for different message types

use crate::frames::{CallbackMessage, EventMessage, SystemMessage, NOT_IMPLEMENTED, OK};
use async_trait::async_trait;

/// Callback handler trait for handling callback messages
#[async_trait]
pub trait CallbackHandler: Send + Sync {
    /// Process a callback message
    async fn process(&self, message: &CallbackMessage) -> (i32, String);

    /// Pre-start hook
    fn pre_start(&self) {}

    /// Get the topic this handler handles
    fn topic(&self) -> &str;
}

/// Event handler trait for handling event messages
#[async_trait]
pub trait EventHandler: Send + Sync {
    /// Process an event message
    async fn process(&self, message: &EventMessage) -> (i32, String);

    /// Pre-start hook
    fn pre_start(&self) {}
}

/// System handler trait for handling system messages
#[async_trait]
pub trait SystemHandler: Send + Sync {
    /// Process a system message
    async fn process(&self, message: &SystemMessage) -> (i32, String);

    /// Pre-start hook
    fn pre_start(&self) {}
}

/// Default no-op callback handler
pub struct DefaultCallbackHandler {
    topic: String,
}

impl DefaultCallbackHandler {
    pub fn new(topic: &str) -> Self {
        Self {
            topic: topic.to_string(),
        }
    }
}

#[async_trait]
impl CallbackHandler for DefaultCallbackHandler {
    async fn process(&self, _message: &CallbackMessage) -> (i32, String) {
        (NOT_IMPLEMENTED, "not implement".to_string())
    }

    fn topic(&self) -> &str {
        &self.topic
    }
}

/// Default no-op event handler
pub struct DefaultEventHandler;

#[async_trait]
impl EventHandler for DefaultEventHandler {
    async fn process(&self, _message: &EventMessage) -> (i32, String) {
        (NOT_IMPLEMENTED, "not implement".to_string())
    }
}

/// Default system handler
pub struct DefaultSystemHandler;

#[async_trait]
impl SystemHandler for DefaultSystemHandler {
    async fn process(&self, _message: &SystemMessage) -> (i32, String) {
        (OK, "OK".to_string())
    }
}

/// Robot handler for handling robot messages
pub struct RobotHandler;

impl RobotHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl CallbackHandler for RobotHandler {
    async fn process(&self, _message: &CallbackMessage) -> (i32, String) {
        (NOT_IMPLEMENTED, "not implement".to_string())
    }

    fn topic(&self) -> &str {
        "/v1.0/im/bot/messages/get"
    }
}

impl Default for RobotHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// AI Graph API handler
pub struct GraphHandler;

impl GraphHandler {
    pub fn new() -> Self {
        Self
    }

    pub fn get_success_response(&self, payload: Option<serde_json::Value>) -> serde_json::Value {
        let body = payload.unwrap_or(serde_json::Value::Null);
        serde_json::json!({
            "statusLine": {
                "code": 200,
                "reasonPhrase": "OK"
            },
            "headers": {
                "Content-Type": "application/json"
            },
            "body": body.to_string()
        })
    }
}

#[async_trait]
impl CallbackHandler for GraphHandler {
    async fn process(&self, _message: &CallbackMessage) -> (i32, String) {
        (NOT_IMPLEMENTED, "not implement".to_string())
    }

    fn topic(&self) -> &str {
        "/v1.0/graph/api/invoke"
    }
}

impl Default for GraphHandler {
    fn default() -> Self {
        Self::new()
    }
}