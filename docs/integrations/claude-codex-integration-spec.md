# Integration Spec: Claude Code Hooks + Codex CLI Events

## Goals

- Provide stable, low-friction notification hooks for Claude Code and Codex CLI.
- Normalize event data into `wakedev send` calls with consistent titles, urgency, and message bodies.
- Keep integration simple: a single script per tool that can be installed and called from hooks or pipelines.

## Common Event Model (wakedev-facing)

Map any tool event into this payload:

- `title`: short summary (tool + event)
- `message`: details or truncated content
- `urgency`: `low | normal | high`
- `tag`: tool name or event category
- `icon`: optional (tool-specific or static)

Suggested default mapping:

- **High urgency:** approval/permission prompts, failures, auth issues
- **Normal urgency:** turn completion, new output, file changes
- **Low urgency:** progress chatter, plan updates, non-critical notices

## Claude Code Integration

### Hook strategy

Use Claude Code hooks to invoke a script that reads JSON from stdin and translates it to `wakedev send`.

Recommended hooks to enable:

- `Notification` (permission prompt, idle prompt, auth success)
- `PostToolUse` (surface tool output changes)
- `Stop` / `SubagentStop` (completion notifications)
- `SessionStart` / `SessionEnd` (session lifecycle)

### Script contract

- Input: JSON on stdin (Claude hook payload).
- Output: none, exit code 0.
- Behavior: map `hook_event_name` + event-specific fields to a `wakedev send` call.

### Example hook script (shell + python JSON parsing)

Save as `wakedev-claude-hook` and make executable.

```bash
#!/usr/bin/env bash
set -euo pipefail

python3 - <<'PY'
import json, sys, subprocess

data = json.load(sys.stdin)
hook = data.get("hook_event_name", "Unknown")

# Default mapping
urgency = "normal"
if hook in ("Notification", "PermissionRequest"):
    urgency = "high"
if hook in ("SessionStart", "SessionEnd"):
    urgency = "low"

# Build title + message
if hook == "Notification":
    ntype = data.get("notification_type", "unknown")
    title = f"Claude Code: {ntype}"
    message = data.get("message", "")
elif hook in ("Stop", "SubagentStop"):
    title = "Claude Code: finished"
    message = "Task completed"
elif hook == "PostToolUse":
    title = "Claude Code: tool finished"
    message = data.get("tool_name", "unknown")
else:
    title = f"Claude Code: {hook}"
    message = data.get("message") or data.get("prompt") or ""

cmd = [
    "wakedev", "send",
    "--title", title,
    "--message", message,
    "--urgency", urgency,
    "--tag", "claude-code",
]

subprocess.run(cmd, check=False)
PY
```

### Hook config example

This is a conceptual example; follow Claude Code docs to register hooks.

```
// settings.json (illustrative)
"hooks": {
  "Notification": ["/usr/local/bin/wakedev-claude-hook"],
  "PostToolUse": ["/usr/local/bin/wakedev-claude-hook"],
  "Stop": ["/usr/local/bin/wakedev-claude-hook"],
  "SessionEnd": ["/usr/local/bin/wakedev-claude-hook"]
}
```

## Codex CLI Integration

Codex CLI does not have hook configs. The recommended approach is to:

1. run `codex exec --json` and parse JSONL events, or
2. tail the session JSONL rollout file.

### Option A: `codex exec --json` pipeline

Run codex and pipe to a watcher that emits `wakedev` notifications.

```bash
codex exec --json "<prompt>" | wakedev-codex-hook
```

### Option B: tail session JSONL logs

If you want background notifications while codex is running:

```bash
tail -F "$CODEX_HOME/sessions/$(date +%Y/%m/%d)"/rollout-*.jsonl | wakedev-codex-hook
```

### Example Codex hook script

Save as `wakedev-codex-hook` and make executable.

```bash
#!/usr/bin/env bash
set -euo pipefail

python3 - <<'PY'
import json, sys, subprocess

def notify(title, message, urgency="normal"):
    subprocess.run([
        "wakedev", "send",
        "--title", title,
        "--message", message,
        "--urgency", urgency,
        "--tag", "codex",
    ], check=False)

for line in sys.stdin:
    line = line.strip()
    if not line:
        continue

    try:
        event = json.loads(line)
    except Exception:
        continue

    etype = event.get("type") or event.get("event")

    if etype in ("turn.completed", "turn.failed"):
        urgency = "high" if etype == "turn.failed" else "normal"
        notify("Codex: turn finished", etype, urgency)
        continue

    # Optional: highlight file changes or plan updates
    if etype in ("item.file_change", "item.diff"):
        notify("Codex: file changes", "Working tree updated", "normal")
        continue

    if etype in ("item.plan", "item.plan_update"):
        notify("Codex: plan updated", "New plan available", "low")
        continue
PY
```

## Suggested wakedev CLI behavior (future)

These would make integrations smoother:

- `wakedev send --json` for programmatic outputs.
- `wakedev send --dedupe-key` to avoid repeated notifications.
- `wakedev send --title-from stdin` to support streaming usage.
- `wakedev config set` for faster CLI integration.

## Open Questions

- Should we add a built-in `wakedev hooks` module to install these scripts automatically?
- Do we want a reusable `wakedev watch codex` and `wakedev hook claude` subcommand?
- Should `--urgency` default to `high` for any `permission_*` notifications?
