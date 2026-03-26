use crate::client::AccessTokenCache;
use crate::frames::RobotMessage;
use crate::{CallbackHandler, ClientConfig, Credential, EventHandler, MessageTopic, SystemHandler};
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

mod access_token;
mod handle_message;
mod lifecycle;
mod send_message;

/// DingTalk Stream Client
pub struct DingTalkStream {
    /// Credential for authentication
    credential: Credential,
    /// Client configuration
    config: ClientConfig,
    /// Event handler
    event_handler: Option<Box<dyn EventHandler>>,
    /// Callback handlers mapped by topic
    callback_handlers: HashMap<MessageTopic, Box<dyn CallbackHandler>>,
    /// System handler
    system_handler: Option<Box<dyn SystemHandler>>,
    /// WebSocket URL
    ws_url: Option<String>,
    /// Whether connected
    connected: AtomicBool,
    /// Whether registered
    registered: AtomicBool,
    /// Access token cache
    access_token: Arc<RwLock<Option<AccessTokenCache>>>,
    http_client: reqwest::Client,
}

impl DingTalkStream {
    /// Create a new DingTalk Stream client
    pub fn new(credential: Credential) -> Self {
        Self::with_config(credential, ClientConfig::default())
    }

    /// Create with custom configuration
    pub fn with_config(credential: Credential, config: ClientConfig) -> Self {
        Self {
            credential,
            config,
            event_handler: None,
            callback_handlers: HashMap::default(),
            system_handler: None,
            ws_url: None,
            connected: AtomicBool::new(false),
            registered: AtomicBool::new(false),
            access_token: Default::default(),
            http_client: reqwest::Client::default(),
        }
    }
}

#[derive(Clone)]
pub struct DingtalkMessageSender(pub(super) tokio::sync::mpsc::Sender<RobotMessage>);

impl Deref for DingtalkMessageSender {
    type Target = tokio::sync::mpsc::Sender<RobotMessage>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DingTalkStream {
    /// Register an event handler
    pub fn register_event_handler<H: EventHandler + 'static>(&mut self, handler: H) -> &mut Self {
        self.event_handler.replace(Box::new(handler));
        self
    }

    /// Register a callback handler for a specific topic
    pub fn register_callback_handler<H: CallbackHandler + 'static>(mut self, handler: H) -> Self {
        let topic = handler.topic().clone();
        self.callback_handlers.insert(topic, Box::new(handler));
        self
    }

    /// Register a system handler
    pub async fn register_system_handler<H: SystemHandler + 'static>(
        &mut self,
        handler: H,
    ) -> &mut Self {
        self.system_handler.replace(Box::new(handler));
        self
    }
}

impl DingTalkStream {
    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.connected.load(Ordering::Relaxed)
    }

    /// Check if registered
    pub fn is_registered(&self) -> bool {
        self.registered.load(Ordering::Relaxed)
    }

    /// Get the credential
    pub fn credential(&self) -> &Credential {
        &self.credential
    }

    /// Get configuration
    pub fn config(&self) -> &ClientConfig {
        &self.config
    }
}
