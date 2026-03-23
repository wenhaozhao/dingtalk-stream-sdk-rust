/*

*/
use crate::frames::{DingTalkPrivateConversationId, DingTalkUserId};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
#[derive(Serialize, Deserialize)]
pub enum RobotMessage {
    Private(RobotPrivateMessage),
    Group(RobotGroupMessage),
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RobotPrivateMessage {
    pub user_ids: Vec<DingTalkUserId>,
    pub content: super::MessageContent,
    #[serde(skip)]
    pub send_result_cb:
        Option<Arc<dyn Fn(Result<(u16, String), anyhow::Error>) + Send + Sync + 'static>>,
}

impl From<RobotPrivateMessage> for RobotMessage {
    fn from(value: RobotPrivateMessage) -> Self {
        Self::Private(value)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RobotGroupMessage {
    pub group_id: DingTalkPrivateConversationId,
    pub content: super::MessageContent,
    #[serde(skip)]
    pub send_result_cb:
        Option<Arc<dyn Fn(Result<(u16, String), anyhow::Error>) + Send + Sync + 'static>>,
}

impl From<RobotGroupMessage> for RobotMessage {
    fn from(value: RobotGroupMessage) -> Self {
        Self::Group(value)
    }
}
