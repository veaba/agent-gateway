//! 网关嵌入控制
//!
//! 在 Tauri GUI 内启动/停止网关服务

use std::sync::Arc;
use tokio::sync::RwLock;

use agw_core::core::GatewayState;
use agw_core::business::AgentAutoConfig;

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
}

/// 启动网关服务
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

/// 停止网关服务
#[tauri::command]
pub async fn stop_gateway() -> Result<(), String> {
    let mut handle_guard = GATEWAY_HANDLE.write().await;
    if let Some(handle) = handle_guard.take() {
        let _ = handle.shutdown.send(());
        tracing::info!("Gateway stopped");
        Ok(())
    } else {
        Err("Gateway is not running".to_string())
    }
}

/// 获取网关状态
#[tauri::command]
pub async fn get_gateway_status() -> GatewayStatus {
    let handle_guard = GATEWAY_HANDLE.read().await;
    match handle_guard.as_ref() {
        Some(handle) => GatewayStatus {
            running: true,
            listen_addr: Some(handle.listen_addr.clone()),
        },
        None => GatewayStatus {
            running: false,
            listen_addr: None,
        },
    }
}

/// 自动配置 Agent 工具连接到网关
#[tauri::command]
pub async fn auto_config_agent(
    agent_id: String,
    gateway_addr: Option<String>,
) -> Result<agw_core::business::ConfigReport, String> {
    let addr = gateway_addr.unwrap_or_else(|| "127.0.0.1:8080".to_string());
    AgentAutoConfig::configure(&agent_id, &addr)
        .await
        .map_err(|e| format!("Auto-config failed: {}", e))
}