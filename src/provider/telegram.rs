use crate::config::TelegramConfig;
use crate::notification::Notification;
use crate::provider::{DeliveryOutcome, DeliveryReport, Provider, ProviderError, SendOptions};
use serde::Deserialize;

#[derive(Debug, Clone, Default)]
pub struct TelegramProvider {
    config: TelegramConfig,
}

impl TelegramProvider {
    pub fn new(config: TelegramConfig) -> Result<Self, ProviderError> {
        if config.bot_token.as_deref().unwrap_or("").is_empty() {
            return Err(ProviderError::Message(
                "telegram bot_token is not configured".to_string(),
            ));
        }
        if config.chat_id.as_deref().unwrap_or("").is_empty() {
            return Err(ProviderError::Message(
                "telegram chat_id is not configured".to_string(),
            ));
        }
        Ok(Self { config })
    }
}

impl Provider for TelegramProvider {
    fn name(&self) -> &'static str {
        "telegram"
    }

    fn send(
        &self,
        notification: &Notification,
        _options: SendOptions,
    ) -> Result<DeliveryReport, ProviderError> {
        let token = self
            .config
            .bot_token
            .as_deref()
            .unwrap_or_default();
        let chat_id = self
            .config
            .chat_id
            .as_deref()
            .unwrap_or_default();
        let parse_mode = self.config.parse_mode.as_deref();
        let silent = self.config.silent.unwrap_or(false);

        let text = build_text(notification, parse_mode)?;
        if text.is_empty() {
            return Err(ProviderError::Message(
                "telegram text is empty".to_string(),
            ));
        }

        let url = format!("https://api.telegram.org/bot{token}/sendMessage");
        let mut payload = serde_json::json!({
            "chat_id": chat_id,
            "text": text,
        });

        if let Some(mode) = parse_mode {
            if !mode.trim().is_empty() {
                payload["parse_mode"] = serde_json::Value::String(mode.to_string());
            }
        }
        if silent {
            payload["disable_notification"] = serde_json::Value::Bool(true);
        }

        let response = ureq::post(&url)
            .set("Content-Type", "application/json")
            .send_json(payload);

        match response {
            Ok(res) => {
                let parsed: TelegramResponse = res
                    .into_json()
                    .map_err(|err| ProviderError::Message(err.to_string()))?;
                if !parsed.ok {
                    let desc = parsed
                        .description
                        .unwrap_or_else(|| "telegram error".to_string());
                    return Err(ProviderError::Message(desc));
                }
                let id = parsed
                    .result
                    .as_ref()
                    .map(|r| r.message_id.to_string());
                Ok(DeliveryReport {
                    provider: self.name(),
                    id,
                    outcome: Some(DeliveryOutcome::Delivered),
                })
            }
            Err(ureq::Error::Status(code, res)) => {
                let desc = res
                    .into_json::<TelegramResponse>()
                    .ok()
                    .and_then(|r| r.description)
                    .unwrap_or_else(|| format!("telegram error status {code}"));
                Err(ProviderError::Message(desc))
            }
            Err(err) => Err(ProviderError::Message(err.to_string())),
        }
    }
}

#[derive(Debug, Deserialize)]
struct TelegramResponse {
    ok: bool,
    result: Option<TelegramMessage>,
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TelegramMessage {
    message_id: i64,
}

fn build_text(
    notification: &Notification,
    parse_mode: Option<&str>,
) -> Result<String, ProviderError> {
    let mut title = notification.title.clone();
    let mut message = notification.message.clone();
    let mut link = notification.link.clone().unwrap_or_default();

    let use_markdown = matches!(
        parse_mode,
        Some(mode) if mode.eq_ignore_ascii_case("markdownv2")
    );

    if use_markdown {
        title = escape_markdown_v2(&title);
        message = escape_markdown_v2(&message);
        if !link.is_empty() {
            link = escape_markdown_v2(&link);
        }
    }

    let mut parts = Vec::new();
    if !title.trim().is_empty() {
        parts.push(title);
    }
    if !message.trim().is_empty() {
        parts.push(message);
    }
    if !link.trim().is_empty() {
        parts.push(link);
    }

    let mut text = parts.join("\n");
    if text.len() > 4096 {
        let suffix = "...";
        let max = 4096usize.saturating_sub(suffix.len());
        text = format!("{}{}", text.chars().take(max).collect::<String>(), suffix);
    }
    Ok(text)
}

fn escape_markdown_v2(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '_' | '*' | '[' | ']' | '(' | ')' | '~' | '`' | '>' | '#' | '+' | '-' | '=' | '|' | '{'
            | '}' | '.' | '!' => {
                out.push('\\');
                out.push(ch);
            }
            _ => out.push(ch),
        }
    }
    out
}
