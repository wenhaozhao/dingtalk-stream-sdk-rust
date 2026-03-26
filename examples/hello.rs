//! Hello World Example for DingTalk Stream SDK
//!
//! This example demonstrates how to create a simple DingTalk bot that responds to messages.

use async_trait::async_trait;
use dingtalk_stream::frames::{
    CallbackMessageData, CallbackMessagePayload, CallbackWebhookMessage, RobotPrivateMessage,
    UpMessageContent,
};
use dingtalk_stream::handlers::{Error, ErrorCode, Resp};
use dingtalk_stream::{
    CallbackHandler, CallbackMessage, Credential, DingTalkStream, MessageTopic, TOPIC_ROBOT,
};
use std::env;
use std::string::ToString;
use std::sync::Arc;
use std::time::Duration;

/// Custom handler for robot messages
struct RobotMessageHandler(MessageTopic);

#[async_trait]
impl CallbackHandler for RobotMessageHandler {
    async fn process(
        &self,
        message: &CallbackMessage,
        cb_webhook_msg_sender: Option<tokio::sync::mpsc::Sender<CallbackWebhookMessage>>,
    ) -> Result<Resp, Error> {
        // Extract text from the message
        if let Some(data) = &message.data {
            println!("{}", serde_json::to_string_pretty(&data).unwrap());
            let CallbackMessageData { payload, .. } = &data;
            if let Some(CallbackMessagePayload::Text { text }) = payload {
                println!("Received message: {}", text.content);
                // You would typically send a response back here
                // For now, just echo the message
                if let Some(sender) = cb_webhook_msg_sender {
                    let _ = sender
                        .send(CallbackWebhookMessage {
                            content: UpMessageContent::Text {
                                text: format!("echo {}", text.content).into(),
                            },
                            at: Default::default(),
                            send_result_cb: Some(Arc::new(|result| {
                                println!("Message sent result: {:?}", result);
                            })),
                        })
                        .await;
                }
                return Ok(Resp::Text(format!("Echo: {}", text.content)));
            }
        }
        Err(Error {
            msg: "No text payload found".to_string(),
            code: ErrorCode::BadRequest,
        })
    }

    fn topic(&self) -> &MessageTopic {
        &self.0
    }
}

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Get credentials from environment variables
    let client_id =
        env::var("DINGTALK_CLIENT_ID").expect("DINGTALK_CLIENT_ID environment variable not set");
    let client_secret = env::var("DINGTALK_CLIENT_SECRET")
        .expect("DINGTALK_CLIENT_SECRET environment variable not set");

    println!("Starting DingTalk Stream bot...");
    println!("Client ID: {}", client_id);

    // Create credential
    let credential = Credential::new(client_id, client_secret);

    // Create client with debug mode
    let (mut dingtalk_stream, message_sender) = DingTalkStream::new(credential)
        .register_callback_handler(RobotMessageHandler(MessageTopic::Callback(
            TOPIC_ROBOT.to_string(),
        )))
        .create_message_sender()
        .await;

    // Start the client (will run forever with auto-reconnect)
    tokio::spawn(async move {
        dingtalk_stream.start_forever().await;
    });
    for _ in 0..10 {
        tokio::time::sleep(Duration::from_secs(1)).await;
        let _ = message_sender
            .send(
                RobotPrivateMessage {
                    user_ids: vec!["12345".into()],
                    content: "Hello, World!".into(),
                    send_result_cb: Some(Arc::new(|result| {
                        println!("{result:?}");
                    })),
                }
                .into(),
            )
            .await;
    }
    let _ = tokio::signal::ctrl_c().await;
}
