//! 统一路由模块
//!
//! 合并 Proxy 路由 (GatewayState) 和 管理 API 路由 (AppState)
//! 用于嵌入式服务器模式

use axum::Router;
use std::sync::Arc;

use crate::core::{GatewayState, create_app};

/// 合并 proxy + 管理 API 路由
///
/// # Arguments
/// * `gateway_state` - Proxy 网关状态，处理 /v1/messages, /v1/chat/completions
/// * `management_router` - 管理 API 路由，处理 /api/v1/* 端点
///
/// # Returns
/// 合并后的 Router，同时支持 proxy 和管理 API
pub async fn create_unified_app(
    gateway_state: Arc<GatewayState>,
    management_router: Router,
) -> Router {
    let proxy_router = create_app(gateway_state).await;

    // 合并两个路由
    // proxy_router: /health, /v1/messages, /v1/chat/completions
    // management_router: /api/v1/*, /health (duplicate, but axum handles it)
    proxy_router.merge(management_router)
}