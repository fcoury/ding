# Ding Implementation Plan: Sources + Click Handling (macOS)

## Goals
- Identify notification source (Claude, Codex, etc.) and resolve logo/icon automatically.
- Support click handling in both blocking and non-blocking modes.
- Enable immediate integrations (tmux/Ghostty/terminal focus) via `--on-click` commands.

## Decisions
- **Blocking is optional** and controlled by `--wait-for-click`.
- **Non-blocking with click handling** uses a detached helper process.
- **Source-based icon resolution** is config-driven, with CLI overrides.

## Phase 1 — Data model + config
1. Add `source` to `Notification`.
2. Add `sources` map to config:
   ```toml
   [sources.claude]
   icon = "/path/to/claude.icns"
   app_bundle_id = "com.apple.Terminal" # optional

   [sources.codex]
   icon = "/path/to/openai.icns"
   ```
3. Resolution order:
   - CLI `--icon`
   - `sources.<source>.icon`
   - Provider defaults

## Phase 2 — CLI surface
1. `ding send` adds:
   - `--source <string>`
   - `--on-click <command>`
   - `--wait-for-click`
   - `--background` (detached helper)
2. Validation:
   - `--background` requires `--on-click`
   - `--background` implies `--wait-for-click`

## Phase 3 — Provider + click handling
1. Extend provider trait to accept `SendOptions { wait_for_click }`.
2. Return `DeliveryReport { outcome }` with click info.
3. macOS provider maps `NotificationResponse` to `DeliveryOutcome`.
4. If `--wait-for-click` and outcome is click/action, run `--on-click`.

## Phase 4 — Detached helper
1. Hidden subcommand `__wait-macos --payload <json>`.
2. Main process writes a payload JSON, spawns helper, and exits.
3. Helper sends notification with `wait_for_click = true`, then runs `--on-click`.

## Phase 5 — Context handoff (future)
- Add auto-detected tmux context (`session/window/pane`) to payload.
- Provide optional Ghostty activation before tmux focus.

## Deliverables
- CLI flags
- Config updates
- macOS click handling (blocking + detached)
- Example config using downloaded logos

