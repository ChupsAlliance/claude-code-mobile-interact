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
        for hook in entry.get("hooks")?.as_array()? {
            let cmd = hook.get("command")?.as_str()?;
            if cmd.contains("SoundPlayer") {
                return extract_wav_path(cmd);
            }
        }
    }
    None
}

fn extract_gchat_from_hooks(settings: &Value) -> Option<String> {
    let src = settings.get("hooks").or_else(|| settings.get("_hooksBackup"))?;
    let arr = src.get("Stop")?.as_array()?;
    for entry in arr {
        for hook in entry.get("hooks")?.as_array()? {
            let cmd = hook.get("command")?.as_str()?;
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
    None
}

fn detect_toast_from_hooks(settings: &Value) -> bool {
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
                                if cmd.contains("ToastNotificationManager") {
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

fn detect_happy_from_hooks(settings: &Value) -> bool {
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
                                if cmd.contains("happy") && cmd.contains("notify") {
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
                // All hooks in this entry were CN — drop the entire entry
                continue;
            }
            // Keep the entry but only with non-CN hooks
            let mut new_entry = entry.clone();
            if let Some(obj) = new_entry.as_object_mut() {
                obj.insert(
                    "hooks".to_string(),
                    Value::Array(non_cn.into_iter().cloned().collect()),
                );
            }
            kept.push(new_entry);
        } else {
            // No hooks array — keep as-is (shouldn't happen but be safe)
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
    let cmd = format!(
        "powershell.exe -c \"try {{ [Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime] > $null; [Windows.Data.Xml.Dom.XmlDocument, Windows.Data.Xml.Dom.XmlDocument, ContentType = WindowsRuntime] > $null; $xml = [Windows.Data.Xml.Dom.XmlDocument]::new(); $xml.LoadXml('<toast><visual><binding template=''ToastGeneric''><text>{}</text><text>{}</text></binding></visual></toast>'); [Windows.UI.Notifications.ToastNotificationManager]::CreateToastNotifier('{{1AC14E77-02E7-4E5D-B744-2EB1AE5198B7}}\\WindowsPowerShell\\v1.0\\powershell.exe').Show([Windows.UI.Notifications.ToastNotification]::new($xml)) }} catch {{}}; exit 0\"",
        title, message
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
    let happy_path = dirs::home_dir()
        .unwrap_or_default()
        .join("AppData")
        .join("Roaming")
        .join("npm")
        .join("happy.cmd");
    let cmd = format!(
        "cmd /c \"\"{}\" notify -t \"{}\" -p \"{}\"\"",
        happy_path.to_string_lossy(),
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
        toast_enabled: detect_toast_from_hooks(&s),
        happy_enabled: detect_happy_from_hooks(&s),
    }
}

#[tauri::command]
fn save_config(args: SaveConfigArgs) -> Value {
    let mut s = read_settings();

    set_auto_start(args.auto_start);

    // Save our own config for reliable round-trip
    write_saved_config(&mut s, &SavedNotifyConfig {
        sound_path: args.sound_path.clone(),
        ask_sound_path: args.ask_sound_path.clone(),
        gchat_webhook: args.gchat_webhook.clone(),
        toast_enabled: args.toast_enabled,
        happy_enabled: args.happy_enabled,
    });

    let stop_entry = build_stop_entry(&args.sound_path, &args.gchat_webhook, args.toast_enabled, args.happy_enabled);
    let pre_entry = build_pre_tool_use_entry(&args.ask_sound_path, &args.gchat_webhook, args.toast_enabled, args.happy_enabled);
    let notif_entry = build_notification_entry(&args.ask_sound_path, &args.gchat_webhook, args.toast_enabled, args.happy_enabled);

    if args.enabled {
        // Restore from backup if needed
        if s.get("_hooksBackup").is_some() && s.get("hooks").is_none() {
            if let Some(backup) = s.get("_hooksBackup").cloned() {
                if let Some(obj) = s.as_object_mut() {
                    obj.insert("hooks".to_string(), backup);
                    obj.remove("_hooksBackup");
                }
            }
        }

        // Ensure hooks object exists
        if s.get("hooks").is_none() {
            if let Some(obj) = s.as_object_mut() {
                obj.insert("hooks".to_string(), Value::Object(Default::default()));
            }
        }

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
    let happy_path = dirs::home_dir()
        .unwrap_or_default()
        .join("AppData")
        .join("Roaming")
        .join("npm")
        .join("happy.cmd");
    if !happy_path.exists() {
        return serde_json::json!({
            "ok": false,
            "error": "happy-coder not installed. Run: npm install -g happy-coder"
        });
    }
    let output = Command::new("cmd")
        .creation_flags(CREATE_NO_WINDOW)
        .args([
            "/c",
            &format!(
                "\"{}\" notify -t \"Test\" -p \"Test from Claude Notify\"",
                happy_path.to_string_lossy()
            ),
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
            // Build tray menu
            let open_item = MenuItemBuilder::with_id("open", "Open Settings").build(app)?;
            let quit_item = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
            let menu = MenuBuilder::new(app)
                .item(&open_item)
                .separator()
                .item(&quit_item)
                .build()?;

            // Build tray icon
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

            // Hide window on close instead of exiting
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
