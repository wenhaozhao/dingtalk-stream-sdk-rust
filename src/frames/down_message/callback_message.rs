use crate::frames::{DingTalkGroupConversationId, DingTalkPrivateConversationId, DingTalkUserId};
use anyhow::anyhow;
use chrono::{TimeZone, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::ops::Deref;
use std::str::FromStr;
use std::time::Duration;
use crate::frames::down_message::{DownStreamMessage, MessageHeaders};

/// Callback message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallbackMessage {
    #[serde(rename = "specVersion")]
    pub spec_version: Option<String>,
    #[serde(rename = "type")]
    pub headers: MessageHeaders,
    pub data: Option<MessageData>,
    #[serde(flatten)]
    pub extensions: HashMap<String, serde_json::Value>,
}

impl TryFrom<DownStreamMessage> for CallbackMessage {
    type Error = anyhow::Error;

    fn try_from(
        DownStreamMessage {
            spec_version,
            headers,
            r#type,
            data,
            extensions,
        }: DownStreamMessage,
    ) -> crate::Result<Self> {
        if let super::MessageType::Callback = r#type {
            Ok(Self {
                spec_version,
                headers,
                data: if let Some(data) = data {
                    serde_json::from_str(&data)?
                } else {
                    None
                },
                extensions,
            })
        } else {
            Err(anyhow!("expected callback message"))
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageData {
    #[serde(rename = "msgId")]
    pub msg_id: String,
    #[serde(flatten)]
    pub conversation: Conversation,
    #[serde(flatten)]
    pub sender: MessageSender,
    #[serde(flatten)]
    pub session_webhook: Option<SessionWebhook>,
    #[serde(flatten)]
    pub chatbot: Chatbot,
    #[serde(rename = "isAdmin")]
    pub is_admin: Option<bool>,
    #[serde(rename = "openThreadId")]
    pub open_thread_id: Option<String>,
    #[serde(rename = "senderPlatform")]
    pub sender_platform: Option<String>,
    #[serde(flatten)]
    pub payload: Option<MessagePayload>,
    #[serde(rename = "atUsers")]
    pub at_users: Option<Vec<AtUser>>,
    #[serde(rename = "isInAtList")]
    pub is_in_at_list: Option<bool>,
    #[serde(rename = "createAt")]
    pub create_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "conversationType")]
pub enum Conversation {
    #[serde(rename = "1")]
    Private {
        #[serde(rename = "conversationId")]
        id: DingTalkPrivateConversationId,
    },
    #[serde(rename = "2")]
    Group {
        #[serde(rename = "conversationId")]
        id: DingTalkGroupConversationId,
        #[serde(rename = "conversationTitle")]
        title: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageSender {
    #[serde(rename = "senderId")]
    pub sender_id: String,
    #[serde(rename = "senderNick")]
    pub sender_nick: String,
    #[serde(rename = "senderCorpId")]
    pub sender_corp_id: Option<String>,
    #[serde(rename = "senderStaffId")]
    pub sender_staff_id: Option<DingTalkUserId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionWebhook {
    #[serde(rename = "sessionWebhook")]
    url: String,
    #[serde(rename = "sessionWebhookExpiredTime")]
    expired_time: i64,
}

impl SessionWebhook {
    pub fn webhook_url(&self) -> crate::Result<url::Url> {
        Ok(url::Url::from_str(&self.url)?)
    }

    pub fn timeout(&self) -> Option<Duration> {
        if let chrono::LocalResult::Single(expired_time) =
            Utc.timestamp_millis_opt(self.expired_time)
        {
            let now = Utc::now();
            if expired_time > now {
                if let Ok(duration) = (expired_time - now).to_std() {
                    return Some(duration);
                }
            }
        }
        None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chatbot {
    #[serde(rename = "chatbotCorpId")]
    pub chatbot_corp_id: Option<String>,
    #[serde(rename = "chatbotUserId")]
    pub chatbot_user_id: String,
    #[serde(rename = "robotCode")]
    pub robot_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtUser {
    #[serde(rename = "dingtalkId")]
    pub dingtalk_id: Option<String>,
    #[serde(rename = "staffId")]
    pub staff_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "msgtype")]
pub enum MessagePayload {
    #[serde(rename = "text")]
    Text { text: PayloadText },
    #[serde(rename = "picture")]
    Picture { content: PayloadPicture },
    #[serde(rename = "file")]
    File { content: PayloadFile },
    #[serde(rename = "richText")]
    RichText { content: PayloadRichText },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayloadText {
    #[serde(rename = "content", alias = "text")]
    pub content: String,
}

impl Display for PayloadText {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.content)
    }
}

impl Deref for PayloadText {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.content
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayloadPicture {
    #[serde(rename = "downloadCode")]
    pub download_code: String,
    #[serde(rename = "pictureDownloadCode")]
    pub picture_download_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayloadFile {
    #[serde(rename = "downloadCode")]
    pub download_code: String,
    #[serde(rename = "fileId")]
    pub file_id: String,
    #[serde(rename = "fileName")]
    pub file_name: String,
    #[serde(rename = "spaceId")]
    pub space_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayloadRichText {
    #[serde(rename = "richText")]
    pub content: Vec<RichTextItem>,
}

impl Deref for PayloadRichText {
    type Target = [RichTextItem];

    fn deref(&self) -> &Self::Target {
        &self.content
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RichTextItem {
    #[serde(rename = "picture")]
    Picture(PayloadPicture),
    #[serde(rename = "text", alias = "content")]
    Text(PayloadText),
}

#[cfg(test)]
mod tests {
    use super::{
        MessageData, MessagePayload, PayloadFile, PayloadPicture, PayloadRichText, PayloadText,
        RichTextItem,
    };

    #[test]
    fn test_text_parse() {
        let data: MessageData = serde_json::from_str(TEXT_JSON).unwrap();
        assert_eq!(data.msg_id.as_str(), "msgBjXREkdlZkfTfrIiQomjAw==");
        if let Some(MessagePayload::Text {
            text: PayloadText { content },
        }) = data.payload
        {
            assert_eq!(content, "hello");
        } else {
            panic!("Expected text payload but got {:?}", data.payload);
        }
    }
    #[test]
    fn test_picture_parse() {
        let data: MessageData = serde_json::from_str(PICTURE_JSON).unwrap();
        assert_eq!(data.msg_id.as_str(), "msgmJpewjjmDF5LPJdRs9n/ZA==");
        if let Some(MessagePayload::Picture {
            content: PayloadPicture { download_code, .. },
        }) = data.payload
        {
            assert!(download_code.starts_with("mIofN681YE3f/+m+NntqpSkhBVXbzJynU"));
        } else {
            panic!("Expected picture payload but got {:?}", data.payload);
        }
    }

    #[test]
    fn test_file_parse() {
        let data: MessageData = serde_json::from_str(FILE_JSON).unwrap();
        assert_eq!(data.msg_id.as_str(), "msgBCO626EXCHXfZoDioTCPxg==");
        if let Some(MessagePayload::File {
            content: PayloadFile { file_id, .. },
        }) = data.payload
        {
            assert!(file_id.eq_ignore_ascii_case("214980176385"));
        } else {
            panic!("Expected picture payload but got {:?}", data.payload);
        }
    }

    #[test]
    fn test_rich_text_parse() {
        let data: MessageData = serde_json::from_str(RICH_TEXT_JSON).unwrap();
        assert_eq!(data.msg_id.as_str(), "msgGDkZWYZlvw7rFtTHcDIFWw==");
        if let Some(MessagePayload::RichText {
            content: PayloadRichText { content: rich_text },
            ..
        }) = &data.payload
        {
            assert!(rich_text.len() > 0);
            if let RichTextItem::Picture(PayloadPicture { download_code, .. }) =
                rich_text.get(0).unwrap()
            {
                assert!(download_code
                    .starts_with("mIofN681YE3f/+m+NntqpeLZQiMFIZMEPWAhjFjD1g5L/SdG/3lCmLWzq"));
            } else {
                panic!("Expected picture payload but got {:?}", data.payload);
            }
            if let RichTextItem::Text(PayloadText { content }) = rich_text.get(2).unwrap() {
                assert!(content.eq("abc"));
            } else {
                panic!("Expected text payload but got {:?}", data.payload);
            }
        } else {
            panic!("Expected picture payload but got {:?}", data.payload);
        }
    }

    const TEXT_JSON: &str = include_str!("../../../test_resources/cb_msg_text.json");
    const PICTURE_JSON: &str = include_str!("../../../test_resources/cb_msg_picture.json");
    const FILE_JSON: &str = include_str!("../../../test_resources/cb_msg_file.json");
    const RICH_TEXT_JSON: &str = include_str!("../../../test_resources/cb_msg_rich_text.json");
}
