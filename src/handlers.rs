//! Handlers module for DingTalk Stream SDK
//!
//! Provides trait-based handlers for different message types

use crate::frames::down_message::callback_message::CallbackMessage;
use crate::frames::down_message::event_message::EventMessage;
use crate::frames::down_message::system_message::SystemMessage;
use crate::frames::down_message::MessageTopic;
use crate::frames::up_message::callback_message::WebhookMessage;
use crate::DingTalkStream;
use async_trait::async_trait;
use std::fmt::{Display, Formatter};
use tokio::sync::mpsc::Sender;
use tokio_tungstenite::tungstenite::Message;

/// Callback handler trait for handling callback messages
#[async_trait]
pub trait CallbackHandler: Send + Sync {
    /// Process a callback message
    async fn process(
        &self,
        client: &DingTalkStream,
        message: &CallbackMessage,
        cb_webhook_msg_sender: Option<Sender<WebhookMessage>>,
    ) -> Result<Resp, Error>;

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

#[derive(Debug, Copy, Clone)]
pub enum LifecycleEvent<'a> {
    Start,
    Connecting {
        websocket_url: &'a str,
    },
    Connected {
        websocket_url: &'a str,
    },
    WebsocketWrite {
        payload: &'a str,
        result: &'a crate::Result<()>,
    },
    WebsocketWriteWithRetry {
        payload: &'a str,
        cnt: u8,
        result: &'a crate::Result<()>,
    },
    WebsocketRead {
        result: &'a crate::Result<Message>,
    },
    Keepalive {
        payload: &'a str,
        result: &'a crate::Result<()>,
    },
    Disconnected {
        result: &'a crate::Result<()>,
    },
    Stopped,
}
#[allow(unused)]
#[async_trait]
pub trait LifecycleListener: Send + Sync {
    async fn on_event<'a>(&self, client: &DingTalkStream, event: LifecycleEvent<'a>) {}

    async fn on_start(&self, client: &DingTalkStream) {
        let _ = self.on_event(client, LifecycleEvent::Start).await;
    }

    async fn on_connecting(&self, client: &DingTalkStream, websocket_url: &str) {
        let _ = self
            .on_event(client, LifecycleEvent::Connecting { websocket_url })
            .await;
    }

    async fn on_connected(&self, client: &DingTalkStream, websocket_url: &str) {
        let _ = self
            .on_event(client, LifecycleEvent::Connected { websocket_url })
            .await;
    }

    async fn on_websocket_write(
        &self,
        client: &DingTalkStream,
        payload: &str,
        result: &crate::Result<()>,
    ) {
        let _ = self
            .on_event(client, LifecycleEvent::WebsocketWrite { payload, result })
            .await;
    }

    async fn on_websocket_write_with_retry(
        &self,
        client: &DingTalkStream,
        payload: &str,
        cnt: u8,
        result: &crate::Result<()>,
    ) {
        let _ = self
            .on_event(
                client,
                LifecycleEvent::WebsocketWriteWithRetry {
                    payload,
                    cnt,
                    result,
                },
            )
            .await;
    }

    async fn on_websocket_read(&self, client: &DingTalkStream, result: &crate::Result<Message>) {
        let _ = self
            .on_event(client, LifecycleEvent::WebsocketRead { result })
            .await;
    }

    async fn on_keepalive(
        &self,
        client: &DingTalkStream,
        payload: &str,
        result: &crate::Result<()>,
    ) {
        let _ = self
            .on_event(client, LifecycleEvent::Keepalive { payload, result })
            .await;
    }

    async fn on_disconnected(&self, client: &DingTalkStream, result: &crate::Result<()>) {
        let _ = self
            .on_event(client, LifecycleEvent::Disconnected { result })
            .await;
    }

    async fn on_stopped(&self, client: &DingTalkStream) {
        let _ = self.on_event(client, LifecycleEvent::Stopped).await;
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct DefaultLifecycleListener;

impl LifecycleListener for DefaultLifecycleListener {}
