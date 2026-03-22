/*

*/
use crate::frames::DingTalkUserId;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct RobotBatchMessage {
    pub user_ids: Vec<DingTalkUserId>,
    pub content: super::MessageContent,
    #[serde(skip)]
    pub send_result_cb:
        Option<Box<dyn Fn(Result<(u16, String), anyhow::Error>) + Send + Sync + 'static>>,
}
