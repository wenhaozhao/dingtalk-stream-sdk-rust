use crate::frames::MessageType;
use crate::{
    AckMessage, CallbackMessage, DingTalkStream, DownStreamMessage, EventMessage, MessageTopic,
    SystemMessage,
};
use std::sync::atomic::Ordering;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

impl DingTalkStream {
    /// Handle incoming message
    pub(super) async fn handle_message(
        &mut self,
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
        &mut self,
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
            let _ = handler.process(&sys_msg).await;
        }
        Ok(())
    }

    /// Handle event message
    async fn handle_event(
        &self,
        msg: DownStreamMessage,
        tx: mpsc::Sender<String>,
    ) -> crate::Result<()> {
        // Call event handler
        if let Some(handler) = &self.event_handler {
            let Ok(event_msg) = EventMessage::try_from(msg) else {
                warn!("Failed to parse event message, skipping processing");
                return Ok(());
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
        let Some(handler) = self.callback_handlers.get(&topic) else {
            warn!("No handler registered for topic: {}", topic);
            return Ok(());
        };
        let (code, response_msg) = match handler.process(&cb_msg).await {
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
}
