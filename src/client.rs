//! DingTalk Stream Client
//!
//! The main client for connecting to DingTalk and handling messages

use crate::credential::Credential;
use crate::frames::{
    AckMessage, CallbackMessage, ClientDownStream, EventMessage, SystemMessage, RobotMessage,
    TOPIC_CONNECTED, TOPIC_DISCONNECT, TOPIC_KEEPALIVE, TOPIC_PING, TOPIC_REGISTERED,
    MSG_TYPE_EVENT, MSG_TYPE_CALLBACK, MSG_TYPE_SYSTEM,
};
use crate::handlers::{CallbackHandler, EventHandler, SystemHandler};
use crate::constants::{GATEWAY_URL, GET_TOKEN_URL, TOPIC_ROBOT, VERSION};
use crate::utils::get_local_ip;

use async_trait::async_trait;
use futures_util::{StreamExt, SinkExt};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::sleep;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{error, info, warn, debug};

/// Client configuration
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// Whether to enable auto-reconnect
    pub auto_reconnect: bool,
    /// Whether to keep alive the connection
    pub keep_alive: bool,
    /// User agent string
    pub ua: String,
    /// Debug mode
    pub debug: bool,
    /// Reconnect interval in seconds
    pub reconnect_interval: u64,
    /// Keep alive interval in seconds
    pub keep_alive_interval: u64,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            auto_reconnect: true,
            keep_alive: false,
            ua: format!("dingtalk-sdk-rust/{}", VERSION),
            debug: false,
            reconnect_interval: 10,
            keep_alive_interval: 60,
        }
    }
}

/// Subscription topic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    /// Message type: EVENT or CALLBACK
    pub topic: String,
    /// Topic path
    #[serde(rename = "type")]
    pub sub_type: String,
}

/// Connection response from gateway
#[derive(Debug, Deserialize)]
pub struct ConnectionResponse {
    pub endpoint: String,
    pub ticket: String,
}

/// Access token response
#[derive(Debug, Deserialize)]
pub struct AccessTokenResponse {
    #[serde(rename = "accessToken")]
    pub access_token: String,
    #[serde(rename = "expireIn")]
    pub expire_in: i64,
}

/// DingTalk Stream Client
pub struct DingTalkStream {
    /// Credential for authentication
    credential: Credential,
    /// Client configuration
    config: ClientConfig,
    /// Event handler
    event_handler: Arc<RwLock<Option<Box<dyn EventHandler>>>>,
    /// Callback handlers mapped by topic
    callback_handlers: Arc<RwLock<HashMap<String, Box<dyn CallbackHandler>>>>,
    /// System handler
    system_handler: Arc<RwLock<Option<Box<dyn SystemHandler>>>>,
    /// WebSocket URL
    ws_url: RwLock<Option<String>>,
    /// Whether connected
    connected: RwLock<bool>,
    /// Whether registered
    registered: RwLock<bool>,
    /// Stop signal sender
    stop_tx: RwLock<Option<mpsc::Sender<()>>>,
    /// Access token cache
    access_token: RwLock<Option<AccessTokenCache>>,
}

/// Access token cache
#[derive(Clone)]
struct AccessTokenCache {
    token: String,
    expire_time: i64,
}

impl DingTalkStream {
    /// Create a new DingTalk Stream client
    pub fn new(credential: Credential) -> Self {
        Self {
            credential,
            config: ClientConfig::default(),
            event_handler: Arc::new(RwLock::new(None)),
            callback_handlers: Arc::new(RwLock::new(HashMap::new())),
            system_handler: Arc::new(RwLock::new(None)),
            ws_url: RwLock::new(None),
            connected: RwLock::new(false),
            registered: RwLock::new(false),
            stop_tx: RwLock::new(None),
            access_token: RwLock::new(None),
        }
    }

