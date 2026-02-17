use crate::config::MacosConfig;
use crate::notification::Notification;
use crate::provider::{DeliveryOutcome, DeliveryReport, Provider, ProviderError, SendOptions};

#[cfg(target_os = "macos")]
use std::io::Write;
#[cfg(target_os = "macos")]
use std::path::PathBuf;
#[cfg(target_os = "macos")]
use std::process::{Command, Stdio};

// Embedded notifier binary compiled from objc/notifier.m by build.rs.
#[cfg(target_os = "macos")]
pub const NOTIFIER_BINARY: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/ding-notifier"));

#[cfg(target_os = "macos")]
#[derive(Debug, Clone, Default)]
pub struct MacosProvider {
    config: MacosConfig,
}

#[cfg(target_os = "macos")]
impl MacosProvider {
    pub fn new(config: Option<MacosConfig>) -> Result<Self, ProviderError> {
        Ok(Self {
            config: config.unwrap_or_default(),
        })
    }

    // Derive the helper binary path from the app_bundle_id.
    // com.ding.claude -> ~/.cache/ding/apps/claude.app/Contents/MacOS/ding-helper
    fn helper_path(&self) -> Result<PathBuf, ProviderError> {
        let bundle_id = self
            .config
            .app_bundle_id
            .as_deref()
            .ok_or_else(|| ProviderError::Message("no app_bundle_id configured".into()))?;

        // Extract the app name from the bundle id (last component after "com.ding.").
        let app_name = bundle_id
            .strip_prefix("com.ding.")
            .unwrap_or(bundle_id)
            .replace('.', "-");

        let base_dir = std::env::var("XDG_CACHE_HOME")
            .map(PathBuf::from)
            .or_else(|_| std::env::var("HOME").map(|h| PathBuf::from(h).join(".cache")))
            .unwrap_or_else(|_| std::env::temp_dir());

        let path = base_dir
            .join("ding/apps")
            .join(format!("{}.app", app_name))
            .join("Contents/MacOS/ding-helper");

        if !path.exists() {
            return Err(ProviderError::Message(format!(
                "helper binary not found at {}",
                path.display()
            )));
        }

        Ok(path)
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
        let helper = self.helper_path()?;

        // Build the JSON request for the notifier helper.
        let sound_value = notification
            .sound
            .as_deref()
            .or(self.config.sound.as_deref())
            .unwrap_or("default");

        let mut req = serde_json::Map::new();
        req.insert(
            "title".into(),
            serde_json::Value::String(notification.title.clone()),
        );
        req.insert(
            "message".into(),
            serde_json::Value::String(notification.message.clone()),
        );
        if let Some(tag) = notification.tag.as_deref() {
            req.insert("subtitle".into(), serde_json::Value::String(tag.into()));
        }
        req.insert(
            "sound".into(),
            serde_json::Value::String(sound_value.into()),
        );
        req.insert(
            "wait_for_click".into(),
            serde_json::Value::Bool(options.wait_for_click),
        );

        let json_input = serde_json::to_string(&req)
            .map_err(|e| ProviderError::Message(format!("failed to serialize request: {e}")))?;

        // Spawn the helper and communicate via stdin/stdout.
        let mut child = Command::new(&helper)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                ProviderError::Message(format!(
                    "failed to spawn {}: {e}",
                    helper.display()
                ))
            })?;

        // Write JSON to stdin and close it.
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(json_input.as_bytes()).map_err(|e| {
                ProviderError::Message(format!("failed to write to helper stdin: {e}"))
            })?;
        }

        // Wait for the helper to finish and read stdout.
        let output = child.wait_with_output().map_err(|e| {
            ProviderError::Message(format!("failed to wait for helper: {e}"))
        })?;

        if output.stdout.is_empty() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ProviderError::Message(format!(
                "helper produced no output (exit={}, stderr={})",
                output.status,
                stderr.trim()
            )));
        }

        // Parse the JSON response.
        let resp: serde_json::Value =
            serde_json::from_slice(&output.stdout).map_err(|e| {
                ProviderError::Message(format!(
                    "failed to parse helper response: {e} (raw: {})",
                    String::from_utf8_lossy(&output.stdout)
                ))
            })?;

        let status = resp["status"].as_str().unwrap_or("error");
        if status == "error" {
            let err_msg = resp["error"]
                .as_str()
                .unwrap_or("unknown error from helper");
            return Err(ProviderError::Message(err_msg.to_string()));
        }

        // Map the helper's status to a DeliveryOutcome.
        let outcome = if options.wait_for_click {
            Some(match status {
                "clicked" => DeliveryOutcome::Clicked,
                "closed" => DeliveryOutcome::Closed(String::new()),
                _ => DeliveryOutcome::Delivered,
            })
        } else {
            None
        };

        Ok(DeliveryReport {
            provider: self.name(),
            id: None,
            outcome,
        })
    }
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
