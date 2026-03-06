use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::TrayIconBuilder,
    Manager,
};

const CREATE_NO_WINDOW: u32 = 0x08000000;

fn settings_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".claude")
        .join("settings.json")
}

fn read_settings() -> Value {
    let path = settings_path();
    match fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or(Value::Object(Default::default())),
        Err(_) => Value::Object(Default::default()),
    }
}

fn write_settings(data: &Value) {
    let path = settings_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let json = serde_json::to_string_pretty(data).unwrap_or_default();
    let _ = fs::write(&path, json);
}

fn get_happy_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join("AppData")
        .join("Roaming")
        .join("npm")
        .join("happy.cmd")
}

// ── Saved config (reliable round-trip, no command parsing) ────

#[derive(Serialize, Deserialize, Clone, Default)]
struct SavedNotifyConfig {
    #[serde(default)]
    sound_path: String,
    #[serde(default)]
    ask_sound_path: String,
    #[serde(default)]
    gchat_webhook: String,
    #[serde(default)]
    toast_enabled: bool,
    #[serde(default)]
    happy_enabled: bool,
}

impl From<&SaveConfigArgs> for SavedNotifyConfig {
    fn from(a: &SaveConfigArgs) -> Self {
        SavedNotifyConfig {
            sound_path: a.sound_path.clone(),
            ask_sound_path: a.ask_sound_path.clone(),
            gchat_webhook: a.gchat_webhook.clone(),
            toast_enabled: a.toast_enabled,
            happy_enabled: a.happy_enabled,
        }
    }
}

fn load_saved_config(settings: &Value) -> Option<SavedNotifyConfig> {
    settings
        .get("_claudeNotifyConfig")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
}

fn write_saved_config(settings: &mut Value, cfg: &SavedNotifyConfig) {
    if let Some(obj) = settings.as_object_mut() {
        if let Ok(v) = serde_json::to_value(cfg) {
            obj.insert("_claudeNotifyConfig".to_string(), v);
        }
    }
}

// ── Legacy extraction (fallback when _claudeNotifyConfig absent) ──

fn extract_wav_path(cmd: &str) -> Option<String> {
    let mut in_quote = false;
    let mut quote_char = ' ';
    let mut current = String::new();
    for ch in cmd.chars() {
        if !in_quote && (ch == '\'' || ch == '"') {
            in_quote = true;
            quote_char = ch;
            current.clear();
        } else if in_quote && ch == quote_char {
            in_quote = false;
            if current.to_lowercase().ends_with(".wav") {
                return Some(current);
            }
        } else if in_quote {
            current.push(ch);
        }
    }
    None
}

fn extract_sound_from_hooks(settings: &Value, hook_key: &str) -> Option<String> {
    let src = settings.get("hooks").or_else(|| settings.get("_hooksBackup"))?;
    let arr = src.get(hook_key)?.as_array()?;
    for entry in arr {
        // Use and_then instead of ? so a missing "hooks" field skips the entry
        if let Some(hooks) = entry.get("hooks").and_then(|h| h.as_array()) {
            for hook in hooks {
                if let Some(cmd) = hook.get("command").and_then(|c| c.as_str()) {
                    if cmd.contains("SoundPlayer") {
                        if let Some(path) = extract_wav_path(cmd) {
                            return Some(path);
                        }
                    }
                }
            }
        }
    }
    None
}

