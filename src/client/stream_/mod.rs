use crate::client::{AccessTokenCache, AccessTokenResponse};
use crate::{
    CallbackHandler, ClientConfig, Credential, EventHandler, MessageTopic, SystemHandler,
    GET_TOKEN_URL,
};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::{mpsc, RwLock};
use tracing::debug;

mod handle_message;
mod lifecycle;

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
    /// Stop signal sender
    stop_tx: RwLock<Option<mpsc::Sender<()>>>,
    /// Access token cache
    access_token: RwLock<Option<AccessTokenCache>>,
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
impl DingTalkStream {
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
}

impl DingTalkStream {
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
}
