//! 剪贴板监控命令

use tauri::AppHandle;
use tauri_plugin_clipboard_manager::ClipboardExt;

/// API Key 前缀列表
const KEY_PREFIXES: &[&str] = &["sk-", "sk-ant-", "sk-proj-", "AIza", "gsk_", "kilo_"];

/// 检查剪贴板是否包含 API Key
#[tauri::command]
pub async fn check_clipboard_for_key(
    app: AppHandle,
    expected_prefix: Option<String>,
) -> Result<Option<String>, String> {
    let clipboard = app.clipboard();
    let content = clipboard.read_text().map_err(|e| e.to_string())?;

    let trimmed = content.trim().to_string();
    if trimmed.is_empty() {
        return Ok(None);
    }

    let is_key = KEY_PREFIXES.iter().any(|prefix| trimmed.starts_with(prefix));

    if is_key {
        if let Some(prefix) = expected_prefix {
            if trimmed.starts_with(&prefix) {
                return Ok(Some(trimmed));
            }
            return Ok(None);
        }
        return Ok(Some(trimmed));
    }

    Ok(None)
}

/// 打开浏览器
#[tauri::command]
pub async fn open_browser(url: String) -> Result<(), String> {
    open::that(&url).map_err(|e| e.to_string())
}