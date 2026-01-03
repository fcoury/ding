# Telegram provider plan

## Goals
- Send notifications to Telegram chats via a bot token + chat ID.
- Keep setup minimal and align with existing `ding send` options.
- Provide a clean path for richer notifications later (photos, buttons, threads).

## Provider design
- Provider name: `telegram`
- Primary API: Telegram Bot API `sendMessage`.
- Output: `DeliveryReport` with Telegram `message_id` when available.

## Config
```toml
[telegram]
bot_token = "123456:ABC..."
chat_id = "123456789" # or "-100..." for groups
parse_mode = "MarkdownV2" # optional
silent = false
```

## CLI
- `ding send --provider telegram`
- Optional overrides:
  - `--telegram-token`
  - `--telegram-chat-id`
  - `--telegram-parse-mode` (MarkdownV2/HTML)
  - `--telegram-silent` (maps to Telegram `disable_notification`)

## Message formatting
- Default text = `title` + "\n" + `message` (if title present).
- If `link` exists, append on its own line.
- If `parse_mode = MarkdownV2`, escape special characters:
  `_ * [ ] ( ) ~ ` > # + - = | { } . !`
- Trim to Telegram’s 4096 character limit with a suffix.

## Error handling
- Return `ProviderError::Message` with:
  - HTTP status
  - Telegram error message (`description`) if present
- Clear errors for missing token/chat ID.

## Implementation steps
1. Add `TelegramConfig` to `src/config.rs` + template.
2. Add `src/provider/telegram.rs` implementing `Provider`.
3. Extend CLI args for telegram overrides.
4. Wire `handle_send` to build and dispatch telegram payloads.
5. Add docs to README and USAGE guide.
6. Add unit tests for MarkdownV2 escaping + payload building.

## Future enhancements
- `sendPhoto` when `icon` is provided (upload or URL).
- Inline buttons (`reply_markup`).
- Thread support (`message_thread_id`).
