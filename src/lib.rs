//! DingTalk Stream SDK for Rust
//!
//! A Rust implementation of the DingTalk Stream SDK, based on the Node.js and Python SDKs.
//!
//! # Quick Start
//!
//! ```rust
//! use dingtalk_stream::{DingTalkStream, Credential, CallbackHandler, EventHandler};
//! use serde_json::json;
//!
//! #[tokio::main]
//! async fn main() {
//!     let credential = Credential::new(
//!         "your-client-id".to_string(),
//!         "your-client-secret".to_string(),
//!     );
//!
//!     let mut client = DingTalkStream::new(credential);
//!
//!     // Register robot message callback handler
//!     client.register_callback_handler("/v1.0/im/bot/messages/get", RobotHandler);
//!
//!     // Start the client
//!     client.start_forever().await;
//! }
//! ```

pub mod client;
pub mod credential;
pub mod frames;
pub mod handlers;
pub mod constants;
pub mod utils;

pub use client::DingTalkStream;
pub use credential::Credential;
pub use frames::{EventMessage, CallbackMessage, SystemMessage, AckMessage, Headers, RobotMessage};
pub use handlers::{EventHandler, CallbackHandler, SystemHandler, RobotHandler, GraphHandler};
pub use constants::*;

// Re-export for convenience
pub use client::ClientConfig;
pub use client::RobotMessageHandler;
pub use frames::ClientDownStream;