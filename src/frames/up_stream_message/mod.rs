mod callback_message;

pub use callback_message::{
    At as CallbackWebhookMessageAt, WebhookMessage as CallbackWebhookMessage,
};
use serde::{Deserialize, Serialize};

mod robot_message;
pub use robot_message::RobotBatchMessage;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "msgtype")]
pub enum MessageContent {
    #[serde(rename = "text")]
    Text { text: MessageContentText },
    #[serde(rename = "markdown")]
    Markdown { markdown: MessageContentMarkdown },
    #[serde(rename = "link")]
    Link { link: MessageContentLink },
}

impl<T: Into<String>> From<T> for MessageContent {
    fn from(value: T) -> Self {
        Self::Text {
            text: MessageContentText::from(value),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageContentText {
    pub content: String,
}

impl<T: Into<String>> From<T> for MessageContentText {
    fn from(value: T) -> Self {
        Self {
            content: value.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageContentMarkdown {
    pub title: String,
    pub text: String,
}

impl<Title: Into<String>, Text: Into<String>> From<(Title, Text)> for MessageContentMarkdown {
    fn from((title, text): (Title, Text)) -> Self {
        Self {
            title: title.into(),
            text: text.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageContentLink {
    pub title: String,
    pub text: String,
    #[serde(rename = "messageUrl")]
    pub message_url: Option<String>,
    #[serde(rename = "picUrl")]
    pub pic_url: Option<String>,
}
