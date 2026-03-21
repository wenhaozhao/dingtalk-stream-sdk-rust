//! DingTalk Stream Client
//!
//! The main client for connecting to DingTalk and handling messages

use crate::constants::{GATEWAY_URL, GET_TOKEN_URL, TOPIC_ROBOT, VERSION};
use crate::credential::Credential;
use crate::frames::{
    AckMessage, CallbackMessage, ClientDownStream, EventMessage, RobotMessage, SystemMessage,
};
use crate::handlers::{CallbackHandler, EventHandler, SystemHandler};
use crate::utils::get_local_ip;

use crate::{MsgType, SystemTopic};
use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tokio::sync::{mpsc, RwLock};
use tokio::time::sleep;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, error, info, warn};

/// Client configuration
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// Whether to enable auto-reconnect
    pub auto_reconnect: bool,
    /// Whether to keep alive the connection
    pub keep_alive: bool,
    /// User agent string
    pub ua: String,
    /// Reconnect interval
    pub reconnect_interval: Duration,
    /// Keep alive interval
    pub keep_alive_interval: Duration,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            auto_reconnect: true,
            keep_alive: true,
            ua: format!("dingtalk-sdk-rust/{}", VERSION),
            reconnect_interval: Duration::from_secs(10),
            keep_alive_interval: Duration::from_secs(60),
        }
    }
}

