use crate::{ClientDownStream, Headers, MsgType};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;

/// Callback message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallbackMessage {
    #[serde(rename = "specVersion")]
    pub spec_version: Option<String>,
    #[serde(rename = "type")]
    pub msg_type: MsgType,
    pub headers: Headers,
    pub data: Option<serde_json::Value>,
    #[serde(flatten)]
    pub extensions: HashMap<String, serde_json::Value>,
}

impl CallbackMessage {
    pub fn new() -> Self {
        Self {
            spec_version: None,
            msg_type: MsgType::Callback,
            headers: Headers::new(),
            data: None,
            extensions: HashMap::new(),
        }
    }

    pub fn from_stream(msg: ClientDownStream) -> Self {
        Self {
            spec_version: msg.spec_version,
            msg_type: msg.msg_type,
            headers: msg.headers,
            data: msg.data.as_ref().and_then(|d| serde_json::from_str(d).ok()),
            extensions: msg.extensions,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallbackMessageData {
    #[serde(rename = "msgId")]
    pub msg_id: String,
    #[serde(rename = "chatbotCorpId")]
    pub chatbot_corp_id: String,
    #[serde(rename = "chatbotUserId")]
    pub chatbot_user_id: String,
    #[serde(rename = "conversationId")]
    pub conversation_id: String,
    #[serde(rename = "conversationType")]
    pub conversation_type: String,
    #[serde(rename = "createAt")]
    pub create_at: i64,
    #[serde(rename = "isAdmin")]
    pub is_admin: bool,
    #[serde(rename = "openThreadId")]
    pub open_thread_id: String,
    #[serde(rename = "robotCode")]
    pub robot_code: String,
    #[serde(rename = "senderCorpId")]
    pub sender_corp_id: String,
    #[serde(rename = "senderId")]
    pub sender_id: String,
    #[serde(rename = "senderNick")]
    pub sender_nick: String,
    #[serde(rename = "senderPlatform")]
    pub sender_platform: String,
    #[serde(rename = "senderStaffId")]
    pub sender_staff_id: String,
    #[serde(rename = "sessionWebhook")]
    pub session_webhook: String,
    #[serde(rename = "sessionWebhookExpiredTime")]
    pub session_webhook_expired_time: i64,
    #[serde(flatten)]
    pub payload: Payload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "msgtype")]
pub enum Payload {
    #[serde(rename = "text")]
    Text { text: Text },
    #[serde(rename = "picture")]
    Picture { content: Picture },
    #[serde(rename = "file")]
    File { content: File },
    #[serde(rename = "richText")]
    RichText { content: RichText },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Text {
    #[serde(rename = "content", alias = "text")]
    content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Picture {
    #[serde(rename = "downloadCode")]
    download_code: String,
    #[serde(rename = "pictureDownloadCode")]
    picture_download_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct File {
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
pub struct RichText {
    #[serde(rename = "richText")]
    content: Vec<RichTextItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum RichTextItem {
    #[serde(rename = "picture")]
    Picture {
        #[serde(rename = "downloadCode")]
        download_code: String,
        #[serde(rename = "pictureDownloadCode")]
        picture_download_code: String,
    },
    #[serde(rename = "text", alias = "content")]
    Text { text: String },
}

#[cfg(test)]
mod tests {
    use crate::frames::callback_message::{File, RichTextItem};
    use crate::frames::{CallbackMessageData, Payload, Picture, RichText, Text};

    #[test]
    fn test_text_parse() {
        let data: CallbackMessageData = serde_json::from_str(TEXT_JSON).unwrap();
        assert_eq!(data.msg_id, "msgBjXREkdlZkfTfrIiQomjAw==");
        if let Payload::Text {
            text: Text { content },
        } = data.payload
        {
            assert_eq!(content, "hello");
        } else {
            panic!("Expected text payload but got {:?}", data.payload);
        }
    }
    #[test]
    fn test_picture_parse() {
        let data: CallbackMessageData = serde_json::from_str(PICTURE_JSON).unwrap();
        assert_eq!(data.msg_id, "msgmJpewjjmDF5LPJdRs9n/ZA==");
        if let Payload::Picture {
            content: Picture { download_code, .. },
        } = data.payload
        {
            assert!(download_code.starts_with("mIofN681YE3f/+m+NntqpSkhBVXbzJynU"));
        } else {
            panic!("Expected picture payload but got {:?}", data.payload);
        }
    }

    #[test]
    fn test_file_parse() {
        let data: CallbackMessageData = serde_json::from_str(FILE_JSON).unwrap();
        assert_eq!(data.msg_id, "msgBCO626EXCHXfZoDioTCPxg==");
        if let Payload::File {
            content: File { file_id, .. },
        } = data.payload
        {
            assert!(file_id.eq_ignore_ascii_case("214980176385"));
        } else {
            panic!("Expected picture payload but got {:?}", data.payload);
        }
    }

    #[test]
    fn test_rich_text_parse() {
        let data: CallbackMessageData = serde_json::from_str(RICH_TEXT_JSON).unwrap();
        assert_eq!(data.msg_id, "msgGDkZWYZlvw7rFtTHcDIFWw==");
        if let Payload::RichText {
            content: RichText { content: rich_text },
            ..
        } = &data.payload
        {
            assert!(rich_text.len() > 0);
            if let RichTextItem::Picture { download_code, .. } = rich_text.get(0).unwrap() {
                assert!(download_code
                    .starts_with("mIofN681YE3f/+m+NntqpeLZQiMFIZMEPWAhjFjD1g5L/SdG/3lCmLWzq"));
            } else {
                panic!("Expected picture payload but got {:?}", data.payload);
            }
        } else {
            panic!("Expected picture payload but got {:?}", data.payload);
        }
    }

    const TEXT_JSON: &str = include_str!("../../test_resources/cb_msg_text.json");
    const PICTURE_JSON: &str = include_str!("../../test_resources/cb_msg_picture.json");
    const FILE_JSON: &str = include_str!("../../test_resources/cb_msg_file.json");
    const RICH_TEXT_JSON: &str = include_str!("../../test_resources/cb_msg_rich_text.json");
}
