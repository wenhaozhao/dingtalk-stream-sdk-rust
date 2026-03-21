use crate::client::{ConnectionResponse, Subscription};
use crate::utils::get_local_ip;
use crate::{DingTalkStream, MessageTopic, GATEWAY_URL};
use futures_util::{SinkExt, StreamExt};
use std::sync::atomic::Ordering;
use anyhow::anyhow;
use tokio::sync::mpsc;
use tokio::time::sleep;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use tracing::{debug, error, info, warn};

impl DingTalkStream {
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
}

impl DingTalkStream {
    /// Connect to DingTalk WebSocket
    async fn connect(&mut self) -> crate::Result<()> {
        let connection = self.open_connection().await.map_err(|err|anyhow!(err))?;
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

    /// Open connection to DingTalk
    async fn open_connection(
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
                    topic: MessageTopic::Event("*".to_string()),
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
                topic: MessageTopic::Event("*".to_string()),
            });
        }
        Ok(topics)
    }
}
