# Remote relay provider + listener plan

## Decisions

- Listener defaults to binding on all interfaces.
- Remote provider auto-fallbacks to local delivery if the listener is unreachable.
- Prefix titles with hostname by default.
- Forwarding can target multiple providers (remote, telegram, etc.).
  - Default `forward on` overwrites the previous target.
  - Optional cumulative mode appends additional targets.
  - Turning forwarding off disables all targets but preserves their config for later re-enable.

## Goals

- Show notifications from SSH/remote agents on the local machine as if they were local.
- Keep the sender UX simple: `ding send` with a provider switch (or config default).
- Keep click handling local so focus returns to local terminal context.
- Allow forwarding fan-out to multiple providers (e.g., remote + telegram).

## Architecture

- **Remote provider (client):** serializes a notification payload and POSTs to a local listener endpoint.
- **Listener (server):** receives payloads, validates, decorates, and re-emits via the local macOS provider.
- **Transport:** HTTP over localhost, typically via SSH port-forwarding.

## CLI

- `ding listen`
  - Starts an HTTP listener for incoming notifications.
  - Options: `--bind`, `--port`, `--token`, `--allow-host`, `--foreground/--daemon`, `--pidfile`.
  - Defaults: bind `0.0.0.0`, port `4280`.
- `ding send --provider remote`
  - Sends to listener (`remote.host`/`remote.port` or `--remote-host`/`--remote-port`).
  - Options: `--remote-host`, `--remote-port`, `--remote-token`, `--remote-timeout`, `--remote-retries`.
- `ding remote ping`
  - Health check endpoint to confirm listener connectivity.
- `ding forward on|off|toggle|status`
  - Turns forwarding on/off globally (all configured targets).
  - `status` returns on/off plus configured targets.
- `ding forward add|remove|list`
  - `add` sets target (remote/telegram/etc.).
  - `add --append` (or `--cumulative`) retains existing targets.
  - `remove` removes one target from the list.
  - `list` shows configured targets.

## Config

```toml
[forward]
enabled = true
targets = ["remote", "telegram"]
last_target = "remote"

[remote]
host = "127.0.0.1"
port = 4280
token = "..."
timeout_ms = 2000
retries = 2
fallback_to_local = true

[listener]
bind = "0.0.0.0"
port = 4280
token = "..."
require_token = true
prefix_hostname = true

[telegram]
bot_token = "123456:ABC..."
chat_id = "123456789"
parse_mode = "MarkdownV2"
disable_notification = false
```

## Payload

- JSON envelope:
  ```json
  {
    "notification": {
      "title": "...",
      "message": "...",
      "source": "...",
      "sound": "..."
    },
    "context": {
      "origin_host": "...",
      "origin_user": "...",
      "cwd": "...",
      "tmux": { "session": "...", "window": "...", "pane": "..." }
    }
  }
  ```
- Listener uses `context.origin_host` to prefix the title by default.

## Security

- Token-based auth in header (`Authorization: Bearer <token>` or `X-Ding-Token`).
- Listener defaults to binding on all interfaces and should be protected with token auth and allowlists.
- Optional allowlist for hostnames/IPs.

## Click handling

- Listener attaches `on_click` with local `ding focus` to restore the local context.
- Preserve incoming metadata so we can route to specific terminal panes later.

## Fallback behavior

- If remote delivery fails (timeout, refused, auth error), auto-fallback to local provider.
- Provide a `--no-fallback` flag to disable if needed.
- Forwarding fan-out does not stop other targets when one target fails.

## Implementation steps (proposed order)

1. **Shared payload types** in `src/remote.rs` (request/response + helpers).
2. **Listener command**: basic HTTP server, token auth, hostname prefixing, local delivery.
3. **Remote provider**: POST JSON, retry logic, fallback to local.
4. **Config + CLI flags** for remote/listener settings.
5. **Docs**: SSH port-forward example + config snippets.
6. **Tests** for payload validation + auth + fallback.

## SSH usage example

- Local machine:
  - `ding listen --bind 0.0.0.0 --port 4280`
- Remote machine:
  - `ssh -L 4280:127.0.0.1:4280 user@remote-host`
  - `ding send --provider remote --remote-host 127.0.0.1 --remote-port 4280 "done"`