/// Subscription topic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    /// Message type: EVENT or CALLBACK
    pub topic: SystemTopic,
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
    event_handler: Option<Box<dyn EventHandler>>,
    /// Callback handlers mapped by topic
    callback_handlers: HashMap<SystemTopic, Box<dyn CallbackHandler>>,
    /// System handler
    system_handler: Option<Box<dyn SystemHandler>>,
    /// WebSocket URL
    ws_url: Option<String>,
    /// Whether connected
    connected: AtomicBool,
    /// Whether registered
    registered: AtomicBool,
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
        Self::with_config(credential, ClientConfig::default())
    }

    /// Create with custom configuration
    pub fn with_config(credential: Credential, config: ClientConfig) -> Self {
        Self {
            credential,
            config,
            event_handler: Default::default(),
            callback_handlers: Default::default(),
            system_handler: Default::default(),
            ws_url: Default::default(),
            connected: Default::default(),
            registered: Default::default(),
            stop_tx: Default::default(),
            access_token: Default::default(),
        }
    }

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

    /// Get access token
    pub async fn get_access_token(
        &self,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // Check cached token
        {
            let cache = self.access_token.read().await;
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
            .unwrap_or(0)
            + token_resp.expire_in
            - 300; // 5 min buffer

        {
            let mut cache = self.access_token.write().await;
            *cache = Some(AccessTokenCache {
                token: token_resp.access_token.clone(),
                expire_time,
            });
        }

        Ok(token_resp.access_token)
    }

    /// Open connection to DingTalk
    pub async fn open_connection(
        &self,
    ) -> Result<ConnectionResponse, Box<dyn std::error::Error + Send + Sync>> {
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

        info!("Opening connection to {}", GATEWAY_URL);
        debug!("Request body: {:?}", request_body);

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

        info!("Connection established: {:?}", connection);

        Ok(connection)
    }

    /// Build subscription list
    fn build_subscriptions(
        &self,
    ) -> Result<Vec<Subscription>, Box<dyn std::error::Error + Send + Sync>> {
        let mut topics = Vec::new();

        // Add event subscription if event handler is registered
        {
            let handler = &self.event_handler;
            if handler.is_some() {
                topics.push(Subscription {
                    sub_type: "EVENT".to_string(),
                    topic: SystemTopic::Event("*".to_string()),
                });
            }
        }

        // Add callback subscriptions
        {
            for topic in self.callback_handlers.keys() {
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
                topic: SystemTopic::Event("*".to_string()),
            });
        }

        Ok(topics)
    }

    /// Connect to DingTalk WebSocket
    pub async fn connect(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let connection = self.open_connection().await?;
        let ws_url = format!("{}?ticket={}", connection.endpoint, connection.ticket);

        info!("Connecting to WebSocket: {}", ws_url);

        self.ws_url.replace(ws_url.clone());

        // Connect to WebSocket
        let (ws_stream, _) = connect_async(&ws_url).await?;
        let (mut write, mut read) = ws_stream.split();

        self.connected.store(true, Ordering::SeqCst);
        info!("Connected to DingTalk WebSocket");

        // Create channel for sending messages
        let (tx, mut rx) = mpsc::channel::<String>(100);

        // Spawn task to forward messages to WebSocket
        let write_task = tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                match write.send(Message::Text(msg.into())).await {
                    Ok(_) => {}
                    Err(e) => {
                        error!("Failed to send message to WebSocket: {}", e);
                        break;
                    }
                }
            }
        });

        // Spawn keep-alive task if enabled
        if self.config.keep_alive {
            let keep_alive_interval = self.config.keep_alive_interval;
            let tx_clone = tx.clone();
            tokio::spawn(async move {
                loop {
                    sleep(keep_alive_interval).await;
                    let ping = serde_json::json!({
                        "code": 200,
                        "message": "ping"
                    });
                    match serde_json::to_string(&ping) {
                        Ok(ping) => {
                            let _ = tx_clone.send(ping).await;
                        }
                        Err(err) => {
                            error!("write ping to json failed: {err}")
                        }
                    }
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
        self.connected.store(false, Ordering::SeqCst);
        self.registered.store(false, Ordering::SeqCst);
        Ok(())
    }

    /// Handle incoming message
    async fn handle_message(
        &mut self,
        text: &str,
        tx: mpsc::Sender<String>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let msg: ClientDownStream = serde_json::from_str(text)?;
        debug!("Received message: {:?}", msg);
        match msg.msg_type{
            MsgType::System => {
                self.handle_system(msg, tx).await?;
            }
            MsgType::Event => {
                self.handle_event(msg, tx).await?;
            }
            MsgType::Callback => {
                self.handle_callback(msg, tx).await?;
            }
            other => {
                warn!("Unknown message type: {}", other);
            }
        }

        Ok(())
    }

    /// Handle system message
    async fn handle_system(
        &mut self,
        msg: ClientDownStream,
        tx: mpsc::Sender<String>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(topic) = &msg.headers.topic {
            match topic {
                SystemTopic::Connected => {
                    info!("Connection established");
                }
                SystemTopic::Registered => {
                    self.registered.store(true, Ordering::SeqCst);
                    info!("Registered successfully");
                }
                SystemTopic::Disconnect => {
                    self.connected.store(false, Ordering::SeqCst);
                    self.registered.store(false, Ordering::SeqCst);
                    info!("Disconnected by server");
                }
                SystemTopic::KeepAlive => {
                    // Heartbeat received
                }
                SystemTopic::Ping => {
                    // Respond to ping
                    let response = serde_json::json!({
                        "code": 200,
                        "headers": msg.headers,
                        "message": "OK",
                        "data": msg.data,
                    });
                    let _ = tx.send(response.to_string()).await;
                }
                SystemTopic::Event(_) => {}
            }
            {
                // Call custom system handler if registered
                if let Some(handler) = &self.system_handler {
                    let system_msg = SystemMessage::from_stream(msg);
                    let _ = handler.process(&system_msg).await;
                }
            }
        } else {
            warn!("System message without topic, skipping processing");
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
        if let Some(ref handler) = &self.event_handler {
            let (code, response_msg) = handler.process(&event_msg).await;
            let ack = AckMessage::ok(&response_msg)
                .with_message_id(event_msg.headers.message_id.clone().unwrap_or_default())
                .with_content_type("application/json");
            let _ = tx.send(serde_json::to_string(&ack)?).await;
            debug!("Event processed with code: {}", code);
        }
        Ok(())
    }

    /// Handle callback message
    async fn handle_callback(
        &self,
        msg: ClientDownStream,
        tx: mpsc::Sender<String>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(topic) = &msg.headers.topic {
            // Find handler for this topic
            if let Some(handler) = self.callback_handlers.get(topic) {
                let callback_msg = CallbackMessage::from_stream(msg);
                let (code, response_msg) = handler.process(&callback_msg).await;
                let ack = AckMessage::ok(&response_msg)
                    .with_message_id(callback_msg.headers.message_id.clone().unwrap_or_default())
                    .with_content_type("application/json")
                    .with_data(serde_json::json!({ "response": response_msg }));
                let ack_json = serde_json::to_string(&ack)?;
                let _ = tx.send(ack_json).await;
                debug!("Callback processed with code: {}", code);
            } else {
                warn!("No handler registered for topic: {}", topic);
            }
        } else {
            warn!("Callback message without topic, skipping processing");
        }
        Ok(())
    }

    /// Send a message response
    pub async fn send(
        &self,
        message_id: &str,
        data: serde_json::Value,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let msg = serde_json::json!({
            "code": 200,
            "headers": {
                "contentType": "application/json",
                "messageId": message_id,
            },
            "message": "OK",
            "data": serde_json::to_string(&data)?,
        });

        debug!("Sending message: {:?}", msg);

        Ok(())
    }

    /// Send callback response (for robot messages)
    pub async fn socket_callback_response(
        &self,
        message_id: &str,
        result: serde_json::Value,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.send(message_id, serde_json::json!({ "response": result }))
            .await
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
    pub async fn start_forever(&mut self) {
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

            info!(
                "Reconnecting in {} seconds...",
                self.config.reconnect_interval.as_secs()
            );
            sleep(self.config.reconnect_interval).await;
        }
    }

    /// Stop the client
    pub async fn stop(&self) {
        if let Some(tx) = self.stop_tx.write().await.take() {
            let _ = tx.send(()).await;
        }
    }

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

/// Simple robot handler for convenience
pub struct RobotMessageHandler {
    topic: SystemTopic,
}

impl RobotMessageHandler {
    pub fn new() -> Self {
        Self {
            topic: SystemTopic::Event(TOPIC_ROBOT.into()),
        }
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

    fn topic(&self) -> &SystemTopic {
        &self.topic
    }
}

impl Default for RobotMessageHandler {
    fn default() -> Self {
        Self::new()
    }
}
