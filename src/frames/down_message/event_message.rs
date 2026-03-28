use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::frames::down_message::{DownStreamMessage, MessageHeaders};

/// Event message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMessage {
    #[serde(rename = "specVersion")]
    pub spec_version: Option<String>,
    #[serde(rename = "type")]
    pub headers: MessageHeaders,
    pub data: Option<serde_json::Value>,
    #[serde(flatten)]
    pub extensions: HashMap<String, serde_json::Value>,
}

impl TryFrom<DownStreamMessage> for EventMessage {
    type Error = anyhow::Error;

    fn try_from(
        DownStreamMessage {
            spec_version,
            headers,
            r#type,
            data,
            extensions,
        }: DownStreamMessage,
    ) -> crate::Result<Self> {
        if let super::MessageType::Event = r#type {
            Ok(Self {
                spec_version,
                headers,
                data: if let Some(data) = data {
                    serde_json::from_str(&data)?
                } else {
                    None
                },
                extensions,
            })
        } else {
            Err(anyhow!("expected event message"))
        }
    }
}
