use crate::client::DingTalkStream;
use crate::frames::down_message::{callback_message::SessionWebhook, MessageType};
use crate::frames::{
    down_message::{
        event_message::EventMessage, system_message::SystemMessage, DownStreamMessage, MessageTopic,
    },
    AckMessage,
};
use std::sync::Arc;

use crate::frames::down_message::callback_message::{CallbackMessage, MessageData};
use crate::frames::up_message::callback_message::WebhookMessage;
use anyhow::anyhow;
use std::sync::atomic::Ordering;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Receiver;
use tracing::{debug, error, info, warn};

impl DingTalkStream {
    /// Handle incoming message
    pub(super) async fn handle_message(
        self: Arc<Self>,
        text: &str,
        tx: mpsc::Sender<String>,
    ) -> crate::Result<()> {
        debug!("Received message: {:?}", text);
        let downstream_message = serde_json::from_str::<DownStreamMessage>(text);
        match downstream_message {
            Ok(
                message @ DownStreamMessage {
                    r#type: MessageType::System,
                    ..
                },
            ) => {
                self.handle_system(message, tx).await?;
            }
            Ok(
                message @ DownStreamMessage {
                    r#type: MessageType::Event,
                    ..
                },
            ) => {
                self.handle_event(message, tx).await?;
            }
            Ok(
                message @ DownStreamMessage {
                    r#type: MessageType::Callback,
                    ..
                },
            ) => {
                self.handle_callback(message, tx).await?;
            }
            Err(err) => {
                error!("Failed to parse message, err: {}", err);
            }
        }
        Ok(())
    }
}
impl DingTalkStream {
    /// Handle system message
    async fn handle_system(
        self: Arc<Self>,
        msg: DownStreamMessage,
        tx: mpsc::Sender<String>,
    ) -> crate::Result<()> {
        match &msg.headers.topic {
            Some(MessageTopic::Connected) => {
                info!("Connection established");
            }
            Some(MessageTopic::Registered) => {
                self.registered.store(true, Ordering::SeqCst);
                info!("Registered successfully");
            }
            Some(MessageTopic::Disconnect) => {
                self.connected.store(false, Ordering::SeqCst);
                self.registered.store(false, Ordering::SeqCst);
                info!("Disconnected by server");
            }
            Some(MessageTopic::KeepAlive) => {
                // Heartbeat received
            }
            Some(MessageTopic::Ping) => {
                // Respond to ping
                let response = serde_json::json!({
                    "code": 200,
                    "headers": msg.headers,
                    "message": "OK",
                    "data": msg.data,
                });
                let _ = tx.send(response.to_string()).await;
            }
            None | Some(MessageTopic::Callback(_)) => {}
        }
        // Call custom system handler if registered
        if let Some(handler) = &self.system_handler {
            let Ok(sys_msg) = SystemMessage::try_from(msg) else {
                warn!("Failed to parse system message, skipping processing");
                return Ok(());
            };
            let _ = handler.process(Arc::clone(&self), &sys_msg).await;
        }
        Ok(())
    }

    /// Handle event message
    async fn handle_event(
        self: Arc<Self>,
        msg: DownStreamMessage,
        tx: mpsc::Sender<String>,
    ) -> crate::Result<()> {
        // Call event handler
        if let Some(handler) = &self.event_handler {
            let Ok(event_msg) = EventMessage::try_from(msg) else {
                warn!("Failed to parse event message, skipping processing");
                return Ok(());
            };
            let (code, response_msg) = match handler.process(Arc::clone(&self), &event_msg).await {
                Ok(result) => (200, result.to_string()),
                Err(err) => (err.code as i32, err.msg),
            };
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
        self: Arc<Self>,
        msg: DownStreamMessage,
        tx: mpsc::Sender<String>,
    ) -> crate::Result<()> {
        // Find handler for this topic
        let Some(topic) = msg.headers.topic.clone() else {
            warn!("Callback message without topic, skipping processing");
            return Ok(());
        };
        let Ok(cb_msg) = CallbackMessage::try_from(msg) else {
            warn!("Failed to parse callback message, skipping processing");
            return Ok(());
        };
        let sender = if let Some(MessageData {
            session_webhook: Some(session_webhook),
            ..
        }) = &cb_msg.data
        {
            let (sender, receiver) = mpsc::channel(1024);
            let http_client = self.http_client.clone();
            let session_webhook = session_webhook.clone();
            tokio::spawn(async move {
                Self::handle_webhook_message(http_client, session_webhook, receiver).await
            });
            Some(sender)
        } else {
            None
        };
        let Some(handler) = self.callback_handlers.get(&topic) else {
            warn!("No handler registered for topic: {}", topic);
            return Ok(());
        };
        let (code, response_msg) = match handler.process(Arc::clone(&self), &cb_msg, sender).await {
            Ok(result) => (200, result.to_string()),
            Err(err) => (err.code as i32, err.msg),
        };
        let ack = AckMessage::ok(&response_msg)
            .with_message_id(cb_msg.headers.message_id.clone().unwrap_or_default())
            .with_content_type("application/json")
            .with_data(serde_json::json!({ "response": response_msg }));
        let _ = tx.send(serde_json::to_string(&ack)?).await;
        debug!("Callback processed with code: {}", code);
        Ok(())
    }

    async fn handle_webhook_message(
        http_client: reqwest::Client,
        session_webhook: SessionWebhook,
        mut receiver: Receiver<WebhookMessage>,
    ) {
        if let (Ok(webhook_url), Some(timeout)) =
            (session_webhook.webhook_url(), session_webhook.timeout())
        {
            match tokio::time::timeout(timeout, async {
                while let Some(message) = receiver.recv().await {
                    let message @ WebhookMessage { send_result_cb, .. } = &message;
                    let response = http_client
                        .post(webhook_url.clone())
                        .header("Content-Type", "application/json")
                        .header("Accept", "*/*")
                        .json(&message)
                        .send()
                        .await;
                    if let Some(cb) = send_result_cb {
                        match response {
                            Ok(response) => {
                                let code = response.status();
                                match response.text().await {
                                    Ok(text) => cb(Ok((code.as_u16(), text))),
                                    Err(err) => cb(Err(anyhow!("{err}"))),
                                }
                            }
                            Err(err) => cb(Err(anyhow!("{err}"))),
                        }
                    }
                }
            })
            .await
            {
                Ok(()) => {}
                Err(_) => {
                    info!(
                        "webhook_url: {} elapsed after {}",
                        webhook_url,
                        timeout.as_millis()
                    )
                }
            }
        }
    }
}
