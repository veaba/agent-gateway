//! agent-gateway GUI (Tauri)

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod clipboard;
mod gateway;
mod tray;

use tauri::Manager;
use agw_core::paths;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            tracing::info!("agent-gateway GUI starting...");

            // 使用统一路径模块
            let gui_dir = paths::gui_dir();
            std::fs::create_dir_all(&gui_dir)?;
            tracing::info!("GUI directory: {:?}", gui_dir);

            // 显示所有目录
            tracing::info!("Root directory: {:?}", paths::root_dir());
            tracing::info!("Core directory: {:?}", paths::core_dir());
            tracing::info!("CLI directory: {:?}", paths::cli_dir());

            tray::setup_tray(app)?;

            // Minimize to tray instead of closing
            let window = app.get_webview_window("main").unwrap();
            window.clone().on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = window.hide();
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            clipboard::check_clipboard_for_key,
            clipboard::open_browser,
            gateway::start_gateway,
            gateway::stop_gateway,
            gateway::get_gateway_status,
            gateway::auto_config_agent,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn main() {
    run();
}