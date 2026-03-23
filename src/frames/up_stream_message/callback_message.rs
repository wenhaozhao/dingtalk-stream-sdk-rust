use crate::frames::DingTalkUserId;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
pub struct WebhookMessage {
    #[serde(flatten)]
    pub content: super::MessageContent,
    #[serde(rename = "at")]
    pub at: At,
    #[serde(skip)]
    pub send_result_cb:
        Option<Arc<dyn Fn(Result<(u16, String), anyhow::Error>) + Send + Sync + 'static>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct At {
    #[serde(rename = "isAtAll")]
    pub at_all: bool,
    #[serde(rename = "atUserIds")]
    pub at_user_ids: Option<Vec<DingTalkUserId>>,
}

impl At {
    pub fn at_all() -> Self {
        Self {
            at_all: true,
            at_user_ids: None,
        }
    }
}

impl From<DingTalkUserId> for At {
    fn from(value: DingTalkUserId) -> Self {
        Self {
            at_all: false,
            at_user_ids: Some(vec![value]),
        }
    }
}

impl From<&DingTalkUserId> for At {
    fn from(value: &DingTalkUserId) -> Self {
        value.clone().into()
    }
}
