use crate::client::{ConnectionResponse, StopSignalSender, Subscription};
use crate::frames::OK;
use crate::utils::get_local_ip;
use crate::{DingTalkStream, MessageTopic, GATEWAY_URL};
use anyhow::anyhow;
use futures_util::{SinkExt, StreamExt};
use std::sync::atomic::Ordering;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
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
                    return;
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
    pub async fn stop(stop_tx: Sender<()>) -> crate::Result<()> {
        let _ = stop_tx.send(()).await;
        Ok(())
    }
}

impl DingTalkStream {
    /// Connect to DingTalk WebSocket
    async fn connect(&mut self) -> crate::Result<()> {
        let connection = self.open_connection().await.map_err(|err| anyhow!(err))?;
        let ws_url = format!("{}?ticket={}", connection.endpoint, connection.ticket);
        info!("Connecting to WebSocket: {}", ws_url);
        self.ws_url.replace(ws_url.clone());
        // Connect to WebSocket
        let (ws_stream, _) = connect_async(&ws_url).await?;
        let (mut ws_write, mut ws_read) = ws_stream.split();
        self.connected.store(true, Ordering::SeqCst);
        info!("Connected to DingTalk WebSocket");
        // Create channel for sending messages
        let (stream_tx, mut stream_rx) = mpsc::channel::<String>(1024);
        let mut stop_signal_rx = {
            let (tx, rx) = mpsc::channel::<()>(1);
            let mut stop_tx = self.stop_tx.lock().await;
            stop_tx.replace(StopSignalSender(tx));
            rx
        };
        // Spawn task to forward messages to WebSocket
        let write_task = tokio::spawn(async move {
            loop {
                tokio::select! {
                    msg = stream_rx.recv() => {
                        if let Some(msg) = msg {
                            match ws_write.send(Message::Text(msg.into())).await {
                                Ok(_) => {}
                                Err(e) => {
                                    error!("Failed to send message to WebSocket: {}", e);
                                    break;
                                }
                            }
                        }
                    },
                    _ = stop_signal_rx.recv() => {
                        break;
                    }
                }
            }
        });
        // Spawn keep-alive task if enabled
        if self.config.keep_alive {
            let keep_alive_interval = self.config.keep_alive_interval;
            let stream_tx = stream_tx.clone();
            tokio::spawn(async move {
                loop {
                    sleep(keep_alive_interval).await;
                    let ping = serde_json::json!({
                        "code": 200,
                        "message": "ping"
                    });
                    match serde_json::to_string(&ping) {
                        Ok(ping) => {
                            let Err(_) = stream_tx.send(ping).await else {
                                continue;
                            };
                            warn!("stream_tx dropped, keepalive task stopping.");
                            return;
                        }
                        Err(err) => {
                            error!("write ping to json failed: {err}")
                        }
                    }
                }
            });
        }
        // Handle incoming messages
        let mut error = None;
        while let Some(msg) = ws_read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    let tx_clone = stream_tx.clone();
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
                    error.replace(e);
                    break;
                }
                _ => {}
            }
        }
        write_task.abort();
        self.connected.store(false, Ordering::SeqCst);
        self.registered.store(false, Ordering::SeqCst);
        if let Some(e) = error {
            Err(e.into())
        } else {
            Ok(())
        }
    }

    /// Open connection to DingTalk
    async fn open_connection(
        &self,
    ) -> Result<ConnectionResponse, Box<dyn std::error::Error + Send + Sync>> {
        let subscriptions = self.build_subscriptions()?;

        let client = &self.http_client;
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
                    topic: MessageTopic::Callback("*".to_string()),
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
                topic: MessageTopic::Callback("*".to_string()),
            });
        }
        Ok(topics)
    }
}