fn extract_gchat_from_hooks(settings: &Value) -> Option<String> {
    let src = settings.get("hooks").or_else(|| settings.get("_hooksBackup"))?;
    let arr = src.get("Stop")?.as_array()?;
    for entry in arr {
        // Use and_then instead of ? so a missing "hooks" field skips the entry
        if let Some(hooks) = entry.get("hooks").and_then(|h| h.as_array()) {
            for hook in hooks {
                if let Some(cmd) = hook.get("command").and_then(|c| c.as_str()) {
                    if cmd.contains("chat.googleapis.com") {
                        for part in cmd.split_whitespace() {
                            let trimmed = part.trim_matches('"').trim_matches('\'');
                            if trimmed.contains("chat.googleapis.com") && trimmed.starts_with("https://") {
                                return Some(trimmed.to_string());
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

/// Generic: returns true if any hook command in the settings satisfies `predicate`.
fn detect_cmd_in_hooks<F: Fn(&str) -> bool>(settings: &Value, predicate: F) -> bool {
    let src = match settings.get("hooks").or_else(|| settings.get("_hooksBackup")) {
        Some(v) => v,
        None => return false,
    };
    if let Some(obj) = src.as_object() {
        for (_key, arr) in obj {
            if let Some(entries) = arr.as_array() {
                for entry in entries {
                    if let Some(hooks) = entry.get("hooks").and_then(|h| h.as_array()) {
                        for hook in hooks {
                            if let Some(cmd) = hook.get("command").and_then(|c| c.as_str()) {
                                if predicate(cmd) {
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    false
}

// ── Hook fingerprinting (identify Claude Notify hooks vs others) ──

fn is_claude_notify_hook(cmd: &str) -> bool {
    cmd.contains("SoundPlayer")
        || cmd.contains("chat.googleapis.com")
        || cmd.contains("ToastNotificationManager")
        || cmd.contains("claude-notify-toast.ps1")
        || cmd.contains("claude-notify-toast.cjs")
        || cmd.contains("claude-notify-balloon.ps1")
        || (cmd.contains("happy") && cmd.contains("notify"))
}

/// Remove Claude Notify hook entries from a hook array, keeping all others.
fn filter_out_cn_entries(arr: &[Value]) -> Vec<Value> {
    let mut kept = Vec::new();
    for entry in arr {
        if let Some(hooks) = entry.get("hooks").and_then(|h| h.as_array()) {
            let non_cn: Vec<&Value> = hooks
                .iter()
                .filter(|h| {
                    h.get("command")
                        .and_then(|c| c.as_str())
                        .map(|cmd| !is_claude_notify_hook(cmd))
                        .unwrap_or(true)
                })
                .collect();
            if non_cn.is_empty() {
                continue;
            }
            let mut new_entry = entry.clone();
            if let Some(obj) = new_entry.as_object_mut() {
                obj.insert(
                    "hooks".to_string(),
                    Value::Array(non_cn.into_iter().cloned().collect()),
                );
            }
            kept.push(new_entry);
        } else {
            kept.push(entry.clone());
        }
    }
    kept
}

/// Merge Claude Notify's hook entry into an existing hook array.
fn merge_cn_entry(existing: &Value, cn_entry: Value) -> Value {
    let mut entries = match existing.as_array() {
        Some(arr) => filter_out_cn_entries(arr),
        None => Vec::new(),
    };
    entries.push(cn_entry);
    Value::Array(entries)
}

// ── Auto-start ────────────────────────────────────────────────

fn get_auto_start_enabled() -> bool {
    let output = Command::new("reg")
        .creation_flags(CREATE_NO_WINDOW)
        .args([
            "query",
            r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
            "/v",
            "ClaudeNotify",
        ])
        .output();
    match output {
        Ok(o) => String::from_utf8_lossy(&o.stdout).contains("ClaudeNotify"),
        Err(_) => false,
    }
}

fn set_auto_start(enabled: bool) {
    if enabled {
        let exe_path = std::env::current_exe()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let _ = Command::new("reg")
            .creation_flags(CREATE_NO_WINDOW)
            .args([
                "add",
                r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
                "/v",
                "ClaudeNotify",
                "/t",
                "REG_SZ",
                "/d",
                &exe_path,
                "/f",
            ])
            .output();
    } else {
        let _ = Command::new("reg")
            .creation_flags(CREATE_NO_WINDOW)
            .args([
                "delete",
                r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
                "/v",
                "ClaudeNotify",
                "/f",
            ])
            .output();
    }
}

// ── Tauri Commands ────────────────────────────────────────────

#[derive(Serialize)]
pub struct Config {
    enabled: bool,
    sound_path: String,
    ask_sound_path: String,
    gchat_webhook: String,
    auto_start: bool,
    toast_enabled: bool,
    happy_enabled: bool,
}

#[derive(Deserialize)]
pub struct SaveConfigArgs {
    enabled: bool,
    sound_path: String,
    ask_sound_path: String,
    gchat_webhook: String,
    auto_start: bool,
    toast_enabled: bool,
    happy_enabled: bool,
}

// ── Hook command builders ─────────────────────────────────────

fn sound_command(path: &str) -> Value {
    serde_json::json!({
        "type": "command",
        "command": format!(
            "powershell.exe -c \"try {{ (New-Object Media.SoundPlayer '{}').PlaySync() }} catch {{}}; exit 0\"",
            path
        )
    })
}

fn toast_command(title: &str, message: &str) -> Value {
    // Use a Node.js wrapper: reads stdin JSON for cwd, then shows balloon via PowerShell
    let script_dir = dirs::home_dir()
        .unwrap_or_default()
        .join(".claude");
    let _ = std::fs::create_dir_all(&script_dir);

    // Simple PS1 that shows a balloon tip (works without WinRT suppression issues)
    let ps_path = script_dir.join("claude-notify-balloon.ps1");
    let ps_content = r#"param([string]$Title, [string]$Message)
try {
    Add-Type -AssemblyName System.Windows.Forms
    $b = New-Object System.Windows.Forms.NotifyIcon
    $b.Icon = [System.Drawing.SystemIcons]::Information
    $b.BalloonTipTitle = $Title
    $b.BalloonTipText = $Message
    $b.Visible = $true
    $b.ShowBalloonTip(6000)
    Start-Sleep -Seconds 5
    $b.Dispose()
} catch {}
exit 0
"#;
    let _ = std::fs::write(&ps_path, ps_content);

    // Node wrapper reads stdin to extract project name, then spawns PS balloon
    let wrapper_path = script_dir.join("claude-notify-toast.cjs");
    let ps_path_fwd = ps_path.to_string_lossy().replace('\\', "/");
    let wrapper_content = format!(
        r#"'use strict';
let d='';
process.stdin.setEncoding('utf8');
process.stdin.on('data',c=>d+=c);
process.stdin.on('end',()=>{{
  let p='';
  try{{ const j=JSON.parse(d); if(j.cwd) p=require('path').basename(j.cwd); }}catch{{}}
  const msg = p ? `${{process.argv[3]}} - ${{p}}` : process.argv[3];
  require('child_process').spawn('powershell.exe',
    ['-WindowStyle','Hidden','-ExecutionPolicy','Bypass','-File',
     '{}','-Title',process.argv[2],'-Message',msg],
    {{stdio:'ignore',detached:true}}).unref();
}});
setTimeout(()=>process.exit(0),3000);
"#,
        ps_path_fwd
    );
    let _ = std::fs::write(&wrapper_path, wrapper_content);

    let cmd = format!(
        "node \"{}\" \"{}\" \"{}\"",
        wrapper_path.to_string_lossy().replace('\\', "/"),
        title,
        message
    );
    serde_json::json!({ "type": "command", "command": cmd })
}

fn gchat_card_json(title: &str, subtitle: &str, icon_url: &str, icon: &str) -> String {
    format!(
        r#"{{"cardsV2":[{{"cardId":"claude-notify","card":{{"header":{{"title":"{}","subtitle":"{}","imageUrl":"{}","imageType":"CIRCLE"}},"sections":[{{"widgets":[{{"decoratedText":{{"startIcon":{{"knownIcon":"{}"}},"text":"{}"}}}}]}}]}}}}]}}"#,
        title, subtitle, icon_url, icon, subtitle
    )
}

fn gchat_command(webhook: &str, title: &str, subtitle: &str, icon_url: &str, icon: &str) -> Value {
    let json_body = gchat_card_json(title, subtitle, icon_url, icon);
    let ps_cmd = format!(
        "try {{ Invoke-RestMethod -Uri '{}' -Method POST -ContentType 'application/json' -Body '{}' }} catch {{}}; exit 0",
        webhook, json_body
    );
    serde_json::json!({
        "type": "command",
        "command": format!("powershell.exe -c \"{}\"", ps_cmd)
    })
}

fn happy_command(title: &str, message: &str) -> Value {
    let cmd = format!(
        "cmd /c \"\"{}\" notify -t \"{}\" -p \"{}\"\"",
        get_happy_path().to_string_lossy(),
        title,
        message
    );
    serde_json::json!({ "type": "command", "command": cmd })
}

fn build_cn_hooks(
    sound: &str,
    webhook: &str,
    toast: bool,
    happy: bool,
    happy_title: &str,
    happy_msg: &str,
    gchat_title: &str,
    gchat_subtitle: &str,
    gchat_icon_url: &str,
    gchat_icon: &str,
    toast_msg: &str,
) -> Vec<Value> {
    let mut hooks = vec![sound_command(sound)];
    if !webhook.is_empty() {
        hooks.push(gchat_command(webhook, gchat_title, gchat_subtitle, gchat_icon_url, gchat_icon));
    }
    if toast {
        hooks.push(toast_command("Claude Code", toast_msg));
    }
    if happy {
        hooks.push(happy_command(happy_title, happy_msg));
    }
    hooks
}

fn build_stop_entry(sound: &str, webhook: &str, toast: bool, happy: bool) -> Value {
    let hooks = build_cn_hooks(
        sound, webhook, toast, happy,
        "Claude Code", "Task finished",
        "Task Finished", "Claude Code finished a task",
        "https://cdn.jsdelivr.net/gh/twitter/twemoji@14.0.2/assets/72x72/2705.png",
        "BOOKMARK", "Claude Code finished a task",
    );
    serde_json::json!({ "hooks": hooks })
}

fn build_pre_tool_use_entry(sound: &str, webhook: &str, toast: bool, happy: bool) -> Value {
    let hooks = build_cn_hooks(
        sound, webhook, toast, happy,
        "Claude Code", "Asking a question",
        "Question", "Claude Code is asking a question",
        "https://cdn.jsdelivr.net/gh/twitter/twemoji@14.0.2/assets/72x72/2753.png",
        "PERSON", "Claude Code is asking a question",
    );
    serde_json::json!({ "matcher": "AskUserQuestion", "hooks": hooks })
}

fn build_notification_entry(sound: &str, webhook: &str, toast: bool, happy: bool) -> Value {
    let hooks = build_cn_hooks(
        sound, webhook, toast, happy,
        "Claude Code", "Needs attention",
        "Attention", "Claude Code needs attention",
        "https://cdn.jsdelivr.net/gh/twitter/twemoji@14.0.2/assets/72x72/1f514.png",
        "DESCRIPTION", "Claude Code needs attention",
    );
    serde_json::json!({ "hooks": hooks })
}

// ── Tauri command handlers ────────────────────────────────────

#[tauri::command]
fn get_config() -> Config {
    let s = read_settings();

    // Prefer saved config, fall back to legacy command parsing
    if let Some(saved) = load_saved_config(&s) {
        return Config {
            enabled: s.get("hooks").is_some(),
            sound_path: saved.sound_path,
            ask_sound_path: saved.ask_sound_path,
            gchat_webhook: saved.gchat_webhook,
            auto_start: get_auto_start_enabled(),
            toast_enabled: saved.toast_enabled,
            happy_enabled: saved.happy_enabled,
        };
    }

    // Legacy fallback
    Config {
        enabled: s.get("hooks").is_some(),
        sound_path: extract_sound_from_hooks(&s, "Stop")
            .unwrap_or_else(|| "C:\\Windows\\Media\\notify.wav".to_string()),
        ask_sound_path: extract_sound_from_hooks(&s, "PreToolUse")
            .unwrap_or_else(|| "C:\\Windows\\Media\\Ring01.wav".to_string()),
        gchat_webhook: extract_gchat_from_hooks(&s).unwrap_or_default(),
        auto_start: get_auto_start_enabled(),
        toast_enabled: detect_cmd_in_hooks(&s, |cmd| cmd.contains("ToastNotificationManager")),
        happy_enabled: detect_cmd_in_hooks(&s, |cmd| cmd.contains("happy") && cmd.contains("notify")),
    }
}

#[tauri::command]
fn save_config(args: SaveConfigArgs) -> Value {
    let mut s = read_settings();

    set_auto_start(args.auto_start);

    // Save our own config for reliable round-trip
    write_saved_config(&mut s, &SavedNotifyConfig::from(&args));

    if args.enabled {
        // Ensure hooks object exists
        if s.get("hooks").is_none() {
            if let Some(obj) = s.as_object_mut() {
                obj.insert("hooks".to_string(), Value::Object(Default::default()));
            }
        }

        let stop_entry = build_stop_entry(&args.sound_path, &args.gchat_webhook, args.toast_enabled, args.happy_enabled);
        let pre_entry = build_pre_tool_use_entry(&args.ask_sound_path, &args.gchat_webhook, args.toast_enabled, args.happy_enabled);
        let notif_entry = build_notification_entry(&args.ask_sound_path, &args.gchat_webhook, args.toast_enabled, args.happy_enabled);

        if let Some(hooks) = s.get_mut("hooks").and_then(|h| h.as_object_mut()) {
            let existing_stop = hooks.get("Stop").cloned().unwrap_or(Value::Array(vec![]));
            let existing_pre = hooks.get("PreToolUse").cloned().unwrap_or(Value::Array(vec![]));
            let existing_notif = hooks.get("Notification").cloned().unwrap_or(Value::Array(vec![]));

            hooks.insert("Stop".to_string(), merge_cn_entry(&existing_stop, stop_entry));
            hooks.insert("PreToolUse".to_string(), merge_cn_entry(&existing_pre, pre_entry));
            hooks.insert("Notification".to_string(), merge_cn_entry(&existing_notif, notif_entry));
        }
    } else {
        // Disable: remove only CN hooks, keep others
        if let Some(hooks) = s.get_mut("hooks").and_then(|h| h.as_object_mut()) {
            for key in &["Stop", "PreToolUse", "Notification"] {
                if let Some(arr) = hooks.get(*key).and_then(|v| v.as_array()) {
                    let remaining = filter_out_cn_entries(arr);
                    if remaining.is_empty() {
                        hooks.remove(*key);
                    } else {
                        hooks.insert(key.to_string(), Value::Array(remaining));
                    }
                }
            }
            // Remove hooks object entirely if empty
            if hooks.is_empty() {
                if let Some(obj) = s.as_object_mut() {
                    obj.remove("hooks");
                }
            }
        }
    }

    write_settings(&s);
    serde_json::json!({ "ok": true })
}

#[tauri::command]
fn test_sound(path: String) -> Value {
    let output = Command::new("powershell.exe")
        .creation_flags(CREATE_NO_WINDOW)
        .args([
            "-c",
            &format!("(New-Object Media.SoundPlayer '{}').PlaySync()", path),
        ])
        .output();
    match output {
        Ok(o) if o.status.success() => serde_json::json!({ "ok": true }),
        Ok(o) => serde_json::json!({
            "ok": false,
            "error": String::from_utf8_lossy(&o.stderr).to_string()
        }),
        Err(e) => serde_json::json!({ "ok": false, "error": e.to_string() }),
    }
}

#[tauri::command]
fn test_gchat(webhook: String) -> Value {
    let json_body = gchat_card_json(
        "Test",
        "Test from Claude Notify app",
        "https://cdn.jsdelivr.net/gh/twitter/twemoji@14.0.2/assets/72x72/1f9ea.png",
        "DESCRIPTION",
    );
    let ps_cmd = format!(
        "Invoke-RestMethod -Uri '{}' -Method POST -ContentType 'application/json' -Body '{}'",
        webhook, json_body
    );
    let output = Command::new("powershell.exe")
        .creation_flags(CREATE_NO_WINDOW)
        .args(["-c", &ps_cmd])
        .output();
    match output {
        Ok(o) if o.status.success() => serde_json::json!({ "ok": true }),
        Ok(o) => serde_json::json!({
            "ok": false,
            "error": String::from_utf8_lossy(&o.stderr).to_string()
        }),
        Err(e) => serde_json::json!({ "ok": false, "error": e.to_string() }),
    }
}

#[tauri::command]
fn test_happy() -> Value {
    let happy_path = get_happy_path();
    if !happy_path.exists() {
        return serde_json::json!({
            "ok": false,
            "error": "happy-coder not installed. Run: npm install -g happy-coder"
        });
    }
    let output = Command::new(&happy_path)
        .creation_flags(CREATE_NO_WINDOW)
        .args(["notify", "-t", "Test", "-p", "Test from Claude Notify"])
        .output();
    match output {
        Ok(o) if o.status.success() => serde_json::json!({ "ok": true }),
        Ok(o) => serde_json::json!({
            "ok": false,
            "error": String::from_utf8_lossy(&o.stderr).to_string()
        }),
        Err(e) => serde_json::json!({ "ok": false, "error": e.to_string() }),
    }
}

#[tauri::command]
fn test_toast() -> Value {
    let script_dir = dirs::home_dir()
        .unwrap_or_default()
        .join(".claude");
    let _ = std::fs::create_dir_all(&script_dir);
    let ps_path = script_dir.join("claude-notify-balloon.ps1");
    let ps_content = r#"param([string]$Title, [string]$Message)
try {
    Add-Type -AssemblyName System.Windows.Forms
    $b = New-Object System.Windows.Forms.NotifyIcon
    $b.Icon = [System.Drawing.SystemIcons]::Information
    $b.BalloonTipTitle = $Title
    $b.BalloonTipText = $Message
    $b.Visible = $true
    $b.ShowBalloonTip(6000)
    Start-Sleep -Seconds 5
    $b.Dispose()
} catch {
    Write-Error $_.Exception.Message
    exit 1
}
"#;
    let _ = std::fs::write(&ps_path, ps_content);
    let output = Command::new("powershell.exe")
        .args([
            "-WindowStyle", "Hidden",
            "-ExecutionPolicy", "Bypass",
            "-File", &ps_path.to_string_lossy(),
            "-Title", "Claude Code",
            "-Message", "Test notification",
        ])
        .output();
    match output {
        Ok(o) if o.status.success() => serde_json::json!({ "ok": true }),
        Ok(o) => serde_json::json!({
            "ok": false,
            "error": String::from_utf8_lossy(&o.stderr).to_string()
        }),
        Err(e) => serde_json::json!({ "ok": false, "error": e.to_string() }),
    }
}

// ── App Setup ─────────────────────────────────────────────────

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let open_item = MenuItemBuilder::with_id("open", "Open Settings").build(app)?;
            let quit_item = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
            let menu = MenuBuilder::new(app)
                .item(&open_item)
                .separator()
                .item(&quit_item)
                .build()?;

            let _tray = TrayIconBuilder::with_id("main-tray")
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("Claude Code Notifications")
                .menu(&menu)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "open" => {
                        if let Some(win) = app.get_webview_window("main") {
                            let _ = win.show();
                            let _ = win.set_focus();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::DoubleClick {
                        button: tauri::tray::MouseButton::Left,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(win) = app.get_webview_window("main") {
                            if win.is_visible().unwrap_or(false) {
                                let _ = win.hide();
                            } else {
                                let _ = win.show();
                                let _ = win.set_focus();
                            }
                        }
                    }
                })
                .build(app)?;

            if let Some(win) = app.get_webview_window("main") {
                let win_clone = win.clone();
                win.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = win_clone.hide();
                    }
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_config,
            save_config,
            test_sound,
            test_gchat,
            test_happy,
            test_toast,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
