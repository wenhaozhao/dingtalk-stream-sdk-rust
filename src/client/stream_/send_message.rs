use crate::frames::{RobotGroupMessage, RobotMessage, RobotPrivateMessage};
use crate::{ROBOT_SEND_GROUP_MESSAGE, ROBOT_SEND_PRIVATE_MESSAGE};
use anyhow::anyhow;
use serde_json::json;

impl super::DingTalkStream {
    pub async fn send_message(&self, message: RobotMessage) -> crate::Result<()> {
        let http_client = self.http_client.clone();
        let credential = self.credential.clone();
        let access_token = self.get_access_token().await?;
        match message {
            RobotMessage::Private(message) => {
                let _ = Self::send_private_message(
                    &http_client,
                    &access_token,
                    &credential.client_id,
                    &message,
                )
                .await?;
            }
            RobotMessage::Group(message) => {
                let _ = Self::send_group_message(
                    &http_client,
                    &access_token,
                    &credential.client_id,
                    &message,
                )
                .await?;
            }
        };
        Ok(())
    }
}

impl super::DingTalkStream {
    async fn send_private_message(
        http_client: &reqwest::Client,
        access_token: &str,
        client_id: &str,
        RobotPrivateMessage {
            user_ids,
            content,
            send_result_cb,
        }: &RobotPrivateMessage,
    ) -> crate::Result<()> {
        let (msg_key, msg_param) = content.to_up_msg()?;
        let response = http_client
            .post(ROBOT_SEND_PRIVATE_MESSAGE)
            .header("x-acs-dingtalk-access-token", access_token)
            .header("Content-Type", "application/json")
            .json(&json!({
                "robotCode": client_id,
                "userIds": user_ids,
                "msgParam": msg_param,
                "msgKey": msg_key
            }))
            .send()
            .await;
        if let Some(cb) = send_result_cb {
            match response {
                Ok(response) => {
                    let code = response.status();
                    match response.text().await {
                        Ok(text) => cb(Ok((code.as_u16(), text))),
                        Err(err) => cb(Err(anyhow!("{err}"))),
                    }
                }
                Err(err) => cb(Err(anyhow!("{err}"))),
            }
        }
        Ok(())
    }

    async fn send_group_message(
        http_client: &reqwest::Client,
        access_token: &str,
        client_id: &str,
        RobotGroupMessage {
            group_id,
            content,
            send_result_cb,
        }: &RobotGroupMessage,
    ) -> crate::Result<()> {
        let (msg_key, msg_param) = content.to_up_msg()?;
        let response = http_client
            .post(ROBOT_SEND_GROUP_MESSAGE)
            .header("x-acs-dingtalk-access-token", access_token)
            .header("Content-Type", "application/json")
            .json(&json!({
                "robotCode": client_id,
                "openConversationId": group_id,
                "msgParam": msg_param,
                "msgKey": msg_key
            }))
            .send()
            .await;
        if let Some(cb) = send_result_cb {
            match response {
                Ok(response) => {
                    let code = response.status();
                    match response.text().await {
                        Ok(text) => cb(Ok((code.as_u16(), text))),
                        Err(err) => cb(Err(anyhow!("{err}"))),
                    }
                }
                Err(err) => cb(Err(anyhow!("{err}"))),
            }
        }
        Ok(())
    }
}
