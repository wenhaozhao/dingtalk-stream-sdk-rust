//! Hello World Example for DingTalk Stream SDK
//!
//! This example demonstrates how to create a simple DingTalk bot that responds to messages.

use async_trait::async_trait;
use dingtalk_stream::client::{DingTalkMedia, DingtalkResource, MediaImage};
use dingtalk_stream::frames::down_message::callback_message::{
    CallbackMessage, MessageData, MessagePayload, RichTextItem,
};
use dingtalk_stream::frames::down_message::MessageTopic;
use dingtalk_stream::frames::up_message::callback_message::WebhookMessage;
use dingtalk_stream::frames::up_message::robot_message::{RobotMessage, RobotPrivateMessage};
use dingtalk_stream::frames::up_message::MessageContent;
use dingtalk_stream::handlers::{CallbackHandler, Error, ErrorCode, Resp};
use dingtalk_stream::{Credential, DingTalkStream, TOPIC_ROBOT};
use std::env;
use std::path::PathBuf;
use std::str::FromStr;
use std::string::ToString;
use std::sync::Arc;

/// Custom handler for robot messages
struct RobotMessageHandler(MessageTopic);

const TMP_DIR: &str = "/var/tmp";

#[async_trait]
impl CallbackHandler for RobotMessageHandler {
    async fn process(
        &self,
        client: Arc<DingTalkStream>,
        message: &CallbackMessage,
        cb_webhook_msg_sender: Option<tokio::sync::mpsc::Sender<WebhookMessage>>,
    ) -> Result<Resp, Error> {
        // Extract text from the message
        if let Some(data) = &message.data {
            println!("{}", serde_json::to_string_pretty(&data).unwrap());
            let MessageData { payload, .. } = &data;
            match payload {
                Some(MessagePayload::Text { text }) => {
                    println!("Received message: {}", text.content);
                    // You would typically send a response back here
                    // For now, just echo the message
                    if let Some(sender) = cb_webhook_msg_sender {
                        let _ = sender
                            .send(WebhookMessage {
                                content: MessageContent::Text {
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
                Some(MessagePayload::Picture { content }) => {
                    match content.fetch(&client, TMP_DIR.into()).await {
                        Ok((filepath, _)) => {
                            println!("Image fetched successfully: {}", filepath.display());
                        }
                        Err(err) => {
                            println!("Error fetching image: {:?}", err);
                        }
                    }
                    return Ok(Resp::Text("Echo: unexpected".to_string()));
                }
                Some(MessagePayload::File { content }) => {
                    match content.fetch(&client, TMP_DIR.into()).await {
                        Ok((filepath, _)) => {
                            println!("file fetched successfully: {}", filepath.display());
                        }
                        Err(err) => {
                            println!("Error fetching file: {:?}", err);
                        }
                    }
                    return Ok(Resp::Text("Echo: unexpected".to_string()));
                }
                Some(MessagePayload::RichText { content }) => {
                    for content in content.iter() {
                        match content {
                            RichTextItem::Text(text) => {
                                println!("{text}");
                            }
                            RichTextItem::Picture(content) => {
                                match content.fetch(&client, TMP_DIR.into()).await {
                                    Ok((filepath, _)) => {
                                        println!("{}", filepath.display());
                                    }
                                    Err(err) => {
                                        println!("Error fetching image: {:?}", err);
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {
                    return Ok(Resp::Text("Echo: unexpected".to_string()));
                }
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
    let (dingtalk_stream, _) = Arc::new(
        DingTalkStream::new(credential)
            .register_callback_handler(
                RobotMessageHandler(MessageTopic::Callback(TOPIC_ROBOT.to_string())).into(),
            )
            .await,
    )
    .start()
    .await
    .unwrap();

    let media_image = MediaImage::from(PathBuf::from_str("test_resources/img.png").unwrap());
    let result = media_image.upload(&dingtalk_stream).await.unwrap();
    println!("Media upload result: {:?}", result);

    let _ = dingtalk_stream
        .send_message(
            RobotMessage::from(RobotPrivateMessage {
                user_ids: vec!["12345".into()],
                content: "Hello, World!".into(),
            })
            .with_cb(|result| {
                println!("{result:?}");
            }),
        )
        .await;
    let _ = tokio::signal::ctrl_c().await;
}
