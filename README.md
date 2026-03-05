# Claude Code Mobile Bridge

Nhận thông báo từ Claude Code trên điện thoại và gửi prompt lại — không cần ngồi trước máy tính.

```
Desktop (VSCode + Claude Code)
  └── hook events → ntfy-server (port 2586)
                         └── PWA (port 2587) ← điện thoại mở URL này
                                └── gửi prompt → cc-commands → claude --resume
```

---

## Yêu cầu

- Windows 10/11
- [Node.js](https://nodejs.org) (để chạy ntfy-server.js)
- [Rust + cargo](https://rustup.rs) (để build Tauri)
- [Claude Code](https://claude.ai/code) đã cài và đăng nhập

---

## Lần đầu setup (5 phút)

### 1. Cài Rust (nếu chưa có)

```
https://rustup.rs → tải rustup-init.exe → chạy → restart terminal
```

Sau khi cài xong, thêm cargo vào PATH **một lần duy nhất**:

```cmd
setx PATH "%USERPROFILE%\.cargo\bin;%PATH%"
```

**Đóng terminal và mở lại** sau khi chạy lệnh trên.

### 2. Cài dependencies

```cmd
cd D:\claude-code-mobile-interact\pwa-app
npm install

cd ..\tauri-app
npm install
```

### 3. Chạy app (dev mode)

```cmd
cd D:\claude-code-mobile-interact\tauri-app
npx tauri dev
```

Lần đầu sẽ mất ~2-3 phút để Rust compile. Các lần sau dưới 30 giây.

Khi thấy dòng này là thành công:

```
INFO pwa_server: PWA server listening on http://0.0.0.0:2587
```

---

## Cách dùng

### Trên máy tính

App sẽ chạy ẩn trong **system tray** (góc phải taskbar). Click vào icon để mở Settings.

Settings window cho phép:
- Xem trạng thái các server
- Install hooks vào Claude Code
- Xem QR code để pair điện thoại
- Cấu hình tunnel URL

### Trên điện thoại

**Cùng mạng WiFi**: mở trình duyệt, vào:

```
http://192.168.0.39:2587
```

*(thay IP bằng IP thực của máy bạn — xem trong Settings > WiFi)*

**Từ mạng khác** (4G, mạng khác): cần setup Cloudflare Tunnel — xem phần bên dưới.

### Nhận thông báo từ Claude Code

Claude Code sẽ tự động gửi về điện thoại khi:
- **Stop** — Claude hoàn thành một task
- **Notification** — Claude cần thông báo gì đó
- **AskUserQuestion** — Claude đang chờ bạn trả lời

### Gửi prompt từ điện thoại

Gõ vào ô text phía dưới PWA và nhấn **↑ Send** (hoặc Enter).

Prompt sẽ được gửi tới `cc-commands` topic. Tauri app nhận được và chạy:

```
claude --print --resume <session_id> "<prompt của bạn>"
```

---

## Setup Cloudflare Tunnel (truy cập từ 4G / ngoài nhà)

Để điện thoại kết nối được dù không cùng WiFi:

### 1. Cài cloudflared

Tải từ https://github.com/cloudflare/cloudflared/releases → `cloudflared-windows-amd64.exe`

Đổi tên thành `cloudflared.exe` và để vào `C:\Windows\System32\` hoặc một thư mục trong PATH.

### 2. Đăng nhập Cloudflare

```cmd
cloudflared tunnel login
```

### 3. Tạo tunnel

```cmd
cloudflared tunnel create cc-mobile
cloudflared tunnel route dns cc-mobile cc-mobile.yourdomain.com
```

### 4. Chạy tunnel

```cmd
cloudflared tunnel --url http://localhost:2587 run cc-mobile
```

### 5. Cập nhật config trong app

Mở Settings window (click tray icon) → điền **Cloudflare tunnel URL** → blur để lưu.

---

## Lỗi thường gặp

**`cargo not found` khi chạy `npx tauri dev`**

```cmd
setx PATH "%USERPROFILE%\.cargo\bin;%PATH%"
:: Đóng terminal và mở lại
```

**`Port 1420 is already in use`**

```cmd
taskkill /F /IM node.exe
npx tauri dev
```

**`Access is denied` khi build**

```cmd
taskkill /F /IM cc-mobile-bridge.exe
npx tauri dev
```

**`localhost:2587` không mở được trên máy**

PWA server chỉ start sau khi Tauri app compile xong và chạy. Chờ đến khi terminal log ra:

```
INFO pwa_server: PWA server listening on http://0.0.0.0:2587
```

**Điện thoại không vào được `192.168.x.x:2587`**

Kiểm tra Windows Firewall — cần allow port 2587 inbound:

```cmd
netsh advfirewall firewall add rule name="CC Mobile Bridge" dir=in action=allow protocol=TCP localport=2587
```

---

## Build installer (để share cho teammate)

```cmd
cd D:\claude-code-mobile-interact\tauri-app
npx tauri build
```

Installer `.exe` sẽ xuất hiện ở:

```
tauri-app\src-tauri\target\release\bundle\nsis\Claude Code Mobile Bridge_0.1.0_x64-setup.exe
```

Share file này cho teammate — họ double-click cài xong là dùng được, không cần Rust hay Node.

---

## Cấu trúc project

```
claude-code-mobile-interact/
├── bin/
│   └── ntfy-server.js        # Custom ntfy-compatible HTTP server (Node.js, zero deps)
├── hooks/
│   └── mobile-bridge.cjs     # Claude Code hook script (auto-installed)
├── pwa-app/                  # Mobile UI (React + Vite PWA)
│   └── src/
│       ├── components/       # EventCard, ReplyBox
│       └── lib/              # ntfy-client, use-event-feed
└── tauri-app/                # Desktop app (Tauri + Rust)
    ├── src/                  # Settings window (React)
    └── src-tauri/src/
        ├── lib.rs            # App entry, system tray
        ├── ntfy.rs           # ntfy-server process manager + subscriber
        ├── pwa_server.rs     # Axum server: serves PWA + proxies ntfy
        ├── bridge.rs         # claude --print --resume executor
        ├── hook_installer.rs # Merges hooks vào ~/.claude/settings.json
        └── config.rs         # Config load/save
```
