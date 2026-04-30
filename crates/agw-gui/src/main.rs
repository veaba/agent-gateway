//! agent-gateway GUI (Tauri)

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod clipboard;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .setup(|app| {
            tracing::info!("agent-gateway GUI starting...");

            // 初始化配置目录
            let config_dir = app.path().app_config_dir()?;
            std::fs::create_dir_all(&config_dir)?;

            tracing::info!("Config directory: {:?}", config_dir);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            clipboard::check_clipboard_for_key,
            clipboard::open_browser,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn main() {
    run();
}