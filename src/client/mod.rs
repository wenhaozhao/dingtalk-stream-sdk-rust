//! DingTalk Stream Client
//!
//! The main client for connecting to DingTalk and handling messages

use crate::constants::VERSION;

use crate::MessageTopic;
use serde::{Deserialize, Serialize};
use std::time::Duration;

mod dingtalk_stream;
pub use dingtalk_stream::*;

/// Client configuration
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// Whether to enable auto-reconnect
    pub auto_reconnect: bool,
    /// Whether to keep alive the connection
    pub keep_alive: bool,
    /// User agent string
    pub ua: String,
    /// Reconnect interval
    pub reconnect_interval: Duration,
    /// Keep alive interval
    pub keep_alive_interval: Duration,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            auto_reconnect: true,
            keep_alive: true,
            ua: format!("dingtalk-sdk-rust/{}", VERSION),
            reconnect_interval: Duration::from_secs(10),
            keep_alive_interval: Duration::from_secs(60),
        }
    }
}

/// Subscription topic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    /// Message type: EVENT or CALLBACK
    pub topic: MessageTopic,
    /// Topic path
    #[serde(rename = "type")]
    pub sub_type: String,
}

/// Connection response from gateway
#[derive(Debug, Deserialize)]
pub struct ConnectionResponse {
    pub endpoint: String,
    pub ticket: String,
}

/// Access token response
#[derive(Debug, Deserialize)]
pub struct AccessTokenResponse {
    #[serde(rename = "accessToken")]
    pub access_token: String,
    #[serde(rename = "expireIn")]
    pub expire_in: i64,
}

/// Access token cache
#[derive(Clone)]
struct AccessTokenCache {
    token: String,
    expire_time: i64,
}
