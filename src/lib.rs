//! DingTalk Stream SDK for Rust
//!
//! A Rust implementation of the DingTalk Stream SDK, based on the Node.js and Python SDKs.
//!
//! # Quick Start
//!
//! ```rust
//! use dingtalk_stream::{Credential, DingTalkStream, CallbackHandler, MessageTopic, TOPIC_ROBOT};
//! use dingtalk_stream::handlers::{Resp, Error, ErrorCode};
//! use async_trait::async_trait;
//! use dingtalk_stream::frames::{CallbackMessage, CallbackWebhookMessage};
//! use tokio::sync::mpsc::Sender;
//!
//! // Define a handler for robot messages
//! struct MyRobotHandler(MessageTopic);
//!
//! #[async_trait]
//! impl CallbackHandler for MyRobotHandler {
//!     async fn process(&self, message: &CallbackMessage, cb_webhook_msg_sender: Option<Sender<CallbackWebhookMessage>>) -> Result<Resp, Error> {
//!         // Process the message and return a response
//!         Ok(Resp::Text("Hello from DingTalk SDK!".to_string()))
//!     }
//!
//!     fn topic(&self) -> &MessageTopic {
//!         &self.0
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let credential = Credential::new(
//!         "your-client-id".to_string(),
//!         "your-client-secret".to_string(),
//!     );
//!
//!     let mut client = DingTalkStream::new(credential)
//!         .register_callback_handler(MyRobotHandler(
//!             MessageTopic::Callback(TOPIC_ROBOT.to_string()),
//!         ));
//!
//!     client.start_forever().await;
//!     Ok(())
//! }
//! ```

pub mod client;
pub mod credential;
pub mod frames;
pub mod handlers;
pub mod utils;

pub use client::DingTalkStream;
pub use credential::Credential;
pub use frames::{
    AckMessage, CallbackMessage, EventMessage, MessageHeaders, MessageTopic, SystemMessage,
};
pub use handlers::{CallbackHandler, EventHandler, SystemHandler, Resp, Error, ErrorCode};

// Re-export for convenience
pub use client::ClientConfig;
pub use frames::DownStreamMessage;

pub type Result<T, E = anyhow::Error> = anyhow::Result<T, E>;

/// The DingTalk gateway URL for opening connections
pub const GATEWAY_URL: &str = "https://api.dingtalk.com/v1.0/gateway/connections/open";

/// The DingTalk API endpoint for getting access tokens
pub const GET_TOKEN_URL: &str = "https://api.dingtalk.com/v1.0/oauth2/accessToken";

pub const ROBOT_BATCH_SEND_MESSAGE: &str = "https://api.dingtalk.com/v1.0/robot/oToMessages/batchSend";

/// The topic for robot message callbacks
pub const TOPIC_ROBOT: &str = "/v1.0/im/bot/messages/get";

/// The topic for robot delegate message callbacks
pub const TOPIC_ROBOT_DELEGATE: &str = "/v1.0/im/bot/messages/delegate";

/// The topic for card callback
pub const TOPIC_CARD: &str = "/v1.0/card/instances/callback";

/// The topic for AI Graph API plugin message callbacks
pub const TOPIC_AI_GRAPH_API: &str = "/v1.0/graph/api/invoke";

/// The default DingTalk OpenAPI endpoint
pub const DEFAULT_OPENAPI_ENDPOINT: &str = "https://api.dingtalk.com";

/// SDK version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
