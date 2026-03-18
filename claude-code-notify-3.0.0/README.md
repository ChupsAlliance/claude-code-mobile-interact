# Claude Notify

Nhận thông báo từ Claude Code khi không nhìn vào màn hình — âm thanh, Windows toast, Google Chat, và push notification tới điện thoại qua Happy app.

---

## Cài đặt

### Cách nhanh (dùng luôn)

```
cd claude-code-notify-3.0.0
npx tauri dev
```

### Build file cài đặt (chia sẻ cho người khác)

```
cd claude-code-notify-3.0.0
npm install
npx tauri build
```

File output: `src-tauri/target/release/bundle/nsis/Claude Notify_3.0.0_x64-setup.exe`

Chạy file đó → Next → Finish. App tự khởi động sau khi cài, không cần quyền Admin.

---

## Mở app

App chạy ẩn — **không có cửa sổ trên taskbar**. Tìm biểu tượng ở **system tray** (góc dưới phải màn hình, cạnh đồng hồ).

- **Double-click** icon → mở cửa sổ Settings
- **Chuột phải** icon → menu "Open Settings" / "Quit"

---

## Hướng dẫn sử dụng

### Lần đầu cấu hình

1. Double-click icon tray → mở Settings
2. Bật **Enable notifications**
3. Chọn file âm thanh (hoặc dùng mặc định)
4. Bật thêm các kênh muốn dùng (Toast, Happy, Google Chat)
5. Nhấn **Save Settings**
6. **Reload Claude Code** (đóng/mở lại terminal hoặc VSCode) để hooks có hiệu lực

Sau bước này, mỗi khi Claude Code hoàn thành task hoặc cần input từ bạn, app sẽ tự động thông báo qua các kênh đã bật.

---

## Các tính năng

### Enable notifications
Toggle chính. Bật = app ghi hooks vào `~/.claude/settings.json`. Tắt = xóa hooks của Claude Notify, **giữ nguyên hooks của app khác**.

---

### Start with Windows
Tự động chạy Claude Notify khi khởi động máy. Không cần quyền Admin — dùng Windows Registry user-level.

---

### Windows toast notifications
Hiện popup ở góc phải màn hình (Windows Action Center). Không cần cài thêm gì.

Hoạt động khi Claude Code:
- Hoàn thành task → popup "Task Finished"
- Cần bạn trả lời → popup "Question"
- Gửi thông báo → popup "Attention"

---

### Sound — task finished
Phát file `.wav` khi Claude Code hoàn thành task (sự kiện `Stop`).

| Nút | Tác dụng |
|-----|---------|
| 📁 Browse | Chọn file WAV bất kỳ |
| ▶ Play | Nghe thử âm thanh ngay |

Default: `C:\Windows\Media\notify.wav`

---

### Sound — asking question
Phát file `.wav` **khác** khi Claude Code hỏi bạn (tool `AskUserQuestion`).

Nên chọn âm thanh khác với Stop để phân biệt "xong rồi" vs "cần input".

Default: `C:\Windows\Media\Ring01.wav`

---

### Happy push notification
Gửi push notification thẳng tới điện thoại qua **Happy** (happy-coder). Miễn phí, không cần tài khoản, mã hóa end-to-end.

---

#### Bước 1 — Cài app trên điện thoại

| Nền tảng | Link |
|----------|------|
| iOS | App Store → tìm **"Happy - Claude Code Client"** |
| Android | Google Play → tìm **"Happy"** (com.ex3ndr.happy) |
| Không muốn cài app | Mở trình duyệt: **https://app.happy.engineering** |

---

#### Bước 2 — Cài CLI trên máy tính

Yêu cầu Node.js >= 18.

```
npm install -g happy-coder
```

---

#### Bước 3 — Pair điện thoại với máy tính

```
happy auth login
```

Terminal sẽ hiện một **QR code**. Mở app Happy trên điện thoại → quét QR. Chỉ cần làm **một lần duy nhất** — sau đó tự động kết nối lại mà không cần quét lại.

> Không có email, không có password, không cần tạo tài khoản. Hoạt động hoàn toàn bằng mã hóa khóa công khai (Curve25519).

Kiểm tra đã pair thành công:
```
happy auth status
```

