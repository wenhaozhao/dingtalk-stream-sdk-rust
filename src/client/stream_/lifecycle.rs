use crate::client::{ConnectionResponse, Subscription};
use crate::utils::get_local_ip;
use crate::{DingTalkStream, GATEWAY_URL};
use anyhow::anyhow;
use futures_util::{SinkExt, Stream, StreamExt};
use std::fmt::Display;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Receiver;
use tokio::task::JoinHandle;
use tokio::time::sleep;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use tracing::{debug, error, info, warn};
use crate::frames::down_message::MessageTopic;

impl DingTalkStream {
    /// Start the client and run forever (auto-reconnect)
    pub async fn start(
        self: Arc<Self>,
    ) -> crate::Result<(Arc<Self>, JoinHandle<crate::Result<()>>)> {
        info!("Starting DingTalk Stream client...");
        let self_ = Arc::clone(&self);
        let join_handle = tokio::spawn(async move {
            loop {
                match Arc::clone(&self_).connect().await {
                    Ok(_) => {
                        info!("Connection closed normally");
                        return Ok(());
                    }
                    Err(e) => {
                        error!("Connection error: {}", e);
                        if self_.config.auto_reconnect {
                            info!(
                                "Reconnecting in {} seconds...",
                                self_.config.reconnect_interval.as_secs()
                            );
                            sleep(self_.config.reconnect_interval).await;
                        } else {
                            return Err(anyhow!(e));
                        }
                    }
                }
            }
        });
        Ok((self, join_handle))
    }
}

impl DingTalkStream {
    /// Connect to DingTalk WebSocket
    async fn connect(self: Arc<Self>) -> crate::Result<()> {
        let connection = self.open_connection().await.map_err(|err| anyhow!(err))?;
        let ws_url = format!("{}?ticket={}", connection.endpoint, connection.ticket);
        info!("Connecting to WebSocket: {}", ws_url);
        // Connect to WebSocket
        let (ws_stream, _) = connect_async(&ws_url).await?;
        let (ws_write, ws_read) = ws_stream.split();
        self.connected.store(true, Ordering::SeqCst);
        info!("Connected to DingTalk WebSocket");

        let (ws_write_join_handle, ws_read_handle) = {
            // Create channel for sending messages
            let (msg_stream_sender, msg_stream_receiver) = mpsc::channel::<String>(256);
            let ws_write_join_handle = Arc::clone(&self)
                .ws_write(ws_write, msg_stream_receiver)
                .await;
            // Spawn keep-alive task if enabled
            let _ = self.keepalive(msg_stream_sender.clone()).await;
            let ws_read_handle = Arc::clone(&self).ws_read(ws_read, msg_stream_sender).await;
            (ws_write_join_handle, ws_read_handle)
        };

        if let Ok(exit_normally) = ws_read_handle.await {
            exit_normally?;
        }
        if let Ok(exit_normally) = ws_write_join_handle.await {
            exit_normally?;
        }
        self.connected.store(false, Ordering::SeqCst);
        self.registered.store(false, Ordering::SeqCst);
        Ok(())
    }

    async fn ws_write<Sink>(
        self: Arc<Self>,
        mut ws_write: Sink,
        mut msg_stream_receiver: Receiver<String>,
    ) -> JoinHandle<crate::Result<()>>
    where
        Sink: SinkExt<Message> + Unpin + Send + Sync + 'static,
        <Sink as futures_util::Sink<Message>>::Error: Display + Into<anyhow::Error> + Send + Sync,
    {
        tokio::spawn(async move {
            while let Some(ref msg) = msg_stream_receiver.recv().await {
                match self.ws_write_with_retry(&mut ws_write, msg).await {
                    Ok(_) => {}
                    Err(err) => {
                        return Err(anyhow!(err));
                    }
                }
            }
            Ok(())
        })
    }
    async fn ws_write_with_retry<W>(&self, ws_write: &mut W, msg: &str) -> crate::Result<()>
    where
        W: SinkExt<Message> + Unpin,
        <W as futures_util::Sink<Message>>::Error: Display + Into<anyhow::Error>,
    {
        const TRY_INTERVAL: Duration = Duration::from_secs(1);
        const TRY_MAX: u8 = 8;
        info!("Sending message to WebSocket, msg: {}", msg);
        let mut cnt = 1;
        loop {
            match ws_write.send(Message::Text(msg.into())).await {
                Ok(_) => {
                    info!("Success to send message to WebSocket, {}", msg);
                    return Ok(());
                }
                Err(err) => {
                    if {
                        cnt += 1;
                        cnt
                    } > TRY_MAX
                    {
                        warn!("Failed to send message to WebSocket, retrying in 1 second, err: {}, msg: {}", err, msg);
                        tokio::time::sleep(TRY_INTERVAL).await;
                        continue;
                    }
                    error!(
                        "Failed to send message to WebSocket, after {} retries, err: {}, msg: {}",
                        err, cnt, msg
                    );
                    return Err(anyhow!(err));
                }
            }
        }
    }

    async fn ws_read<R, E>(
        self: Arc<Self>,
        mut ws_read: R,
        msg_stream_sender: mpsc::Sender<String>,
    ) -> JoinHandle<crate::Result<()>>
    where
        E: Display + Into<anyhow::Error> + Send + Sync,
        R: Stream<Item = Result<Message, E>> + Unpin + Send + Sync + 'static,
    {
        tokio::spawn(async move {
            while let Some(msg) = ws_read.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        info!("Received text message: {}", text);
                        if let Err(e) = self.handle_message(&text, msg_stream_sender.clone()).await
                        {
                            error!("Error handling message: {}", e);
                        }
                    }
                    Ok(Message::Close(_)) => {
                        warn!("Received close message: WebSocket connection will be closed!!!");
                        return Ok(());
                    }
                    Err(err) => {
                        error!("WebSocket error: {}", err);
                        return Err(anyhow!(err));
                    }
                    _ => continue,
                }
            }
            unreachable!()
        })
    }

    async fn keepalive(&self, msg_stream_sender: mpsc::Sender<String>) -> JoinHandle<()> {
        let keep_alive_interval = self.config.keep_alive_interval;
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(keep_alive_interval).await;
                const PING: &str = r#"{"code": 200,"message": "ping"}"#;
                match msg_stream_sender.send(PING.into()).await {
                    Ok(_) => {
                        continue;
                    }
                    Err(err) => {
                        warn!("stream_tx dropped error, keepalive task stopping. err: {err}");
                        return;
                    }
                }
            }
        })
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
