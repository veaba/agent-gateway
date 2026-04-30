//! 剪贴板监控命令

use tauri::AppHandle;

/// 检查剪贴板是否包含 API Key
#[tauri::command]
pub async fn check_clipboard_for_key(
    _app: AppHandle,
) -> Result<Option<String>, String> {
    // TODO: 实现剪贴板检测
    Ok(None)
}

/// 打开浏览器
#[tauri::command]
pub async fn open_browser(url: String) -> Result<(), String> {
    open::that(&url).map_err(|e| e.to_string())
}