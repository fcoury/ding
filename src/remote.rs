use crate::context::{Context, TerminalContext, TmuxContext};
use crate::notification::Notification;
use libc::gethostname;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteEnvelope {
    pub notification: Notification,
    pub context: Option<RemoteContext>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RemoteContext {
    pub origin_host: Option<String>,
    pub origin_user: Option<String>,
    pub cwd: Option<String>,
    pub tmux: Option<TmuxContext>,
    pub terminal: Option<TerminalContext>,
}

impl RemoteContext {
    pub fn from_local(context: Option<Context>) -> Self {
        let origin_host = local_hostname();
        let origin_user = env::var("USER").ok();
        let cwd = env::current_dir()
            .ok()
            .and_then(|p| p.to_str().map(|s| s.to_string()));
        let (tmux, terminal) = if let Some(ctx) = context {
            (ctx.tmux, ctx.terminal)
        } else {
            (None, None)
        };

        Self {
            origin_host,
            origin_user,
            cwd,
            tmux,
            terminal,
        }
    }
}

fn local_hostname() -> Option<String> {
    let mut buf = [0u8; 256];
    let res = unsafe { gethostname(buf.as_mut_ptr() as *mut i8, buf.len()) };
    if res != 0 {
        return None;
    }
    let len = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
    let slice = &buf[..len];
    Some(String::from_utf8_lossy(slice).to_string())
}
