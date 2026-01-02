use crate::config::MacosConfig;
use crate::notification::{Notification, Urgency};
use crate::provider::{DeliveryOutcome, DeliveryReport, Provider, ProviderError, SendOptions};

#[cfg(target_os = "macos")]
use mac_notification_sys::error::{ApplicationError, Error as MacError};
#[cfg(target_os = "macos")]
use mac_notification_sys::{
    set_application, Notification as MacNotification, NotificationResponse, Sound,
};

#[cfg(target_os = "macos")]
#[derive(Debug, Clone, Default)]
pub struct MacosProvider {
    config: MacosConfig,
}

#[cfg(target_os = "macos")]
impl MacosProvider {
    pub fn new(config: Option<MacosConfig>) -> Result<Self, ProviderError> {
        let provider = Self {
            config: config.unwrap_or_default(),
        };

        if let Some(bundle) = provider.config.app_bundle_id.as_deref() {
            if let Err(err) = set_application(bundle) {
                if !matches!(err, MacError::Application(ApplicationError::AlreadySet(_))) {
                    return Err(ProviderError::Message(err.to_string()));
                }
            }
        }

        Ok(provider)
    }
}

#[cfg(target_os = "macos")]
impl Provider for MacosProvider {
    fn name(&self) -> &'static str {
        "macos"
    }

    fn send(
        &self,
        notification: &Notification,
        options: SendOptions,
    ) -> Result<DeliveryReport, ProviderError> {
        let mut mac = MacNotification::new();
        mac.title(&notification.title).message(&notification.message);

        if let Some(tag) = notification.tag.as_deref() {
            mac.subtitle(tag);
        }

        if let Some(sound) = self.config.sound.as_deref() {
            if sound.eq_ignore_ascii_case("default") {
                mac.default_sound();
            } else {
                mac.sound(Sound::from(sound));
            }
        } else if matches!(notification.urgency, Some(Urgency::High)) {
            mac.default_sound();
        }

        let icon_path = notification
            .icon
            .as_ref()
            .or(self.config.icon.as_ref());
        let icon_string = icon_path.map(|icon| icon.to_string_lossy().into_owned());
        if let Some(path) = icon_string.as_deref() {
            mac.app_icon(path);
        }

        mac.wait_for_click(options.wait_for_click);
        if options.wait_for_click {
            mac.asynchronous(false);
        } else {
            mac.asynchronous(true);
        }

        let response = mac
            .send()
            .map_err(|err| ProviderError::Message(err.to_string()))?;
        let outcome = map_response(response, options.wait_for_click);

        Ok(DeliveryReport {
            provider: self.name(),
            id: None,
            outcome,
        })
    }
}

#[cfg(target_os = "macos")]
fn map_response(
    response: NotificationResponse,
    waited: bool,
) -> Option<DeliveryOutcome> {
    if !waited {
        return None;
    }
    let outcome = match response {
        NotificationResponse::None => DeliveryOutcome::Delivered,
        NotificationResponse::Click => DeliveryOutcome::Clicked,
        NotificationResponse::ActionButton(label) => DeliveryOutcome::ActionButton(label),
        NotificationResponse::CloseButton(label) => DeliveryOutcome::Closed(label),
        NotificationResponse::Reply(text) => DeliveryOutcome::Replied(text),
    };
    Some(outcome)
}

#[cfg(not(target_os = "macos"))]
#[derive(Debug, Clone, Default)]
pub struct MacosProvider;

#[cfg(not(target_os = "macos"))]
impl MacosProvider {
    pub fn new(_config: Option<MacosConfig>) -> Result<Self, ProviderError> {
        Err(ProviderError::Unsupported)
    }
}

#[cfg(not(target_os = "macos"))]
impl Provider for MacosProvider {
    fn name(&self) -> &'static str {
        "macos"
    }

    fn send(
        &self,
        _notification: &Notification,
        _options: SendOptions,
    ) -> Result<DeliveryReport, ProviderError> {
        Err(ProviderError::Unsupported)
    }
}
