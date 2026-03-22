//! Reply Message Example for DingTalk Stream SDK
//!
//! This example demonstrates how to create a DingTalk bot that replies to user messages.
//!
//! ## How it works
//!
//! 1. Receive a message from a user via WebSocket callback
//! 2. Extract the sessionWebhook from the message
//! 3. Send a reply using the sessionWebhook via HTTP POST
//!
//! ## Usage
//!
//! ```bash
//! export DINGTALK_CLIENT_ID="your-client-id"
//! export DINGTALK_CLIENT_SECRET="your-client-secret"
//! cargo run --example reply_message
//! ```

use async_trait::async_trait;
use dingtalk_stream::frames::{CallbackMessage, CallbackMessageData, CallbackMessagePayload};
use dingtalk_stream::handlers::{Error, ErrorCode, Resp};
use dingtalk_stream::{
    CallbackHandler, Credential, DingTalkStream, MessageTopic, Result, TOPIC_ROBOT,
};
use reqwest::Client;
use serde_json::json;
use std::env;

/// Custom handler for robot messages that sends replies
struct ReplyHandler(MessageTopic);

#[async_trait]
impl CallbackHandler for ReplyHandler {
    async fn process(&self, message: &CallbackMessage) -> Result<Resp, Error> {
        // Get the message data
        let data = message.data.as_ref().ok_or_else(|| Error {
            msg: "No message data".to_string(),
            code: ErrorCode::BadRequest,
        })?;

        // Extract necessary fields for replying
        let session_webhook = data.session_webhook.as_ref().ok_or_else(|| Error {
            msg: "No sessionWebhook found".to_string(),
            code: ErrorCode::BadRequest,
        })?;

        let sender_staff_id = data.sender_staff_id.as_ref().ok_or_else(|| Error {
            msg: "No senderStaffId found".to_string(),
            code: ErrorCode::BadRequest,
        })?;

        // Extract message content
        let content = extract_text_content(data);

        println!("Received message: {}", content);

        // Send a reply to the user
        let response_text = format!("Echo: {}", content);
        let reply_result = send_reply(session_webhook, sender_staff_id, &response_text).await;

        match reply_result {
            Ok(_) => {
                println!("Reply sent successfully");
                Ok(Resp::Text(response_text))
            }
            Err(e) => {
                println!("Failed to send reply: {}", e);
                Err(Error {
                    msg: format!("Failed to send reply: {}", e),
                    code: ErrorCode::InternalServerError,
                })
            }
        }
    }

    fn topic(&self) -> &MessageTopic {
        &self.0
    }
}

/// Extract text content from callback message data
fn extract_text_content(data: &CallbackMessageData) -> String {
    if let Some(CallbackMessagePayload::Text { text }) = &data.payload {
        return text.content.clone();
    }
    // Fallback: try to find text in extensions or other fields
    "Unknown content".to_string()
}

/// Send a text message reply via sessionWebhook
async fn send_reply(
    session_webhook: &str,
    sender_staff_id: &str,
    text: &str,
) -> Result<reqwest::Response> {
    let client = Client::new();

    let body = json!({
        "msgtype": "text",
        "text": {
            "content": text
        },
        "at": {
            "atUserIds": [sender_staff_id],
            "isAtAll": false
        }
    });

    let response = client
        .post(session_webhook)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;

    Ok(response)
}

/// Send a markdown message reply via sessionWebhook
async fn send_markdown_reply(
    session_webhook: &str,
    sender_staff_id: &str,
    title: &str,
    text: &str,
) -> Result<reqwest::Response> {
    let client = Client::new();

    let body = json!({
        "msgtype": "markdown",
        "markdown": {
            "title": title,
            "text": text
        },
        "at": {
            "atUserIds": [sender_staff_id],
            "isAtAll": false
        }
    });

    let response = client
        .post(session_webhook)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;

    Ok(response)
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Get credentials from environment variables
    let client_id =
        env::var("DINGTALK_CLIENT_ID").expect("DINGTALK_CLIENT_ID environment variable not set");
    let client_secret = env::var("DINGTALK_CLIENT_SECRET")
        .expect("DINGTALK_CLIENT_SECRET environment variable not set");

    println!("Starting DingTalk Stream bot with reply...");
    println!("Client ID: {}", client_id);

    // Create credential
    let credential = Credential::new(client_id, client_secret);

    // Create client and register handler
    let mut client = DingTalkStream::new(credential)
        .register_callback_handler(ReplyHandler(MessageTopic::Callback(TOPIC_ROBOT.to_string())));

    // Start the client (will run forever with auto-reconnect)
    client.start_forever().await;

    Ok(())
}
