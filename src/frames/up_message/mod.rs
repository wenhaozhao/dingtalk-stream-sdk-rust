pub mod callback_message;
use serde::{Deserialize, Serialize};

pub mod robot_message;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "msgtype")]
pub enum MessageContent {
    #[serde(rename = "text")]
    Text { text: MessageContentText },
    #[serde(rename = "picture")]
    Picture { picture: MessageContentPicture },
    #[serde(rename = "markdown")]
    Markdown { markdown: MessageContentMarkdown },
    #[serde(rename = "link")]
    Link { link: MessageContentLink },
}

impl MessageContent {
    pub(crate) fn to_up_msg(&self) -> crate::Result<(&'static str, String)> {
        Ok(match self {
            MessageContent::Text { text } => ("sampleText", serde_json::to_string(text)?),
            MessageContent::Picture { picture } => {
                ("sampleImageMsg", serde_json::to_string(picture)?)
            }
            MessageContent::Markdown { markdown } => {
                ("sampleMarkdown", serde_json::to_string(markdown)?)
            }
            MessageContent::Link { .. } => ("sampleLink", serde_json::to_string(&())?),
        })
    }
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

impl From<MessageContentText> for MessageContent {
    fn from(value: MessageContentText) -> Self {
        Self::Text { text: value }
    }
}

impl<T: Into<String>> From<T> for MessageContentText {
    fn from(value: T) -> Self {
        Self {
            content: value.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageContentPicture {
    #[serde(rename = "photoURL")]
    photo_url: String,
}

impl From<MessageContentPicture> for MessageContent {
    fn from(value: MessageContentPicture) -> Self {
        Self::Picture { picture: value }
    }
}

impl<T: Into<String>> From<T> for MessageContentPicture {
    fn from(value: T) -> Self {
        Self {
            photo_url: value.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageContentMarkdown {
    pub title: String,
    pub text: String,
}

impl From<MessageContentMarkdown> for MessageContent {
    fn from(value: MessageContentMarkdown) -> Self {
        Self::Markdown { markdown: value }
    }
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

impl From<MessageContentLink> for MessageContent {
    fn from(value: MessageContentLink) -> Self {
        Self::Link { link: value }
    }
}
