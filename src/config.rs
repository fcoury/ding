use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    pub default_provider: Option<String>,
    pub macos: Option<MacosConfig>,
    pub sources: Option<BTreeMap<String, SourceConfig>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MacosConfig {
    pub sound: Option<String>,
    pub app_bundle_id: Option<String>,
    pub icon: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SourceConfig {
    pub icon: Option<PathBuf>,
    pub app_bundle_id: Option<String>,
    pub display_name: Option<String>,
}

impl Config {
    pub fn template() -> &'static str {
        r#"# wakedev config
# default_provider = "macos"

[macos]
# sound = "default" # use "none" to disable
# app_bundle_id = "com.apple.Terminal"
# icon = "/path/to/icon.png"

[sources.claude]
# icon = "/path/to/claude.icns"
# app_bundle_id = "com.apple.Terminal"

[sources.codex]
# icon = "/path/to/openai.icns"
"#
    }
}
