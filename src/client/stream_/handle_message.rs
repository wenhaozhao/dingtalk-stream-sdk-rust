use crate::frames::DownStreamMessageData;
use crate::{AckMessage, DingTalkStream, DownStreamMessage, MessageTopic};
use std::sync::atomic::Ordering;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

impl DingTalkStream {
    /// Handle incoming message
    pub(super) async fn handle_message(
        &mut self,
        text: &str,
        tx: mpsc::Sender<String>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        debug!("Received message: {:?}", text);
        match serde_json::from_str::<DownStreamMessage>(text) {
            Ok(msg) => match msg.data {
                Some(DownStreamMessageData::System { .. }) => {
                    self.handle_system(msg, tx).await?;
                }
                Some(DownStreamMessageData::Event { .. }) => {
                    self.handle_event(msg, tx).await?;
                }
                Some(DownStreamMessageData::Callback { .. }) => {
                    self.handle_callback(msg, tx).await?;
                }
                _ => {}
            },
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
        &mut self,
        msg: DownStreamMessage,
        tx: mpsc::Sender<String>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(topic) = &msg.headers.topic {
            match topic {
                MessageTopic::Connected => {
                    info!("Connection established");
                }
                MessageTopic::Registered => {
                    self.registered.store(true, Ordering::SeqCst);
                    info!("Registered successfully");
                }
                MessageTopic::Disconnect => {
                    self.connected.store(false, Ordering::SeqCst);
                    self.registered.store(false, Ordering::SeqCst);
                    info!("Disconnected by server");
                }
                MessageTopic::KeepAlive => {
                    // Heartbeat received
                }
                MessageTopic::Ping => {
                    // Respond to ping
                    let response = serde_json::json!({
                        "code": 200,
                        "headers": msg.headers,
                        "message": "OK",
                        "data": msg.data,
                    });
                    let _ = tx.send(response.to_string()).await;
                }
                MessageTopic::Event(_) => {}
            }
            {
                // Call custom system handler if registered
                if let Some(handler) = &self.system_handler {
                    let Some(data) = msg.data else {
                        warn!("data is empty");
                        return Ok(());
                    };
                    let DownStreamMessageData::System { data: system_msg } = data else {
                        unreachable!("expected system-message")
                    };
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
        msg: DownStreamMessage,
        tx: mpsc::Sender<String>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Call event handler
        if let Some(ref handler) = &self.event_handler {
            let Some(data) = msg.data else {
                warn!("data is empty");
                return Ok(());
            };
            let DownStreamMessageData::Event { data: event_msg } = data else {
                unreachable!("expected event-message")
            };
            let (code, response_msg) = match handler.process(&event_msg).await {
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
        &self,
        msg: DownStreamMessage,
        tx: mpsc::Sender<String>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(topic) = &msg.headers.topic {
            let Some(data) = msg.data else {
                warn!("data is empty");
                return Ok(());
            };
            let DownStreamMessageData::Callback { data: callback_msg } = data else {
                unreachable!("expected callback-message")
            };
            // Find handler for this topic
            if let Some(handler) = self.callback_handlers.get(topic) {
                let (code, response_msg) = match handler.process(&callback_msg).await {
                    Ok(result) => (200, result.to_string()),
                    Err(err) => (err.code as i32, err.msg),
                };
                let ack = AckMessage::ok(&response_msg)
                    .with_message_id(callback_msg.headers.message_id.clone().unwrap_or_default())
                    .with_content_type("application/json")
                    .with_data(serde_json::json!({ "response": response_msg }));
                let _ = tx.send(serde_json::to_string(&ack)?).await;
                debug!("Callback processed with code: {}", code);
            } else {
                warn!("No handler registered for topic: {}", topic);
            }
        } else {
            warn!("Callback message without topic, skipping processing");
        }
        Ok(())
    }
}
