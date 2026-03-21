use crate::frames::CallbackMessageData;
use serde::{Deserialize, Serialize};

/// Robot message (chatbot message)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RobotMessage(CallbackMessageData);

/// At user information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtUser {
    #[serde(rename = "dingtalkId")]
    pub dingtalk_id: Option<String>,
    #[serde(rename = "staffId")]
    pub staff_id: Option<String>,
}

impl From<CallbackMessageData> for RobotMessage {
    fn from(value: CallbackMessageData) -> Self {
        RobotMessage(value)
    }
}

impl RobotMessage {
    /// Get text content from the message
    pub fn get_text(&self) -> Option<String> {
        if let Some(text) = &self.text {
            return Some(text.content.clone());
        }
        None
    }

    /// Get text list from the message (supports rich text)
    pub fn get_text_list(&self) -> Vec<String> {
        let mut result = Vec::new();

        if let Some(text) = &self.text {
            result.push(text.content.clone());
        } else if let Some(content) = &self.content {
            if let Some(rich_text) = content.get("richText").and_then(|v| v.as_array()) {
                for item in rich_text {
                    if let Some(text) = item.get("text").and_then(|v| v.as_str()) {
                        result.push(text.to_string());
                    }
                }
            }
        }

        result
    }

    /// Get image download codes from the message
    pub fn get_image_list(&self) -> Vec<String> {
        let mut result = Vec::new();

        if let Some(content) = &self.content {
            if let Some(download_code) = content.get("downloadCode").and_then(|v| v.as_str()) {
                result.push(download_code.to_string());
            } else if let Some(rich_text) = content.get("richText").and_then(|v| v.as_array()) {
                for item in rich_text {
                    if let Some(code) = item.get("downloadCode").and_then(|v| v.as_str()) {
                        result.push(code.to_string());
                    }
                }
            }
        }

        result
    }
}
