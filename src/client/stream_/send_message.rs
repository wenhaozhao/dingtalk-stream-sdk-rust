use crate::frames::up_message::robot_message::{RobotGroupMessage, RobotMessage, RobotPrivateMessage};
use crate::frames::{SendMessageCallback, SendMessageCallbackData};
use crate::{ROBOT_SEND_GROUP_MESSAGE, ROBOT_SEND_PRIVATE_MESSAGE};
use anyhow::anyhow;
use serde_json::json;
use tracing::info;

impl super::DingTalkStream {
    pub async fn send_message<M: Into<RobotMessage>>(&self, message: M) -> crate::Result<()> {
        let message = message.into();
        let http_client = self.http_client.clone();
        let credential = self.credential.clone();
        let access_token = self.get_access_token().await?;
        match message {
            RobotMessage::Private {
                message,
                send_result_cb,
            } => {
                let _ = Self::send_private_message(
                    &http_client,
                    &access_token,
                    &credential.client_id,
                    &message,
                    send_result_cb.as_ref(),
                )
                .await?;
            }
            RobotMessage::Group {
                message,
                send_result_cb,
            } => {
                let _ = Self::send_group_message(
                    &http_client,
                    &access_token,
                    &credential.client_id,
                    &message,
                    send_result_cb.as_ref(),
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
        RobotPrivateMessage { user_ids, content }: &RobotPrivateMessage,
        send_result_cb: Option<&SendMessageCallback>,
    ) -> crate::Result<()> {
        let (msg_key, msg_param) = content.to_up_msg()?;
        let msg_md5 = format!("{:x}", md5::compute(&msg_param));
        info!("Sending private robot-message[{}]: {}", msg_md5, msg_param);
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
            Self::exec_send_cb(response, cb).await;
        }
        info!("Send private robot-message[{}] ok", msg_md5);
        Ok(())
    }

    async fn send_group_message(
        http_client: &reqwest::Client,
        access_token: &str,
        client_id: &str,
        RobotGroupMessage { group_id, content }: &RobotGroupMessage,
        send_result_cb: Option<&SendMessageCallback>,
    ) -> crate::Result<()> {
        let (msg_key, msg_param) = content.to_up_msg()?;
        let msg_md5 = format!("{:x}", md5::compute(&msg_param));
        info!("Sending group robot-message[{}]: {}", msg_md5, msg_param);
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
            Self::exec_send_cb(response, cb).await;
        }
        info!("Send group robot-message[{}] ok", msg_md5);
        Ok(())
    }

    async fn exec_send_cb(
        response: Result<reqwest::Response, reqwest::Error>,
        cb: &SendMessageCallback,
    ) {
        match response {
            Ok(response) => {
                let code = response.status();
                match response.text().await {
                    Ok(text) => cb(Ok(SendMessageCallbackData {
                        http_status: code.as_u16(),
                        text,
                    })),
                    Err(err) => cb(Err(anyhow!("{err}"))),
                }
            }
            Err(err) => cb(Err(anyhow!("{err}"))),
        }
    }
}
