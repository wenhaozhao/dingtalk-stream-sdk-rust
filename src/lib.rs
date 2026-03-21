//! DingTalk Stream SDK for Rust
//!
//! A Rust implementation of the DingTalk Stream SDK, based on the Node.js and Python SDKs.
//!
//! # Quick Start
//!
//! ```rust
//! todo add it
//! ```

pub mod client;
pub mod credential;
pub mod frames;
pub mod handlers;
pub mod constants;
pub mod utils;

pub use client::DingTalkStream;
pub use credential::Credential;
pub use frames::{EventMessage, CallbackMessage, SystemMessage, AckMessage, MessageHeaders, MessageTopic};
pub use handlers::{EventHandler, CallbackHandler, SystemHandler};
pub use constants::*;

// Re-export for convenience
pub use client::ClientConfig;
pub use frames::DownStreamMessage;