# Claude Notify — Hướng dẫn cài đặt

Nhận thông báo trên điện thoại khi Claude Code xong task hoặc cần bạn trả lời.

---

## Bước 1 — Cài app

Double-click file **`Claude Notify_3.0.0_x64-setup.exe`** → Next → Finish.

> Không cần quyền Admin. Không cần cài Rust hay Node.js.

Sau khi cài xong, app tự khởi động. Tìm biểu tượng nhỏ ở **góc dưới phải màn hình** (cạnh đồng hồ).

---

## Bước 2 — Mở Settings

**Double-click** vào icon tray để mở cửa sổ Settings.

---

## Bước 3 — Bật thông báo

1. Bật **Enable notifications**
2. Nhấn **Save Settings**
3. **Khởi động lại Claude Code** (đóng/mở lại terminal hoặc VSCode)

Xong! Từ giờ khi Claude Code hoàn thành task → bạn sẽ nghe tiếng + thấy popup.

---

## Bước 4 — Nhận thông báo trên điện thoại (tuỳ chọn)

Nếu muốn nhận push notification thẳng vào điện thoại:

### 4a. Cài app Happy trên điện thoại
- **iOS**: App Store → tìm **"Happy - Claude Code Client"**
- **Android**: Google Play → tìm **"Happy"**

### 4b. Cài CLI trên máy tính
Cần có [Node.js](https://nodejs.org) >= 18.

```
npm install -g happy-coder
```

### 4c. Pair điện thoại (chỉ làm 1 lần)

```
happy auth login
```

Terminal hiện QR code → mở app Happy → quét QR.

### 4d. Bật trong Claude Notify

1. Mở Settings → bật **Happy push notification**
2. Nhấn nút **↑ test** → kiểm tra điện thoại có nhận không
3. Nhấn **Save Settings**

---

## Thông báo sẽ nhận khi nào?

| Lúc nào | Thông báo |
|---------|-----------|
| Claude xong task | ✅ "Done" + tên project |
| Claude hỏi bạn | ❓ "Question" + nội dung câu hỏi |
| Claude cần chú ý | 🔔 "Alert" |
| Claude xin quyền | 🔐 "Permission" |

---

## Gỡ cài đặt

Settings > Apps > tìm **Claude Notify** > Uninstall.

Trước khi gỡ: tắt **Enable notifications** → Save để dọn sạch hooks.

---

## Cần hỗ trợ?

Liên hệ **@pnguyentrong** trong team.
