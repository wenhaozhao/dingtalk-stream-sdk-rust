use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct WebhookMessage {
    #[serde(flatten)]
    pub content: Content,
    #[serde(rename = "at")]
    pub at: At,
    #[serde(skip)]
    pub send_result_cb:
        Option<Box<dyn Fn(Result<(u16, String), anyhow::Error>) + Send + Sync + 'static>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct At {
    #[serde(rename = "isAtAll")]
    pub at_all: bool,
    #[serde(rename = "atUserIds")]
    pub at_user_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "msgtype")]
pub enum Content {
    #[serde(rename = "text")]
    Text { text: Text },
    #[serde(rename = "markdown")]
    Markdown { markdown: Markdown },
    #[serde(rename = "link")]
    Link { link: Link },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Text {
    pub content: String,
}

impl From<String> for Text {
    fn from(s: String) -> Self {
        Text { content: s }
    }
}
impl From<&str> for Text {
    fn from(value: &str) -> Self {
        value.to_string().into()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Markdown {
    pub title: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Link {
    pub title: String,
    pub text: String,
    #[serde(rename = "messageUrl")]
    pub message_url: Option<String>,
    #[serde(rename = "picUrl")]
    pub pic_url: Option<String>,
}