---

#### Bước 4 — Test thử

```
happy notify -p "Hello từ máy tính" -t "Test"
```

Điện thoại sẽ nhận được push notification ngay lập tức.

---

#### Bước 5 — Bật trong Claude Notify

1. Bật toggle **Happy push notification**
2. Nhấn nút **↑** để gửi test (điện thoại phải nhận được thông báo)
3. Nhấn **Save Settings**

Từ đây, mỗi khi Claude Code hoàn thành task hoặc hỏi bạn, điện thoại sẽ rung.

---

**Các lệnh auth khác:**

```bash
happy auth login --force   # Xóa pair cũ và pair lại từ đầu
happy auth logout          # Xóa toàn bộ credentials (~/.happy/)
happy auth status          # Xem trạng thái hiện tại
```

Nếu Claude Notify báo lỗi `happy-coder not installed`, chạy lại `npm install -g happy-coder` và đảm bảo Node.js >= 18.

---

### Google Chat webhook
Gửi card notification vào Google Chat space.

**Lấy webhook URL:**
1. Vào Google Chat space → click tên space → **Manage webhooks**
2. Nhấn **Add webhook** → đặt tên → Save → copy URL
3. Paste URL vào ô **Google Chat webhook** trong app
4. Nhấn nút **💬 test** → space sẽ nhận được một card thử
5. Nhấn **Save Settings**

**Card gửi theo sự kiện:**

| Sự kiện | Tiêu đề card | Icon |
|---------|-------------|------|
| Stop (xong task) | Task Finished | ✅ |
| Notification | Attention | 🔔 |
| AskUserQuestion | Question | ❓ |

---

## Sự kiện Claude Code được theo dõi

| Sự kiện | Khi nào xảy ra |
|---------|--------------|
| `Stop` | Claude hoàn thành xử lý, trả quyền điều khiển |
| `Notification` | Claude gửi thông báo giữa chừng |
| `PreToolUse` (AskUserQuestion) | Claude cần người dùng nhập input |
| `SessionEnd` | ~~Claude Code session kết thúc~~ (đã bỏ — không cần thiết) |
| `PermissionRequest` | Claude cần quyền thực thi (file, command) |

> **Note:** Hook format mới (Claude Code 2026+) yêu cầu `matcher` + `hooks[]` array, không phải flat `type`/`command`.

---

## Lưu ý quan trọng

**Phải reload Claude Code sau khi Save** — hooks chỉ có hiệu lực khi Claude Code khởi động lại. Với VSCode Extension: đóng và mở lại cửa sổ chat.

**Uninstall đúng cách** — trước khi gỡ app, nên tắt **Enable notifications** → Save để dọn hooks ra khỏi `settings.json`. Nếu gỡ thẳng, hooks vẫn còn trong file nhưng sẽ không chạy được (binary không còn).

**Không ảnh hưởng hooks khác** — app chỉ quản lý hook entry do nó tạo ra. Hook từ các tool khác (ví dụ script tùy chỉnh) được giữ nguyên hoàn toàn.

---

## Technical Reference

### Kiến trúc

```
Claude Code fires hook
  └─ node "~/.claude/claude-notify-hook.cjs" "<event>" "<message>"
       ├─ Đọc stdin (JSON event từ Claude Code)
       ├─ Phát âm thanh (child_process → PowerShell SoundPlayer)
       ├─ Toast notification (child_process → PowerShell toast script)
       ├─ Happy push (child_process → happy.cmd notify)
       └─ Google Chat webhook (child_process → PowerShell Invoke-RestMethod)
```

App không có background process. Khi Save Settings, app tạo **một file duy nhất** `~/.claude/claude-notify-hook.cjs` chứa toàn bộ config (đường dẫn âm thanh, webhook URL, toggle toast/happy). Hook command trong `settings.json` chỉ là:

```
node "C:/Users/.../.claude/claude-notify-hook.cjs" "stop" "Claude Code finished a task"
```

Script tự đọc stdin, phân tích event, và dispatch tới các kênh đã bật — tất cả chạy song song. Toast dùng `child_process.exec()` (cần shell context để WinRT toast hiện), các kênh khác dùng `spawn()` detached.

