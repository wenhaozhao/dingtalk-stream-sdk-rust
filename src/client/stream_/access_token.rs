use crate::client::{AccessToken, AccessTokenCache, AccessTokenResponse};
use crate::{Credential, DingTalkStream, GET_TOKEN_URL};
use anyhow::anyhow;
use std::sync::Arc;
use tokio::sync::RwLock;

impl DingTalkStream {
    pub(super) async fn get_access_token(&self) -> crate::Result<AccessToken> {
        Self::get_access_token_(
            &self.http_client,
            &self.credential,
            Arc::clone(&self.access_token),
        )
        .await
    }
    /// Get access token
    pub(super) async fn get_access_token_(
        http_client: &reqwest::Client,
        credential: &Credential,
        access_token: Arc<RwLock<Option<AccessTokenCache>>>,
    ) -> crate::Result<AccessToken> {
        // Check cached token
        {
            let cache = access_token.read().await;
            if let Some(ref cache) = *cache {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0);
                if now < cache.expire_time {
                    return Ok(cache.token.clone());
                }
            }
        }

        let response = http_client
            .post(GET_TOKEN_URL)
            .json(&serde_json::json!({
                "appKey": credential.client_id,
                "appSecret": credential.client_secret,
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to get access token"));
        }

        let token_resp: AccessTokenResponse = response.json().await?;
        let expire_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0)
            + token_resp.expire_in
            - 300; // 5 min buffer

        let access_token = {
            let mut cache = access_token.write().await;
            let access_token = AccessToken(token_resp.access_token);
            *cache = Some(AccessTokenCache {
                token: access_token.clone(),
                expire_time,
            });
            access_token
        };
        Ok(access_token)
    }
}
