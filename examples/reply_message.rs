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
use chrono::{Local, TimeZone};
use dingtalk_stream::frames::{CallbackMessage, CallbackMessageData, CallbackMessagePayload};
use dingtalk_stream::handlers::{Error, ErrorCode, Resp};
use dingtalk_stream::{
    CallbackHandler, Credential, DingTalkStream, MessageTopic, Result, TOPIC_ROBOT,
};
use std::env;

/// Custom handler for robot messages that sends replies using SDK methods
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
        let session_webhook_expired_time = data.session_webhook_expired_time.unwrap_or(0);
        let expire_time = Local.timestamp_millis_opt(session_webhook_expired_time).unwrap();
        println!("{session_webhook}, expire-at: {}", expire_time.format("%Y-%m-%d %H:%M:%S"));
        let sender_staff_id = data.sender_staff_id.as_ref().ok_or_else(|| Error {
            msg: "No senderStaffId found".to_string(),
            code: ErrorCode::BadRequest,
        })?;

        // Extract message content
        let content = extract_text_content(data);

        println!("Received message: {}", content);

        // Send a reply to the user using SDK's send_text method
        let response_text = format!("Echo: {}", content);
        let client = DingTalkStream::new(Credential::new(
            "dummy".to_string(),
            "dummy".to_string(),
        ));


        // Use SDK's send_text method
        match client
            .send_text(
                session_webhook,
                &response_text,
                Some(vec![sender_staff_id.clone()]),
                false,
            )
            .await
        {
            Ok(result) => {
                println!("Reply sent successfully: {:?}", result);
            }
            Err(e) => {
                println!("Failed to send reply: {}", e);
                return Err(Error {
                    msg: format!("Failed to send reply: {}", e),
                    code: ErrorCode::InternalServerError,
                });
            }
        };

        // Use SDK's send_text method
        match client
            .send_markdown(
                session_webhook,
                "send_markdown title",
                &format!(r#"
### hello {}
- A
- B
                "#, &response_text),
                Some(vec![sender_staff_id.clone()]),
                false,
            )
            .await
        {
            Ok(result) => {
                println!("Reply send_markdown successfully: {:?}", result);
            }
            Err(e) => {
                println!("Failed to send reply: {}", e);
                return Err(Error {
                    msg: format!("Failed to send reply: {}", e),
                    code: ErrorCode::InternalServerError,
                })
            }
        }
        Ok(Resp::Text("ack".to_string()))

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

/// Example: Send markdown message
async fn send_markdown_example(client: &DingTalkStream, session_webhook: &str) -> Result<()> {
    // Send a markdown message
    let result = client
        .send_markdown(
            session_webhook,
            "Title",
            "## Hello\nThis is a markdown message",
            None,
            false,
        )
        .await?;

    println!("Markdown sent: {:?}", result);
    Ok(())
}

/// Example: Send link message
async fn send_link_example(client: &DingTalkStream, session_webhook: &str) -> Result<()> {
    // Send a link message
    let result = client
        .send_link(
            session_webhook,
            "Link Title",
            "Click to open",
            "https://open.dingtalk.com",
            None,
        )
        .await?;

    println!("Link sent: {:?}", result);
    Ok(())
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
        .register_callback_handler(ReplyHandler(MessageTopic::Callback(
            TOPIC_ROBOT.to_string(),
        )));

    // Start the client (will run forever with auto-reconnect)
    client.start_forever().await;

    Ok(())
}