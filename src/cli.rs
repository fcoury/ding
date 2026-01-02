use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "wakedev", version, about = "Multi-provider notification CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Path to config file (TOML)
    #[arg(long, global = true)]
    pub config: Option<PathBuf>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Send a notification
    Send(SendArgs),
    /// Manage config
    Config {
        #[command(subcommand)]
        command: ConfigCmd,
    },
    /// List available providers
    Providers {
        #[command(subcommand)]
        command: ProvidersCmd,
    },
    /// Manage configured sources
    Sources {
        #[command(subcommand)]
        command: SourcesCmd,
    },
    /// Install integrations for Claude Code or Codex
    Install(InstallArgs),
    /// Hook entrypoint for Claude Code or Codex notify
    Hook(HookArgs),
    /// Focus the originating terminal/tmux context
    Focus(FocusArgs),
    /// Internal macOS click-wait helper
    #[command(hide = true)]
    WaitMacos(WaitMacosArgs),
}

#[derive(Debug, Args)]
pub struct SendArgs {
    /// Notification title (optional)
    #[arg(long)]
    pub title: Option<String>,

    /// Notification message/body
    #[arg(value_name = "MESSAGE")]
    pub message: String,

    /// Icon path (provider-specific)
    #[arg(long)]
    pub icon: Option<PathBuf>,

    /// Disable icon usage
    #[arg(long)]
    pub no_icon: bool,

    /// Optional URL to associate with the notification
    #[arg(long)]
    pub link: Option<String>,

    /// Sound name to play (macOS)
    #[arg(long)]
    pub sound: Option<String>,

    /// Notification urgency
    #[arg(long, value_enum)]
    pub urgency: Option<UrgencyArg>,

    /// Optional tag/category (provider-specific)
    #[arg(long)]
    pub tag: Option<String>,

    /// Source identifier to resolve icon/logo (e.g. claude, codex)
    #[arg(long)]
    pub source: Option<String>,

    /// Command to execute on click
    #[arg(long)]
    pub on_click: Option<String>,

    /// Wait for user click (blocking)
    #[arg(long)]
    pub wait_for_click: bool,

    /// Detach and wait for click in background (implies --wait-for-click)
    #[arg(long)]
    pub background: bool,

    /// Output a JSON report to stdout
    #[arg(long)]
    pub json: bool,

    /// Provider override (e.g. macos)
    #[arg(long)]
    pub provider: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum ConfigCmd {
    /// Create a default config file
    Init(ConfigInitArgs),
}

#[derive(Debug, Args)]
pub struct ConfigInitArgs {
    /// Override path for config file
    #[arg(long)]
    pub path: Option<PathBuf>,

    /// Overwrite if the config file already exists
    #[arg(long)]
    pub force: bool,
}

#[derive(Debug, Subcommand)]
pub enum ProvidersCmd {
    /// List providers available on this platform
    List,
}

#[derive(Debug, Subcommand)]
pub enum SourcesCmd {
    /// List configured sources
    List,
}

#[derive(Debug, Args)]
pub struct InstallArgs {
    /// Target tool (claude or codex)
    #[arg(value_enum)]
    pub target: InstallTarget,

    /// Apply changes (default is dry-run)
    #[arg(long)]
    pub apply: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum InstallTarget {
    Claude,
    Codex,
}

#[derive(Debug, Args)]
pub struct HookArgs {
    /// Target tool (claude or codex)
    #[arg(value_enum)]
    pub target: InstallTarget,

    /// JSON payload (if not provided, read from stdin)
    pub json: Option<String>,
}

#[derive(Debug, Args)]
pub struct FocusArgs {
    /// tmux session name
    #[arg(long)]
    pub tmux_session: Option<String>,

    /// tmux window id (e.g. @1)
    #[arg(long)]
    pub tmux_window: Option<String>,

    /// tmux pane id (e.g. %3)
    #[arg(long)]
    pub tmux_pane: Option<String>,

    /// Terminal app name (ghostty, iterm, terminal)
    #[arg(long)]
    pub terminal: Option<String>,

    /// Skip terminal activation
    #[arg(long)]
    pub no_activate: bool,
}

#[derive(Debug, Args)]
pub struct WaitMacosArgs {
    /// Path to payload JSON
    #[arg(long)]
    pub payload: PathBuf,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum UrgencyArg {
    Low,
    Normal,
    High,
}
