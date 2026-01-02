# Research: Claude Code hooks + Codex CLI events

## Scope

This is a summary of what Claude Code hooks send (inputs, outputs, events) and the closest Codex CLI equivalents for hook-style integration, based on official documentation and issues.

## Claude Code hooks

### Hook configuration and events

Claude Code supports hook events that can run shell commands or prompt-based actions. Documented events include:

- `PreToolUse`
- `PermissionRequest`
- `PostToolUse`
- `Notification`
- `UserPromptSubmit`
- `Stop`
- `SubagentStop`
- `PreCompact`
- `SessionStart`
- `SessionEnd`

Hooks can filter by tool or pattern via matchers.  
Source: https://docs.claude.com/en/docs/claude-code/hooks

### Input payload (stdin JSON)

Each hook receives JSON on stdin with common fields (examples include `session_id`, `transcript_path`, `cwd`, `permission_mode`, `hook_event_name`), plus event-specific data. Examples from the docs:

- `PreToolUse`: `tool_name`, `tool_input`, `tool_use_id`
- `PostToolUse`: `tool_input`, `tool_response`, `tool_use_id`
- `Notification`: `message`, `notification_type`
- `UserPromptSubmit`: `prompt`
- `Stop`/`SubagentStop`: `stop_hook_active`
- `PreCompact`: `trigger`, `custom_instructions`
- `SessionStart`: `source`
- `SessionEnd`: `reason`

Source: https://docs.claude.com/en/docs/claude-code/hooks

### Notification types

`Notification` hook uses a `notification_type` field; examples include:

- `permission_prompt`
- `idle_prompt`
- `auth_success`
- `elicitation_dialog`

Source: https://docs.claude.com/en/docs/claude-code/hooks

### Output semantics (hook control)

Hooks can return exit codes and/or JSON to control behavior. Examples from docs:

- Exit code `0` = success, `2` = blocking (varies by hook type)
- JSON fields like `continue`, `stopReason`, `systemMessage`
- Event-specific fields like `permissionDecision`, `updatedInput`, and `decision`/`reason`

Source: https://docs.claude.com/en/docs/claude-code/hooks

### Environment

Docs mention `CLAUDE_PROJECT_DIR` and `CLAUDE_ENV_FILE` (SessionStart only) for persisting env vars. Hook commands run under user permissions with timeouts and parallel execution behavior.  
Source: https://docs.claude.com/en/docs/claude-code/hooks

## Codex CLI

### Hooks status

There is an open feature request for hooks in the official `openai/codex` repo, implying no built-in hook system yet.

- https://github.com/openai/codex/issues/2109

### JSON event stream (codex exec --json)

`codex exec --json` emits newline-delimited JSON events with types like:

- `thread.started`
- `turn.started`
- `turn.completed`
- `turn.failed`
- `item.*` (agent messages, reasoning, command execution, file changes, MCP calls, web queries, plan updates)

Sources:

- https://developers.openai.com/codex/noninteractive/
- https://developers.openai.com/codex/cli/reference

### Session JSONL logs

An issue notes Codex CLI writes session JSONL logs under `$CODEX_HOME/sessions/YYYY/MM/DD/rollout-*.jsonl`, suitable for tail-based integrations.

- https://github.com/openai/codex/issues/2288?utm_source=openai

### Changelog insights

The Codex changelog notes richer event metadata (git info, cwd, CLI version, thread/turn IDs) and notifications for diffs, plan updates, token usage changes, and compaction events.

- https://developers.openai.com/codex/changelog

### MCP server integration surface

`codex mcp-server` exposes tools like `codex` and `codex-reply`. This is not hooks, but is a programmable interface.

- https://developers.openai.com/codex/mcp/
