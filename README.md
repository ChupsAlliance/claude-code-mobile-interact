# Claude Notify

Nhận thông báo từ Claude Code khi không nhìn vào màn hình — âm thanh và Windows toast notification.

> **Phiên bản hiện tại: v3.0.2** — Notification-only release. Push notification tới điện thoại và two-way mobile chat sẽ phát triển ở phiên bản sau.

```
Claude Code (hooks)
  └── Stop / Notification / AskUserQuestion / PermissionRequest
       └── claude-notify-hook.cjs
            ├── 🔊 Âm thanh (PowerShell SoundPlayer)
            ├── 🪟 Windows toast notification
            └── 💬 Google Chat webhook (tuỳ chọn)
```

---

## Tính năng

| Kênh | Mô tả | Cần cài thêm? |
|------|--------|---------------|
| 🔊 Sound | Phát file `.wav` khi Claude Code xong task hoặc hỏi question | Không |
| 🪟 Windows Toast | Popup notification góc phải màn hình | Không |
| 💬 Google Chat | Card notification vào Google Chat space | Webhook URL |

### Hook events được theo dõi

| Sự kiện | Khi nào |
|---------|---------|
| `Stop` | Claude hoàn thành task |
| `Notification` | Claude gửi thông báo giữa chừng |
| `PreToolUse` → `AskUserQuestion` | Claude cần bạn trả lời |
| `PermissionRequest` | Claude xin quyền thực thi |

---

## Cài đặt

### Cách 1: Dùng file cài đặt (cho teammate)

Tải file `Claude Notify_3.0.2_x64-setup.exe` → Double-click → Next → Finish.

- Không cần quyền Admin
- Không cần cài Rust hay Node.js
- App tự khởi động sau khi cài

**Đã cài bản cũ?** Chạy installer mới đè lên, không cần uninstall. Sau đó mở app → **Save Settings** → restart Claude Code.

### Cách 2: Build từ source

**Yêu cầu:** Windows 10/11, [Node.js](https://nodejs.org) >= 18, [Rust + Cargo](https://rustup.rs)

```bash
cd claude-code-notify-3.0.0
npm install
npx tauri build
```

File output: `src-tauri/target/release/bundle/nsis/Claude Notify_3.0.2_x64-setup.exe`

### Chạy dev mode

```bash
cd claude-code-notify-3.0.0
npx tauri dev
```

Lần đầu mất ~2-3 phút (Rust compile). Sau đó < 30 giây.

---

## Hướng dẫn sử dụng

### Lần đầu

1. App chạy ẩn trong **system tray** (góc dưới phải, cạnh đồng hồ)
2. **Double-click** icon tray → mở Settings
3. Bật **Enable notifications**
4. Chọn âm thanh, bật Toast / Google Chat tuỳ ý
5. Nhấn **Save Settings**
6. **Reload Claude Code** (đóng/mở lại terminal hoặc VSCode) để hooks có hiệu lực

### Sau khi setup

Mỗi khi Claude Code hoàn thành task, hỏi question, hoặc cần attention — bạn sẽ nhận thông báo qua các kênh đã bật mà không cần nhìn vào màn hình.

---

## Lưu ý quan trọng

- **Phải reload Claude Code sau khi Save** — hooks chỉ load khi session mới start
- **Uninstall đúng cách** — tắt "Enable notifications" → Save trước khi gỡ app
- **Không ảnh hưởng hooks khác** — app chỉ quản lý hooks do nó tạo, giữ nguyên hooks từ tool khác

---

## Cấu trúc project

```
claude-code-mobile-interact/
├── README.md                      # File này
├── CHANGELOG.md                   # Lịch sử thay đổi
├── test-hooks.md                  # Hướng dẫn test hooks
├── future-development.md          # Kế hoạch phát triển
│
└── claude-code-notify-3.0.0/      # Tauri app
    ├── package.json               # npm dependencies
    ├── index.html                 # Settings window UI
    ├── renderer.js                # Settings window logic
    └── src-tauri/
        ├── Cargo.toml             # Rust dependencies
        ├── tauri.conf.json        # Tauri config
        └── src/
            ├── main.rs            # Entry point
            └── lib.rs             # Toàn bộ logic: tray, hooks, config, notifications
```

---

## Roadmap

### v3.0.2 (hiện tại) — Notification only
- ✅ Sound notification (custom .wav)
- ✅ Windows toast notification
- ✅ Google Chat webhook
- ✅ System tray app + Settings UI
- ✅ Auto-start with Windows
- ✅ Hook merging (không ghi đè hooks khác)

### Future — Mobile Push & Two-way Chat
- 📱 Push notification tới điện thoại
- 📱 Gửi prompt từ điện thoại → Claude Code
- 🔄 PWA mobile UI xem conversation real-time
- 🌐 Cloudflare Tunnel cho truy cập từ 4G

---

## License

Internal tool — dùng nội bộ trong team.
