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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_credential_new() {
        let cred = Credential::new(
            "test_client_id".to_string(),
            "test_client_secret".to_string(),
        );
        assert_eq!(cred.client_id, "test_client_id");
        assert_eq!(cred.client_secret, "test_client_secret");
    }
}