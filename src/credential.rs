//! Credential module for DingTalk Stream SDK

use serde::{Deserialize, Serialize};

/// Credential for authenticating with DingTalk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credential {
    /// The client ID (app key)
    pub client_id: String,
    /// The client secret (app secret)
    pub client_secret: String,
}

impl Credential {
    /// Create a new Credential
    pub fn new(client_id: String, client_secret: String) -> Self {
        Self {
            client_id,
            client_secret,
        }
    }

    pub fn from_env() -> Self {
        let client_id = std::env::var("DINGTALK_CLIENT_ID")
            .expect("DINGTALK_CLIENT_ID environment variable must be set");
        let client_secret = std::env::var("DINGTALK_CLIENT_SECRET")
            .expect("DINGTALK_CLIENT_SECRET environment variable must be set");
        Self::new(client_id, client_secret)
    }
}
