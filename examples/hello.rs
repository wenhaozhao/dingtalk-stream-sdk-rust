//! Hello World Example for DingTalk Stream SDK
//!
//! This example demonstrates how to create a simple DingTalk bot that responds to messages.

use async_trait::async_trait;
use dingtalk_stream::frames::{CallbackMessageData, CallbackMessagePayload};
use dingtalk_stream::handlers::{Error, ErrorCode, Resp};
use dingtalk_stream::{
    CallbackHandler, CallbackMessage, Credential, DingTalkStream, MessageTopic, TOPIC_ROBOT,
};
use std::env;
use std::string::ToString;

/// Custom handler for robot messages
struct RobotMessageHandler(MessageTopic);

#[async_trait]
impl CallbackHandler for RobotMessageHandler {
    async fn process(&self, message: &CallbackMessage) -> Result<Resp, Error> {
        // Extract text from the message
        if let Some(data) = &message.data {
            println!("{}", serde_json::to_string_pretty(&data).unwrap());
            let CallbackMessageData { payload, .. } = &data;
            if let Some(CallbackMessagePayload::Text { text }) = payload {
                println!("Received message: {}", text.content);
                // You would typically send a response back here
                // For now, just echo the message
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
    let mut client = DingTalkStream::new(credential).register_callback_handler(
        RobotMessageHandler(MessageTopic::Callback(TOPIC_ROBOT.to_string())),
    );
    // Start the client (will run forever with auto-reconnect)
    client.start_forever().await;
}
