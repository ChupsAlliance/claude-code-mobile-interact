# Future Development

Các ý tưởng phát triển tiếp theo cho Claude Code Mobile Interact.

---

## 1. Two-way Mobile Chat (ưu tiên cao)

**Mục tiêu:** Gửi prompt từ điện thoại → Claude Code trên máy tính, không cần ngồi trước màn hình.

### Kiến trúc dự kiến

```
Claude Code (hooks) → ntfy-server (port 2586)
                           ↓ SSE stream
                        Axum PWA server (port 2587)
                           ↓ serves
                        PWA (React) embedded via include_dir!
                           ↓ QR code
                        Phone browser (PWA)

Phone gửi prompt → cc-commands ntfy topic → Tauri app → claude --print --resume <id>
```

### Yêu cầu
- Custom ntfy server (Node.js) cho messaging
- PWA mobile UI (React + Vite) hiển thị conversation
- Tauri app listen topic, chạy `claude --print --resume`
- QR code pairing (pair 1 lần)
- Cloudflare Tunnel cho truy cập từ 4G

### Estimated effort
- ntfy server: 1 day
- PWA mobile UI: 2-3 days
- Tauri bridge integration: 1-2 days
- Polish + testing: 1-2 days

---

## 2. VSCode Extension — Mobile Code Bridge (ý tưởng)

**Mục tiêu:** Custom VSCode extension thay thế official Claude Code extension, với built-in mobile two-way interaction.

### Kiến trúc
```
User (VSCode) ──┐
                 ├── Our Extension ── claude CLI (stream-json) ── Claude API
User (Mobile) ──┘        │
                    Relay server
                   (Happy relay or custom)
```

### Cách hoạt động
1. Extension spawn `claude --print --input-format stream-json --output-format stream-json`
2. Parse JSON stream → hiển thị trong VSCode webview
3. Relay messages tới mobile qua Happy hoặc custom relay
4. Mobile gửi message → relay → extension → claude stdin

### Ưu điểm
- Sở hữu toàn bộ message pipeline → two-way sync
- Hỗ trợ Vietnamese, images, rich UI trong webview
- Không phụ thuộc Anthropic extension

### Estimated effort: 4-8 ngày

### Lý do không integrate với Claude Code extension hiện tại
- VSCode extension dùng stream-json SDK protocol internally
- Happy cần PTY (terminal I/O) → không capture được JSON stream
- Không có API inject messages vào running session