    /// Create with custom configuration
    pub fn with_config(credential: Credential, config: ClientConfig) -> Self {
        Self {
            credential,
            config,
            event_handler: Arc::new(RwLock::new(None)),
            callback_handlers: Arc::new(RwLock::new(HashMap::new())),
            system_handler: Arc::new(RwLock::new(None)),
            ws_url: RwLock::new(None),
            connected: RwLock::new(false),
            registered: RwLock::new(false),
            stop_tx: RwLock::new(None),
            access_token: RwLock::new(None),
        }
    }

    /// Set debug mode
    pub fn with_debug(mut self, debug: bool) -> Self {
        self.config.debug = debug;
        self
    }

    /// Register an event handler
    pub fn register_event_handler<H: EventHandler + 'static>(&self, handler: H) {
        let mut guard = self.event_handler.write();
        *guard = Some(Box::new(handler));
    }

    /// Register a callback handler for a specific topic
    pub fn register_callback_handler<H: CallbackHandler + 'static>(&self, topic: &str, handler: H) {
        let mut handlers = self.callback_handlers.write();
        handlers.insert(topic.to_string(), Box::new(handler));
    }

    /// Register a system handler
    pub fn register_system_handler<H: SystemHandler + 'static>(&self, handler: H) {
        let mut guard = self.system_handler.write();
        *guard = Some(Box::new(handler));
    }

    /// Get access token
    pub async fn get_access_token(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // Check cached token
        {
            let cache = self.access_token.read();
            if let Some(ref cache) = *cache {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0);
                if now < cache.expire_time {
                    return Ok(cache.token.clone());
                }
            }
        }

        let client = reqwest::Client::new();
        let response = client
            .post(GET_TOKEN_URL)
            .json(&serde_json::json!({
                "appKey": self.credential.client_id,
                "appSecret": self.credential.client_secret,
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err("Failed to get access token".into());
        }

        let token_resp: AccessTokenResponse = response.json().await?;
        let expire_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0) + token_resp.expire_in - 300; // 5 min buffer

        {
            let mut cache = self.access_token.write();
            *cache = Some(AccessTokenCache {
                token: token_resp.access_token.clone(),
                expire_time,
            });
        }

        Ok(token_resp.access_token)
    }

    /// Open connection to DingTalk
    pub async fn open_connection(&self) -> Result<ConnectionResponse, Box<dyn std::error::Error + Send + Sync>> {
        let subscriptions = self.build_subscriptions()?;

        let client = reqwest::Client::new();
        let local_ip = get_local_ip().unwrap_or_else(|| "127.0.0.1".to_string());

        let request_body = serde_json::json!({
            "clientId": self.credential.client_id,
            "clientSecret": self.credential.client_secret,
            "subscriptions": subscriptions,
            "ua": self.config.ua,
            "localIp": local_ip,
        });

        if self.config.debug {
            info!("Opening connection to {}", GATEWAY_URL);
            debug!("Request body: {:?}", request_body);
        }

        let response = client
            .post(GATEWAY_URL)
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let text = response.text().await?;
            error!("Failed to open connection: {}", text);
            return Err(format!("Failed to open connection: {}", text).into());
        }

        let connection: ConnectionResponse = response.json().await?;

        if self.config.debug {
            info!("Connection established: {:?}", connection);
        }

        Ok(connection)
    }

    /// Build subscription list
    fn build_subscriptions(&self) -> Result<Vec<Subscription>, Box<dyn std::error::Error + Send + Sync>> {
        let mut topics = Vec::new();

        // Add event subscription if event handler is registered
        {
            let guard = self.event_handler.read();
            if guard.is_some() {
                topics.push(Subscription {
                    sub_type: "EVENT".to_string(),
                    topic: "*".to_string(),
                });
            }
        }

        // Add callback subscriptions
        {
            let handlers = self.callback_handlers.read();
            for topic in handlers.keys() {
                topics.push(Subscription {
                    sub_type: "CALLBACK".to_string(),
                    topic: topic.clone(),
                });
            }
        }

        if topics.is_empty() {
            // Default to all events if no handlers registered
            topics.push(Subscription {
                sub_type: "EVENT".to_string(),
                topic: "*".to_string(),
            });
        }

        Ok(topics)
    }

    /// Connect to DingTalk WebSocket
    pub async fn connect(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let connection = self.open_connection().await?;
        let ws_url = format!("{}?ticket={}", connection.endpoint, connection.ticket);

        if self.config.debug {
            info!("Connecting to WebSocket: {}", ws_url);
        }

        *self.ws_url.write() = Some(ws_url.clone());

        // Connect to WebSocket
        let (ws_stream, _) = connect_async(&ws_url).await?;
        let (mut write, mut read) = ws_stream.split();

        *self.connected.write() = true;
        info!("Connected to DingTalk WebSocket");

        // Create channel for sending messages
        let (tx, mut rx) = mpsc::channel::<String>(100);

        // Spawn task to forward messages to WebSocket
        let write_task = tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                if write.send(Message::Text(msg.into())).await.is_err() {
                    break;
                }
            }
        });

        // Spawn keep-alive task if enabled
        if self.config.keep_alive {
            let keep_alive_interval = self.config.keep_alive_interval;
            let tx_clone = tx.clone();
            tokio::spawn(async move {
                loop {
                    sleep(Duration::from_secs(keep_alive_interval)).await;
                    let ping = serde_json::json!({
                        "code": 200,
                        "message": "ping"
                    });
                    let _ = tx_clone.send(ping.to_string()).await;
                }
            });
        }

        // Handle incoming messages
        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    let tx_clone = tx.clone();
                    if let Err(e) = self.handle_message(&text, tx_clone).await {
                        error!("Error handling message: {}", e);
                    }
                }
                Ok(Message::Close(_)) => {
                    warn!("WebSocket connection closed");
                    break;
                }
                Err(e) => {
                    error!("WebSocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }

        write_task.abort();
        *self.connected.write() = false;
        *self.registered.write() = false;

        Ok(())
    }

    /// Handle incoming message
    async fn handle_message(
        &self,
        text: &str,
        tx: mpsc::Sender<String>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let msg: ClientDownStream = serde_json::from_str(text)?;

        if self.config.debug {
            debug!("Received message: {:?}", msg);
        }

        match msg.msg_type.as_str() {
            MSG_TYPE_SYSTEM => {
                self.handle_system(msg, tx).await?;
            }
            MSG_TYPE_EVENT => {
                self.handle_event(msg, tx).await?;
            }
            MSG_TYPE_CALLBACK => {
                self.handle_callback(msg, tx).await?;
            }
            _ => {
                warn!("Unknown message type: {}", msg.msg_type);
            }
        }

        Ok(())
    }

    /// Handle system message
    async fn handle_system(
        &self,
        msg: ClientDownStream,
        tx: mpsc::Sender<String>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let topic = msg.headers.topic.as_deref().unwrap_or("");

        match topic {
            TOPIC_CONNECTED => {
                info!("Connection established");
            }
            TOPIC_REGISTERED => {
                *self.registered.write() = true;
                info!("Registered successfully");
            }
            TOPIC_DISCONNECT => {
                *self.connected.write() = false;
                *self.registered.write() = false;
                info!("Disconnected by server");
            }
            TOPIC_KEEPALIVE => {
                // Heartbeat received
            }
            TOPIC_PING => {
                // Respond to ping
                let response = serde_json::json!({
                    "code": 200,
                    "headers": msg.headers,
                    "message": "OK",
                    "data": msg.data,
                });
                let _ = tx.send(response.to_string()).await;
            }
            _ => {
                warn!("Unknown system topic: {}", topic);
            }
        }

        // Call custom system handler if registered
        {
            let guard = self.system_handler.read();
            if let Some(ref handler) = *guard {
                let system_msg = SystemMessage::from_stream(msg);
                let _ = handler.process(&system_msg).await;
            }
        }

        Ok(())
    }

    /// Handle event message
    async fn handle_event(
        &self,
        msg: ClientDownStream,
        tx: mpsc::Sender<String>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let event_msg = EventMessage::from_stream(msg);

        // Call event handler
        let guard = self.event_handler.read();
        if let Some(ref handler) = *guard {
            let (code, response_msg) = handler.process(&event_msg).await;
            let ack = AckMessage::ok(&response_msg)
                .with_message_id(event_msg.headers.message_id.clone().unwrap_or_default())
                .with_content_type("application/json");
            let _ = tx.send(serde_json::to_string(&ack)?).await;

            if self.config.debug {
                debug!("Event processed with code: {}", code);
            }
        }

        Ok(())
    }

    /// Handle callback message
    async fn handle_callback(
        &self,
        msg: ClientDownStream,
        tx: mpsc::Sender<String>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let topic = msg.headers.topic.clone().unwrap_or_default();
        let topic_str = topic.as_str();
        let callback_msg = CallbackMessage::from_stream(msg);

        // Find handler for this topic
        let handlers = self.callback_handlers.read();
        if let Some(handler) = handlers.get(topic_str) {
            let (code, response_msg) = handler.process(&callback_msg).await;
            let ack = AckMessage::ok(&response_msg)
                .with_message_id(callback_msg.headers.message_id.clone().unwrap_or_default())
                .with_content_type("application/json")
                .with_data(serde_json::json!({ "response": response_msg }));
            let _ = tx.send(serde_json::to_string(&ack)?).await;

            if self.config.debug {
                debug!("Callback processed with code: {}", code);
            }
        } else {
            warn!("No handler registered for topic: {}", topic);
        }

        Ok(())
    }

    /// Send a message response
    pub async fn send(&self, message_id: &str, data: serde_json::Value) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let msg = serde_json::json!({
            "code": 200,
            "headers": {
                "contentType": "application/json",
                "messageId": message_id,
            },
            "message": "OK",
            "data": serde_json::to_string(&data)?,
        });

        if self.config.debug {
            debug!("Sending message: {:?}", msg);
        }

        Ok(())
    }

    /// Send callback response (for robot messages)
    pub async fn socket_callback_response(
        &self,
        message_id: &str,
        result: serde_json::Value,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.send(message_id, serde_json::json!({ "response": result })).await
    }

    /// Send Graph API response
    pub async fn send_graph_api_response(
        &self,
        message_id: &str,
        response: serde_json::Value,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.send(message_id, response).await
    }

    /// Start the client and run forever (auto-reconnect)
    pub async fn start_forever(&self) {
        info!("Starting DingTalk Stream client...");

        loop {
            match self.connect().await {
                Ok(_) => {
                    info!("Connection closed normally");
                }
                Err(e) => {
                    error!("Connection error: {}", e);
                }
            }

            if !self.config.auto_reconnect {
                break;
            }

            info!("Reconnecting in {} seconds...", self.config.reconnect_interval);
            sleep(Duration::from_secs(self.config.reconnect_interval)).await;
        }
    }

    /// Stop the client
    pub async fn stop(&self) {
        if let Some(tx) = self.stop_tx.write().take() {
            let _ = tx.send(()).await;
        }
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        *self.connected.read()
    }

    /// Check if registered
    pub fn is_registered(&self) -> bool {
        *self.registered.read()
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

/// Simple robot handler for convenience
pub struct RobotMessageHandler;

impl RobotMessageHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl CallbackHandler for RobotMessageHandler {
    async fn process(&self, message: &CallbackMessage) -> (i32, String) {
        // Parse robot message
        if let Some(data) = &message.data {
            if let Ok(robot_msg) = serde_json::from_value::<RobotMessage>(data.clone()) {
                if let Some(text) = robot_msg.get_text() {
                    return (200, text);
                }
            }
        }
        (404, "not implement".to_string())
    }

    fn topic(&self) -> &str {
        TOPIC_ROBOT
    }
}

impl Default for RobotMessageHandler {
    fn default() -> Self {
        Self::new()
    }
}