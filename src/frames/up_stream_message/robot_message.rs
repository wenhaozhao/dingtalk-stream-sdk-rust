/*

*/
use crate::frames::{DingTalkGroupConversationId, DingTalkUserId, OptionSendMessageCallback};
use serde::{Deserialize, Serialize};
#[derive(Clone, Serialize, Deserialize)]
pub enum RobotMessage {
    Private {
        message: RobotPrivateMessage,
        #[serde(skip)]
        send_result_cb: OptionSendMessageCallback,
    },
    Group {
        message: RobotGroupMessage,
        #[serde(skip)]
        send_result_cb: OptionSendMessageCallback,
    },
}

impl RobotMessage {
    pub fn with_cb<T: Into<OptionSendMessageCallback>>(self, cb: T) -> Self {
        match self {
            RobotMessage::Private { message, .. } => Self::Private {
                message,
                send_result_cb: cb.into(),
            },
            RobotMessage::Group { message, .. } => Self::Group {
                message,
                send_result_cb: cb.into(),
            },
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RobotPrivateMessage {
    pub user_ids: Vec<DingTalkUserId>,
    pub content: super::MessageContent,
}

impl From<RobotPrivateMessage> for RobotMessage {
    fn from(message: RobotPrivateMessage) -> Self {
        Self::Private {
            message,
            send_result_cb: Default::default(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RobotGroupMessage {
    pub group_id: DingTalkGroupConversationId,
    pub content: super::MessageContent,
}

impl From<RobotGroupMessage> for RobotMessage {
    fn from(message: RobotGroupMessage) -> Self {
        Self::Group {
            message,
            send_result_cb: Default::default(),
        }
    }
}
