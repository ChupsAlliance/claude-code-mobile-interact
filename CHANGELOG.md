# Changelog

## v3.0.0 (2026-03-17) — Notification Release

Phiên bản ổn định đầu tiên, tập trung vào **notification** từ Claude Code → người dùng.

### Features
- **Sound notification** — phát file `.wav` tuỳ chọn khi Claude Code xong task hoặc hỏi question
- **Windows toast notification** — popup native Windows thông qua WinRT API
- **Happy push notification** — push tới điện thoại qua happy-coder (end-to-end encrypted)
- **Google Chat webhook** — gửi card notification vào Google Chat space
- **System tray app** — chạy ẩn, double-click mở Settings, right-click menu
- **Auto-start with Windows** — tuỳ chọn tự khởi động khi bật máy (user-level registry)
- **Settings UI** — dark theme, 360px, toggle từng kênh, test button, Happy setup wizard
- **Hook merging** — ghi hooks vào `~/.claude/settings.json` mà không ảnh hưởng hooks từ tool khác

### Hook events
- `Stop` — Claude hoàn thành task
- `Notification` — Claude gửi thông báo
- `PreToolUse` → `AskUserQuestion` — Claude cần người dùng trả lời
- `PermissionRequest` — Claude xin quyền thực thi

### Technical
- Tauri v2 + Rust backend (system tray, config I/O, hook generation)
- Vanilla HTML/CSS/JS frontend (không framework)
- Single combined hook script: `~/.claude/claude-notify-hook.cjs`
- Config round-trip qua `_claudeNotifyConfig` key trong settings.json
- NSIS installer, user-mode (không cần Admin)
- Release binary optimized: strip + LTO + codegen-units=1

### Known Issues
- Toast notification chỉ work khi PowerShell gọi qua `exec()` (không phải `spawn` detached) — đã fix trong hook scripts
- Windows Focus Assist / Do Not Disturb có thể chặn toast

---

## Future Plans

### Two-way Mobile Chat (chưa phát triển)
- Gửi prompt từ điện thoại → Claude Code
- PWA mobile UI xem conversation real-time
- Custom ntfy server hoặc relay
- Cloudflare Tunnel cho truy cập ngoài mạng LAN

### VSCode Extension (ý tưởng)
- Custom extension thay thế official Claude Code extension
- Built-in mobile relay cho two-way sync
- Xem chi tiết: `future-development.md`
