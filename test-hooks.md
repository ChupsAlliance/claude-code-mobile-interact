# Test Plan: Claude Code Hooks End-to-End

Test xem khi Claude Code thực sự trigger từng hook, notification có hoạt động hay không.

**Trước khi test:** Mở app Claude Notify > Save Settings (để hooks mới SessionEnd + PermissionRequest được ghi vào settings.json).

Verify settings.json sau khi save phải có 5 hooks: Stop, PreToolUse, Notification, SessionEnd, PermissionRequest.

---

## Test 1: Stop hook

**Trigger:** Claude hoàn thành task.

**Cách test:**
1. Mở terminal
2. Chạy `claude`
3. Gõ prompt đơn giản: `what is 1+1`
4. Chờ Claude trả lời xong

**Expect:**
- [ ] Nghe sound notify.wav
- [ ] Thấy toast "Claude Code finished a task"
- [ ] Nhận happy push trên điện thoại "[project] Done"

---

## Test 2: PreToolUse (AskUserQuestion) hook

**Trigger:** Claude dùng tool AskUserQuestion để hỏi user.

**Cách test:**
1. Mở terminal, chạy `claude`
2. Gõ prompt mơ hồ buộc Claude phải hỏi lại: `refactor this` (không chỉ rõ file nào)
3. Hoặc: `I want to change something` (Claude sẽ hỏi "change what?")

**Expect:**
- [ ] Nghe sound Ring01.wav
- [ ] Thấy toast "Claude Code is asking a question"
- [ ] Nhận happy push "[project] Question"

---

## Test 3: Notification hook

**Trigger:** Claude gửi notification (ít gặp hơn, thường xảy ra khi task chạy lâu).

**Cách test:**
1. Mở terminal, chạy `claude`
2. Gõ task phức tạp chạy lâu, ví dụ: `read all files in this project and summarize each one`
3. Hoặc dùng slash command: `/notification test message`
4. Chờ xem Claude có gửi notification không

> **Note:** Hook Notification khó trigger thủ công. Có thể skip nếu không trigger được.

**Expect:**
- [ ] Nghe sound Ring01.wav
- [ ] Thấy toast "Claude Code needs attention"
- [ ] Nhận happy push "[project] Alert"

---

## Test 4: SessionEnd hook (MỚI)

**Trigger:** Claude Code session kết thúc.

**Cách test:**
1. Mở terminal, chạy `claude`
2. Gõ bất kỳ prompt gì, chờ xong
3. Thoát session bằng cách gõ `/exit` hoặc nhấn Ctrl+C
4. Session đóng lại

**Expect:**
- [ ] Nghe sound notify.wav
- [ ] Thấy toast "Claude Code session ended"
- [ ] Nhận happy push "[project] Ended"

---

## Test 5: PermissionRequest hook (MỚI)

**Trigger:** Claude cần xin phép trước khi thực hiện action.

**Cách test:**
1. Mở terminal, chạy `claude` (KHÔNG dùng `--dangerously-skip-permissions`)
2. Gõ prompt yêu cầu chạy command: `run ls in the terminal`
3. Claude sẽ hiện dialog hỏi permission trước khi chạy Bash tool
4. Lúc dialog permission hiện ra = hook đã fire

> **Note:** Nếu bạn đang dùng skipDangerousModePermissionPrompt = true, thì permission dialog sẽ không hiện. Cần tạm tắt setting đó hoặc test trong project mới.

**Expect:**
- [ ] Nghe sound Ring01.wav
- [ ] Thấy toast "Claude Code needs permission"
- [ ] Nhận happy push "[project] Permission"

---

## Checklist tổng hợp

| # | Hook | Sound | Toast | Happy Push |
|---|------|-------|-------|------------|
| 1 | Stop | [ ] | [ ] | [ ] |
| 2 | PreToolUse (Ask) | [ ] | [ ] | [ ] |
| 3 | Notification | [ ] | [ ] | [ ] |
| 4 | SessionEnd | [ ] | [ ] | [ ] |
| 5 | PermissionRequest | [ ] | [ ] | [ ] |

---

## Troubleshooting

- **Không nghe sound:** Kiểm tra file WAV path trong settings có đúng không
- **Không thấy toast:** Kiểm tra Windows Notifications settings > PowerShell phải được phép gửi notification
- **Không nhận happy push:** Chạy `happy auth status` kiểm tra paired chưa
- **Hook không fire:** Kiểm tra `~/.claude/settings.json` > mục `hooks` có đúng 5 keys không
