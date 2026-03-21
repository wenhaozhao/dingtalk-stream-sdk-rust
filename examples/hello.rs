//! Hello World Example for DingTalk Stream SDK
//!
//! This example demonstrates how to create a simple DingTalk bot that responds to messages.

use dingtalk_stream::{
    Credential, DingTalkStream, CallbackHandler, CallbackMessage,
    TOPIC_ROBOT,
};
use async_trait::async_trait;
use std::env;

/// Custom handler for robot messages
struct RobotMessageHandler;

#[async_trait]
impl CallbackHandler for RobotMessageHandler {
    async fn process(&self, message: &CallbackMessage) -> (i32, String) {
        // Extract text from the message
        if let Some(data) = &message.data {
            if let Some(text_obj) = data.get("text") {
                if let Some(content) = text_obj.get("content").and_then(|v| v.as_str()) {
                    println!("Received message: {}", content);

                    // You would typically send a response back here
                    // For now, just echo the message
                    return (200, format!("Echo: {}", content));
                }
            }
        }
        (404, "not implement".to_string())
    }

    fn topic(&self) -> &str {
        TOPIC_ROBOT
    }
}

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Get credentials from environment variables
    let client_id = env::var("DINGTALK_CLIENT_ID")
        .expect("DINGTALK_CLIENT_ID environment variable not set");
    let client_secret = env::var("DINGTALK_CLIENT_SECRET")
        .expect("DINGTALK_CLIENT_SECRET environment variable not set");

    println!("Starting DingTalk Stream bot...");
    println!("Client ID: {}", client_id);

    // Create credential
    let credential = Credential::new(client_id, client_secret);

    // Create client with debug mode
    let client = DingTalkStream::new(credential).with_debug(true);

    // Register robot message handler
    client.register_callback_handler(TOPIC_ROBOT, RobotMessageHandler);

    // Start the client (will run forever with auto-reconnect)
    client.start_forever().await;
}