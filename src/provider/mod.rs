use crate::notification::Notification;

pub mod macos;
pub mod telegram;

#[derive(Debug, Clone, Copy, Default)]
pub struct SendOptions {
    pub wait_for_click: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum DeliveryOutcome {
    Delivered,
    Clicked,
    ActionButton(String),
    Closed(String),
    Replied(String),
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DeliveryReport {
    pub provider: &'static str,
    pub id: Option<String>,
    pub outcome: Option<DeliveryOutcome>,
}

#[allow(dead_code)]
#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("provider not available on this platform")]
    Unsupported,
    #[error("provider error: {0}")]
    Message(String),
}

pub trait Provider {
    fn name(&self) -> &'static str;
    fn send(
        &self,
        notification: &Notification,
        options: SendOptions,
    ) -> Result<DeliveryReport, ProviderError>;
}
