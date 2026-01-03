# Wakedev Usage Guide

This guide covers detailed setup and usage of wakedev for local notifications, remote delivery, and integrations with Claude Code and OpenAI Codex.

## Table of Contents

1. [Installation](#installation)
2. [Basic Local Usage](#basic-local-usage)
3. [Configuration](#configuration)
4. [Claude Code Integration](#claude-code-integration)
5. [OpenAI Codex Integration](#openai-codex-integration)
6. [Remote Notifications](#remote-notifications)
7. [Advanced Usage](#advanced-usage)
8. [Troubleshooting](#troubleshooting)

---

## Installation

### Build from source

```bash
git clone https://github.com/fcoury/wakedev.git
cd wakedev
cargo build --release
```

The binary will be at `./target/release/wakedev`.

### Install to PATH

```bash
cargo install --path .
```

This installs to `~/.cargo/bin/wakedev`. Ensure `~/.cargo/bin` is in your PATH.

### Verify installation

```bash
wakedev --version
wakedev send "Hello from wakedev!"
```

---

## Basic Local Usage

### Simple notifications

```bash
# Basic message
wakedev send "Build complete"

# With title
wakedev send "All 42 tests passed" --title "Test Results"

# With sound
wakedev send "Deployment finished" --sound default

# Silent notification
wakedev send "Background task done" --silent
```

### Urgency levels

```bash
# Low urgency (subtle)
wakedev send "Sync complete" --urgency low

# Normal urgency (default)
wakedev send "Build finished" --urgency normal

# High urgency (prominent)
wakedev send "Build failed!" --urgency high
```

### Tags for grouping

```bash
wakedev send "Test 1 passed" --tag tests
wakedev send "Test 2 passed" --tag tests
wakedev send "Test 3 failed" --tag tests --urgency high
```

### Wait for interaction

```bash
# Block until notification is clicked
wakedev send "Review required" --wait-for-click
echo "User clicked the notification"

# Run command on click
wakedev send "PR ready for review" --on-click "open https://github.com/repo/pull/123"

# Background wait (doesn't block)
wakedev send "Check when ready" --background --on-click "wakedev focus"
```

### JSON output

```bash
wakedev send "Test" --json
# Output: {"delivered":true,"clicked":false,"action":null}
```

---

## Configuration

### Initialize config

```bash
wakedev config init
```

Creates `~/.config/wakedev/config.toml`.

### View config

```bash
# Show config file path
wakedev config path

# Display full config
wakedev config list
```

### Set config values

```bash
# Set default sound
wakedev config set macos.sound default

# Set remote host
wakedev config set remote.host 192.168.1.100

# Set remote port
wakedev config set remote.port 4280

# Set listener token
wakedev config set listener.token "your-secret-token"
```

### Full config example

```toml
# ~/.config/wakedev/config.toml

default_provider = "macos"

[macos]
sound = "default"
# app_bundle_id = "com.apple.Terminal"

[remote]
host = "192.168.1.100"
port = 4280
token = "your-secret-token"
timeout_ms = 2000
retries = 2
fallback_to_local = true

[listener]
bind = "0.0.0.0"
port = 4280
token = "your-secret-token"
require_token = true
prefix_hostname = true

[sources.claude]
# Custom icon for Claude notifications
# icon = "/path/to/claude.icns"

[sources.codex]
# Custom icon for Codex notifications
# icon = "/path/to/openai.icns"
```

---

## Claude Code Integration

Claude Code is Anthropic's AI coding assistant. Wakedev can receive hook events from Claude Code and display native notifications.

### Step 1: View installation instructions

```bash
wakedev install claude
```

This shows what changes will be made to your Claude Code configuration.

### Step 2: Apply the integration

```bash
wakedev install claude --apply
```

This modifies `~/.claude/settings.json` to add notification hooks.

### Step 3: Verify setup

The integration adds this to your Claude settings:

```json
{
  "hooks": {
    "Notification": [
      {
        "matcher": "",
        "hooks": ["wakedev hook claude"]
      }
    ]
  }
}
```

### How it works

1. Claude Code emits events during operation (task complete, permission needed, etc.)
2. Events are sent to `wakedev hook claude` via stdin as JSON
3. Wakedev parses the event and shows an appropriate notification
4. Clicking the notification returns focus to your terminal/tmux session

### Event urgency mapping

| Event Type | Urgency |
|------------|---------|
| Permission prompts | High |
| Failures/errors | High |
| Auth issues | High |
| Task completion | Normal |
| File changes | Normal |
| Progress updates | Low |
| Plan changes | Low |

### Manual hook testing

```bash
echo '{"type":"task_complete","message":"Refactoring done"}' | wakedev hook claude
```

### Using with remote sessions

If you're running Claude Code over SSH, enable remote forwarding:

```bash
# On your local machine
wakedev listen

# On the remote server
wakedev remote forward on --host YOUR_LOCAL_IP
```

Now Claude Code notifications from the remote session appear on your local machine.

---

## OpenAI Codex Integration

OpenAI Codex CLI is another AI coding assistant. Wakedev provides similar integration.

### Step 1: View installation instructions

```bash
wakedev install codex
```

### Step 2: Apply the integration

```bash
wakedev install codex --apply
```

### Step 3: Usage

Works identically to Claude Code integration. Events from Codex CLI are processed and displayed as native notifications.

### Manual hook testing

```bash
echo '{"type":"completion","message":"Code generated"}' | wakedev hook codex
```

---

## Remote Notifications

Remote notifications let you receive notifications on your local machine from commands running on remote servers (via SSH).

### Architecture

```
┌─────────────────┐         HTTP POST         ┌─────────────────┐
│  Remote Server  │  ──────────────────────►  │  Local Machine  │
│                 │                           │                 │
│  wakedev send   │   :4280/notify            │  wakedev listen │
│  (remote mode)  │                           │  (HTTP server)  │
└─────────────────┘                           └─────────────────┘
                                                      │
                                                      ▼
                                              ┌─────────────────┐
                                              │ macOS Notif.    │
                                              │ Center          │
                                              └─────────────────┘
```

### Local machine setup (receiver)

#### 1. Configure the listener

```bash
# Set auth token
wakedev config set listener.token "your-secret-token"
wakedev config set listener.require_token true
wakedev config set listener.bind "0.0.0.0"
wakedev config set listener.port 4280
```

#### 2. Start the listener

```bash
wakedev listen
```

Or with command-line options:

```bash
wakedev listen \
  --bind 0.0.0.0 \
  --port 4280 \
  --token "your-secret-token" \
  --require-token
```

#### 3. Keep listener running

Use a process manager or terminal multiplexer:

```bash
# With tmux
tmux new-session -d -s wakedev 'wakedev listen'

# With launchd (macOS)
# Create ~/Library/LaunchAgents/com.wakedev.listener.plist
```

### Remote server setup (sender)

#### 1. Install wakedev on remote

```bash
# SSH to remote server
ssh user@remote-server

# Install wakedev
git clone https://github.com/fcoury/wakedev.git
cd wakedev
cargo install --path .
```

#### 2. Configure remote delivery

```bash
wakedev config set remote.host "YOUR_LOCAL_IP"
wakedev config set remote.port 4280
wakedev config set remote.token "your-secret-token"
```

#### 3. Enable remote forwarding

```bash
wakedev remote forward on
```

Check status:

```bash
wakedev remote forward status
```

#### 4. Test connection

```bash
wakedev remote ping
```

#### 5. Send notifications

```bash
wakedev send "Remote build complete"
```

The notification appears on your local machine with `[hostname]` prefix.

### Security considerations

#### Token authentication

Always use a token in production:

```bash
# Local
wakedev listen --token "secret" --require-token

# Remote
wakedev config set remote.token "secret"
```

#### Host allowlist

Restrict which hosts can send notifications:

```bash
wakedev listen --allow-host 192.168.1.0/24 --allow-host 10.0.0.5
```

#### Firewall

Ensure port 4280 (or your chosen port) is accessible from remote servers but not the public internet.

### Fallback behavior

If remote delivery fails, wakedev can fall back to local notifications:

```bash
# Enable fallback (default)
wakedev config set remote.fallback_to_local true

# Disable fallback
wakedev send "Must reach remote" --no-fallback
```

---

## Advanced Usage

### Context-aware click handling

When you click a notification, wakedev can return focus to the originating terminal:

```bash
wakedev send "Click to return" --on-click "wakedev focus"
```

The focus command uses captured context:

- **Terminal app**: iTerm, Ghostty, Terminal, etc.
- **Tmux session/window/pane**: Restores exact pane

### Custom click commands

```bash
# Open URL
wakedev send "PR merged" --on-click "open https://github.com/repo/pull/123"

# Run script
wakedev send "Deploy ready" --on-click "./scripts/deploy.sh"

# Multiple commands
wakedev send "Review" --on-click "wakedev focus && echo 'Focused'"
```

### Environment in click handlers

Click commands receive context via environment variables:

```bash
wakedev send "Test" --source myapp --on-click 'echo $WAKEDEV_SOURCE'
# Outputs: myapp
```

Available variables:

- `WAKEDEV_SOURCE`
- `WAKEDEV_TITLE`
- `WAKEDEV_MESSAGE`
- `WAKEDEV_TAG`
- `WAKEDEV_TMUX_SESSION`
- `WAKEDEV_TMUX_WINDOW`
- `WAKEDEV_TMUX_PANE`
- `WAKEDEV_TERMINAL_APP`
- `WAKEDEV_CONTEXT_JSON`

### Build workflow integration

```bash
#!/bin/bash
# build-notify.sh

if cargo build --release; then
    wakedev send "Build succeeded" \
        --title "Cargo Build" \
        --tag build \
        --sound default
else
    wakedev send "Build failed" \
        --title "Cargo Build" \
        --tag build \
        --urgency high \
        --wait-for-click
fi
```

### Long-running task wrapper

```bash
#!/bin/bash
# notify-on-complete.sh

"$@"
exit_code=$?

if [ $exit_code -eq 0 ]; then
    wakedev send "Command succeeded: $1" --sound default
else
    wakedev send "Command failed: $1 (exit $exit_code)" --urgency high
fi

exit $exit_code
```

Usage:

```bash
./notify-on-complete.sh make test
```

### Provider override

Force a specific provider:

```bash
# Always use local macOS
wakedev send "Local only" --provider macos

# Always use remote
wakedev send "Remote only" --provider remote
```

---

## Troubleshooting

### Notifications not appearing

1. Check macOS notification settings:
   - System Preferences → Notifications → Terminal (or your terminal app)
   - Ensure notifications are enabled

2. Test basic notification:
   ```bash
   wakedev send "Test notification"
   ```

3. Check if Do Not Disturb is enabled

### Remote notifications not working

1. Test connectivity:
   ```bash
   wakedev remote ping
   ```

2. Check listener is running:
   ```bash
   curl http://YOUR_LOCAL_IP:4280/health
   ```

3. Verify token matches on both sides

4. Check firewall allows port 4280

5. Verify remote forwarding is enabled:
   ```bash
   wakedev remote forward status
   ```

### Click handler not working

1. Test the command directly:
   ```bash
   wakedev focus
   ```

2. Check tmux context is captured:
   ```bash
   wakedev send "Test" --json --on-click "env | grep WAKEDEV"
   ```

3. For background mode, check the process is running:
   ```bash
   ps aux | grep wakedev
   ```

### Claude/Codex integration issues

1. Verify hook is installed:
   ```bash
   cat ~/.claude/settings.json | grep wakedev
   ```

2. Test hook manually:
   ```bash
   echo '{"type":"test"}' | wakedev hook claude
   ```

3. Check wakedev is in PATH for the hook

### Sound not playing

1. Check system volume
2. Verify sound name is valid:
   ```bash
   # List available sounds
   ls /System/Library/Sounds/
   ```
3. Use `--sound default` for the default notification sound

### Config not loading

1. Check config path:
   ```bash
   wakedev config path
   ```

2. Validate config syntax:
   ```bash
   wakedev config list
   ```

3. Check file permissions on config file

---

## Summary

| Use Case | Command |
|----------|---------|
| Simple notification | `wakedev send "Message"` |
| With sound | `wakedev send "Done" --sound default` |
| Wait for click | `wakedev send "Review" --wait-for-click` |
| Run on click | `wakedev send "Open" --on-click "open URL"` |
| Remote setup | `wakedev listen` (local) + `wakedev remote forward on` (remote) |
| Claude integration | `wakedev install claude --apply` |
| Codex integration | `wakedev install codex --apply` |
| Check config | `wakedev config list` |
| Test remote | `wakedev remote ping` |
