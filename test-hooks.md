# Test Plan: Claude Notify Hooks

Test xem khi Claude Code thực sự trigger từng hook, notification có hoạt động hay không.

**Trước khi test:** Mở Claude Notify > Save Settings → reload Claude Code (đóng/mở lại terminal).

---

## Test 1: Stop hook

**Trigger:** Claude hoàn thành task.

**Cách test:**
1. Mở terminal, chạy `claude`
2. Gõ: `what is 1+1`
3. Chờ Claude trả lời xong

**Expect:**
- [ ] Nghe sound (notify.wav)
- [ ] Thấy toast "✅ Done"
- [ ] Nhận Happy push "[project] Done — ..."

---

## Test 2: PreToolUse (AskUserQuestion) hook

**Trigger:** Claude dùng tool AskUserQuestion để hỏi user.

**Cách test:**
1. Mở terminal, chạy `claude`
2. Gõ prompt mơ hồ: `refactor this` hoặc `I want to change something`
3. Claude sẽ hỏi lại bạn

**Expect:**
- [ ] Nghe sound (Ring01.wav — khác với Stop)
- [ ] Thấy toast "❓ Question"
- [ ] Nhận Happy push "[project] Question — ..."

---

## Test 3: Notification hook

**Trigger:** Claude gửi notification (ít gặp, thường khi task chạy lâu).

**Cách test:**
1. Mở terminal, chạy `claude`
2. Gõ task phức tạp: `read all files in this project and summarize each one`
3. Chờ xem Claude có gửi notification không

> **Note:** Hook này khó trigger thủ công. Có thể skip nếu không trigger được.

**Expect:**
- [ ] Nghe sound
- [ ] Thấy toast "🔔 Alert"
- [ ] Nhận Happy push "[project] Alert — ..."

---

## Test 4: PermissionRequest hook

**Trigger:** Claude cần xin phép trước khi thực hiện action.

**Cách test:**
1. Mở terminal, chạy `claude` (KHÔNG dùng `--dangerously-skip-permissions`)
2. Gõ: `run ls in the terminal`
3. Claude hiện dialog hỏi permission → hook fire

> **Note:** Nếu dùng `skipDangerousModePermissionPrompt: true`, permission dialog sẽ không hiện.

**Expect:**
- [ ] Nhận Happy push "[project] Permission — Permission for Bash"

---

## Checklist tổng hợp

| # | Hook | Sound | Toast | Happy Push |
|---|------|-------|-------|------------|
| 1 | Stop | [ ] | [ ] | [ ] |
| 2 | PreToolUse (AskUserQuestion) | [ ] | [ ] | [ ] |
| 3 | Notification | [ ] | [ ] | [ ] |
| 4 | PermissionRequest | — | — | [ ] |

---

## Troubleshooting

- **Không nghe sound:** Kiểm tra file WAV path trong Settings có đúng không
- **Không thấy toast:** Settings > System > Notifications > PowerShell phải được bật. Kiểm tra Focus Assist / Do Not Disturb
- **Không nhận Happy push:** Chạy `happy auth status` kiểm tra paired chưa
- **Hook không fire:** Kiểm tra `~/.claude/settings.json` > mục `hooks` — phải dùng format `matcher` + `hooks[]` array
- **Toast works khi test trực tiếp nhưng không works từ hook:** Đảm bảo CJS script dùng `exec()` thay vì `spawn({detached: true})` để gọi PowerShell
