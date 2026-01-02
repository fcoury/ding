use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Context {
    pub tmux: Option<TmuxContext>,
    pub terminal: Option<TerminalContext>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TmuxContext {
    pub session: String,
    pub window: String,
    pub pane: String,
    pub client: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalContext {
    pub app: Option<String>,
}

pub fn detect_context() -> Option<Context> {
    let tmux = detect_tmux();
    let terminal = detect_terminal();
    if tmux.is_none() && terminal.is_none() {
        return None;
    }
    Some(Context { tmux, terminal })
}

fn detect_tmux() -> Option<TmuxContext> {
    if std::env::var("TMUX").ok().is_none() {
        return None;
    }

    let output = Command::new("tmux")
        .args([
            "display-message",
            "-p",
            "#{session_name}\t#{window_id}\t#{pane_id}\t#{client_name}",
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parts: Vec<&str> = stdout.trim().split('\t').collect();
    if parts.len() < 3 {
        return None;
    }

    Some(TmuxContext {
        session: parts[0].to_string(),
        window: parts[1].to_string(),
        pane: parts[2].to_string(),
        client: parts.get(3).map(|s| s.to_string()).filter(|s| !s.is_empty()),
    })
}

fn detect_terminal() -> Option<TerminalContext> {
    let app = std::env::var("TERM_PROGRAM").ok()?;
    Some(TerminalContext { app: Some(app) })
}
