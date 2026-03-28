use crate::client::{AccessToken, AccessTokenCache, AccessTokenResponse};
use crate::{DingTalkStream, GET_TOKEN_URL};
use anyhow::anyhow;

impl DingTalkStream {
    pub(super) async fn get_access_token(&self) -> crate::Result<AccessToken> {
        // Check cached token
        {
            let cache = self.access_token.read().await;
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

        let response = self
            .http_client
            .post(GET_TOKEN_URL)
            .json(&serde_json::json!({
                "appKey": self.credential.client_id,
                "appSecret":self.credential.client_secret,
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
            let mut cache = self.access_token.write().await;
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
