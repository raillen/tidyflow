use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum NotifyEvent {
    Started,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum NotifyChannel {
    Generic {
        url: String,
        #[serde(default)]
        headers: HashMap<String, String>,
    },
    Discord {
        webhook_url: String,
    },
    Telegram {
        bot_token: String,
        chat_id: String,
        #[serde(default)]
        remember_token: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct NotifyConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub events: Vec<NotifyEvent>,
    #[serde(default)]
    pub channels: Vec<NotifyChannel>,
}
