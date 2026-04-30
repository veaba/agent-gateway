//! API 处理器

use axum::{http::StatusCode, Json};
use serde_json::{json, Value};

/// 健康检查
pub async fn health() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

/// 列出所有套餐
pub async fn list_plans() -> Json<Value> {
    Json(json!({
        "plans": [],
    }))
}

/// 创建套餐
pub async fn create_plan(Json(_payload): Json<Value>) -> (StatusCode, Json<Value>) {
    (StatusCode::CREATED, Json(json!({"created": true})))
}

/// 列出所有 Provider
pub async fn list_providers() -> Json<Value> {
    Json(json!({
        "providers": [],
    }))
}

/// 配额状态
pub async fn quota_status() -> Json<Value> {
    Json(json!({
        "quotas": [],
    }))
}

/// Fallback 状态
pub async fn fallback_status() -> Json<Value> {
    Json(json!({
        "enabled": true,
        "max_attempts": 3,
        "priority_order": [],
    }))
}