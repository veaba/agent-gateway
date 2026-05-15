//! agent-gateway GUI (Tauri)

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod clipboard;
mod config;
mod gateway;
mod tray;
mod api_invoke;

use tauri::Manager;
use agw_core::paths;
use agw_api::state::AppState;
use crate::config::ServerConfig;

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

            // 初始化 AppState（异步）
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                match AppState::init().await {
                    Ok(state) => {
                        tracing::info!("AppState initialized successfully");
                        app_handle.manage(state);

                        // 检查是否自动启动服务器
                        let config = ServerConfig::load();
                        if config.auto_start && config.mode == crate::config::ServerMode::Embedded {
                            tracing::info!("Auto-starting embedded server...");
                            if let Err(e) = gateway::start_full_server(Some(config.embedded_listen.clone())).await {
                                tracing::error!("Failed to auto-start server: {}", e);
                            } else {
                                tracing::info!("Embedded server started on {}", config.embedded_listen);
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to initialize AppState: {}", e);
                    }
                }
            });

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
            // Gateway commands
            gateway::start_gateway,
            gateway::start_full_server,
            gateway::stop_gateway,
            gateway::get_gateway_status,
            gateway::get_server_config,
            gateway::set_server_config,
            gateway::auto_config_agent,
            // API invoke handlers (for embedded mode)
            api_invoke::fetch_plans,
            api_invoke::fetch_plan,
            api_invoke::fetch_providers,
            api_invoke::fetch_provider,
            api_invoke::fetch_fallback_config,
            api_invoke::fetch_quota_status,
            api_invoke::test_external_connection,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn main() {
    run();
}