use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct WebhookMessage {
    #[serde(flatten)]
    pub content: super::MessageContent,
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
