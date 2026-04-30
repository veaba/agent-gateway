//! agent-gateway GUI (Tauri)

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod clipboard;
mod gateway;
mod tray;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            tracing::info!("agent-gateway GUI starting...");

            let config_dir = app.path().app_config_dir()?;
            std::fs::create_dir_all(&config_dir)?;
            tracing::info!("Config directory: {:?}", config_dir);

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