### So với phiên bản cũ

| v2.x (cũ) | v3.0 (hiện tại) |
|---|---|
| PowerShell inline command dài, chứa `$d=[Console]::In.ReadToEnd()` | Node.js script gọn, đọc stdin natively |
| Nhiều file wrapper riêng (happy.cjs, gchat.cjs, toast.ps1) | 1 file `claude-notify-hook.cjs` xử lý tất cả |
| PowerShell `$d` pipe bị strip khi Claude Code gọi → crash | Node.js `process.stdin` luôn nhận đúng dữ liệu |
| Có `SessionEnd` hook | Bỏ `SessionEnd` (không cần thiết) |
| Config nhúng trong command string | Config nhúng trong script file, dễ debug |
| Toast dùng `spawn` detached → không hiện | Toast dùng `exec()` qua shell → hiện đúng |

### Dữ liệu lưu trong settings.json

```json
{
  "_claudeNotifyConfig": {
    "sound_path": "C:\\Windows\\Media\\notify.wav",
    "ask_sound_path": "C:\\Windows\\Media\\Ring01.wav",
    "gchat_webhook": "https://chat.googleapis.com/...",
    "toast_enabled": true,
    "happy_enabled": false
  },
  "hooks": {
    "Stop": [ { "hooks": [{ "type": "command", "command": "node \"...hook.cjs\" \"stop\" \"...\"" }] } ],
    "PreToolUse": [ { "matcher": "AskUserQuestion", "hooks": [...] } ],
    "Notification": [ { "hooks": [...] } ],
    "PermissionRequest": [ { "hooks": [...] } ]
  }
}
```

`_claudeNotifyConfig` là nơi lưu settings UI để round-trip đáng tin cậy (không parse lại từ command string).

### File được tạo bởi app

| File | Mục đích |
|------|---------|
| `~/.claude/claude-notify-hook.cjs` | Script tổng hợp — xử lý sound, toast, happy, gchat |
| `~/.claude/claude-notify-toast.cjs` | Node.js wrapper gọi toast PS1 (dùng `exec()` thay vì `spawn` detached) |
| `~/.claude/claude-notify-happy.cjs` | Node.js wrapper gọi Happy push |

### Hook fingerprinting

Khi Save, app nhận biết hook của mình qua các chuỗi đặc trưng:
- `claude-notify-hook.cjs` → script tổng hợp mới
- `SoundPlayer` → sound hook (legacy)
- `chat.googleapis.com` → Google Chat hook (legacy)
- `ToastNotificationManager` → toast hook (legacy)
- `claude-notify-happy.cjs` → Happy hook (legacy)
- `claude-notify-gchat.cjs` → GChat hook (legacy)

Hook của app khác không chứa các chuỗi này → không bao giờ bị xóa.

### Tauri commands (Rust → JS)

| Command | Input | Output | Mô tả |
|---------|-------|--------|-------|
| `get_config` | — | `Config` | Đọc settings.json, trả về config hiện tại |
| `save_config` | `SaveConfigArgs` | `{ok: bool}` | Ghi config, tạo hook script, merge hooks |
| `test_sound` | `{path: string}` | `{ok, error?}` | Phát thử file WAV |
| `test_gchat` | `{webhook: string}` | `{ok, error?}` | Gửi test card tới Google Chat |
| `test_happy` | — | `{ok, error?}` | Gửi test push qua happy-coder |
| `test_toast` | — | `{ok, error?}` | Hiện test toast notification |
| `get_happy_status` | — | `HappyStatus` | Kiểm tra Node/happy/pair status |
| `install_happy` | — | `{ok, error?}` | Chạy `npm install -g happy-coder` |
| `pair_happy` | — | `{ok, error?}` | Mở terminal `happy auth login` |
| `launch_happy_session` | `{cwd: string}` | `{ok, error?}` | Mở happy session trong directory |
| `detect_vscode_workspaces` | — | `VscodeWorkspace[]` | Quét VSCode workspace gần đây |

### Yêu cầu hệ thống

- Windows 10/11 (64-bit)
- Claude Code đã cài đặt
- (tùy chọn) `npm install -g happy-coder` cho Happy push
- (dev) Rust/Cargo + Node.js cho build từ source
