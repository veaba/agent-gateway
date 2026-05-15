//! 网关嵌入控制
//!
//! 在 Tauri GUI 内启动/停止网关服务
//! 支持嵌入式（proxy + 管理 API）和外部服务器模式

use std::sync::Arc;
use tokio::sync::RwLock;

use agw_core::core::GatewayState;
use agw_core::core::unified_router::create_unified_app;
use agw_core::business::AgentAutoConfig;
use agw_api::handlers::create_router;
use agw_api::state::AppState;

use crate::config::{ServerConfig, ServerMode};

static GATEWAY_HANDLE: std::sync::LazyLock<RwLock<Option<GatewayHandle>>> =
    std::sync::LazyLock::new(|| RwLock::new(None));

struct GatewayHandle {
    shutdown: tokio::sync::oneshot::Sender<()>,
    listen_addr: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct GatewayStatus {
    pub running: bool,
    pub listen_addr: Option<String>,
    pub mode: ServerMode,
}

/// 启动完整服务器 (proxy + 管理 API)
#[tauri::command]
pub async fn start_full_server(listen: Option<String>) -> Result<String, String> {
    let listen_addr = listen.unwrap_or_else(|| {
        let config = ServerConfig::load();
        config.embedded_listen
    });

    let mut handle_guard = GATEWAY_HANDLE.write().await;
    if handle_guard.is_some() {
        return Err("Server is already running".to_string());
    }

    // 初始化 GatewayState (proxy)
    let gateway_state = GatewayState::new()
        .await
        .map_err(|e| format!("Failed to create gateway state: {}", e))?;
    let gateway_state = Arc::new(gateway_state);

    // 初始化 AppState (管理 API)
    let api_state = AppState::init()
        .await
        .map_err(|e| format!("Failed to create API state: {}", e))?;

    // 创建管理 API 路由
    let management_router = create_router(api_state);

    // 合并 proxy + 管理 API 路由
    let app = create_unified_app(gateway_state, management_router).await;

    let addr: std::net::SocketAddr = listen_addr
        .parse()
        .map_err(|e| format!("Invalid address {}: {}", listen_addr, e))?;

    let (tx, rx) = tokio::sync::oneshot::channel::<()>();

    tokio::spawn(async move {
        let listener = match tokio::net::TcpListener::bind(addr).await {
            Ok(l) => l,
            Err(e) => {
                tracing::error!("Failed to bind server to {}: {}", addr, e);
                return;
            }
        };
        tracing::info!("Unified server listening on {} (proxy + management API)", addr);

        let server = axum::serve(listener, app);

        tokio::select! {
            _ = server => {}
            _ = rx => {
                tracing::info!("Server shutting down");
            }
        }
    });

    *handle_guard = Some(GatewayHandle {
        shutdown: tx,
        listen_addr: listen_addr.clone(),
    });

    tracing::info!("Full server started on {}", listen_addr);
    Ok(listen_addr)
}

/// 启动网关服务 (legacy - 仅 proxy)
#[tauri::command]
pub async fn start_gateway(listen: Option<String>) -> Result<String, String> {
    let listen_addr = listen.unwrap_or_else(|| "127.0.0.1:8080".to_string());

    let mut handle_guard = GATEWAY_HANDLE.write().await;
    if handle_guard.is_some() {
        return Err("Gateway is already running".to_string());
    }

    let state = GatewayState::new()
        .await
        .map_err(|e| format!("Failed to create gateway state: {}", e))?;
    let state = Arc::new(state);
    let app = agw_core::create_app(state).await;

    let addr: std::net::SocketAddr = listen_addr
        .parse()
        .map_err(|e| format!("Invalid address {}: {}", listen_addr, e))?;

    let (tx, rx) = tokio::sync::oneshot::channel::<()>();

    tokio::spawn(async move {
        let listener = match tokio::net::TcpListener::bind(addr).await {
            Ok(l) => l,
            Err(e) => {
                tracing::error!("Failed to bind gateway to {}: {}", addr, e);
                return;
            }
        };
        tracing::info!("Gateway embedded server listening on {}", addr);

        let server = axum::serve(listener, app);

        tokio::select! {
            _ = server => {}
            _ = rx => {
                tracing::info!("Gateway server shutting down");
            }
        }
    });

    *handle_guard = Some(GatewayHandle {
        shutdown: tx,
        listen_addr: listen_addr.clone(),
    });

    tracing::info!("Gateway started on {}", listen_addr);
    Ok(listen_addr)
}

/// 停止服务器
#[tauri::command]
pub async fn stop_gateway() -> Result<(), String> {
    let mut handle_guard = GATEWAY_HANDLE.write().await;
    if let Some(handle) = handle_guard.take() {
        let _ = handle.shutdown.send(());
        tracing::info!("Server stopped");
        Ok(())
    } else {
        Err("Server is not running".to_string())
    }
}

/// 获取服务器状态
#[tauri::command]
pub async fn get_gateway_status() -> GatewayStatus {
    let config = ServerConfig::load();
    let handle_guard = GATEWAY_HANDLE.read().await;
    match handle_guard.as_ref() {
        Some(handle) => GatewayStatus {
            running: true,
            listen_addr: Some(handle.listen_addr.clone()),
            mode: config.mode,
        },
        None => GatewayStatus {
            running: false,
            listen_addr: None,
            mode: config.mode,
        },
    }
}

/// 获取服务器配置
#[tauri::command]
pub async fn get_server_config() -> ServerConfig {
    ServerConfig::load()
}

/// 设置服务器配置
#[tauri::command]
pub async fn set_server_config(config: ServerConfig) -> Result<(), String> {
    config.save().map_err(|e| e.to_string())
}

/// 自动配置 Agent 工具连接到网关
#[tauri::command]
pub async fn auto_config_agent(
    agent_id: String,
    gateway_addr: Option<String>,
) -> Result<agw_core::business::ConfigReport, String> {
    let addr = gateway_addr.unwrap_or_else(|| {
        let config = ServerConfig::load();
        config.get_endpoint()
    });
    AgentAutoConfig::configure(&agent_id, &addr)
        .await
        .map_err(|e| format!("Auto-config failed: {}", e))
}