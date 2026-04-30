//! API 路由

use axum::{
    routing::{get, post},
    Router,
};

use super::handlers;

/// 创建 API 路由
#[allow(dead_code)]
pub fn create_router() -> Router {
    Router::new()
        .route("/health", get(handlers::health))
        .route("/api/v1/plans", get(handlers::list_plans))
        .route("/api/v1/plans", post(handlers::create_plan))
        .route("/api/v1/providers", get(handlers::list_providers))
        .route("/api/v1/quota", get(handlers::quota_status))
        .route("/api/v1/fallback", get(handlers::fallback_status))
}