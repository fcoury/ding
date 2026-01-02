use crate::config::MacosConfig;
use crate::context::Context;
use crate::notification::Notification;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaitPayload {
    pub notification: Notification,
    pub macos: Option<MacosConfig>,
    pub on_click: Option<String>,
    pub context: Option<Context>,
}
