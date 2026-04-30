//! 日志详情处理器

use axum::{
    extract::{State, Path, Query},
    Json,
};
use crate::state::AppState;
use crate::types::ApiResponse;
use crate::error::ApiError;
use crate::handlers::logs::{LogEntryResponse, LogsQuery};

/// GET /api/v1/logs/:id
pub async fn get_log_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<LogEntryResponse>>, ApiError> {
    // 读取日志文件查找指定 ID
    let logs = state.log_store.read(10000, 0, None, None).await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let entry = logs.into_iter()
        .find(|l| l.id == id)
        .ok_or_else(|| ApiError::NotFound(format!("Log entry not found: {}", id)))?;

    Ok(Json(ApiResponse::success(LogEntryResponse::from(entry))))
}

/// GET /api/v1/logs/export
pub async fn export_logs(
    State(state): State<AppState>,
    Query(query): Query<LogsQuery>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    let limit = query.limit.min(10000);

    let logs = state.log_store.read(limit, query.offset, query.level.as_ref().and_then(|l| {
        match l.to_uppercase().as_str() {
            "DEBUG" => Some(agw_core::storage::LogLevel::Debug),
            "INFO" => Some(agw_core::storage::LogLevel::Info),
            "WARN" | "WARNING" => Some(agw_core::storage::LogLevel::Warn),
            "ERROR" => Some(agw_core::storage::LogLevel::Error),
            _ => None,
        }
    }), query.plan_id.clone()).await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let total = state.log_store.count(
        query.level.as_ref().and_then(|l| {
            match l.to_uppercase().as_str() {
                "DEBUG" => Some(agw_core::storage::LogLevel::Debug),
                "INFO" => Some(agw_core::storage::LogLevel::Info),
                "WARN" | "WARNING" => Some(agw_core::storage::LogLevel::Warn),
                "ERROR" => Some(agw_core::storage::LogLevel::Error),
                _ => None,
            }
        }),
        query.plan_id.clone()
    ).await.unwrap_or(0);

    // 转换为导出格式
    let entries: Vec<serde_json::Value> = logs.iter().map(|l| {
        serde_json::json!({
            "id": l.id,
            "timestamp": l.timestamp,
            "level": l.level.to_string(),
            "message": l.message,
            "target": l.target,
            "planId": l.plan_id,
            "requestId": l.request_id,
            "agentId": l.agent_id,
            "modelId": l.model_id,
            "statusCode": l.status_code,
            "latencyMs": l.latency_ms,
            "error": l.error,
        })
    }).collect();

    Ok(Json(ApiResponse::success(serde_json::json!({
        "logs": entries,
        "total": total,
        "limit": limit,
        "offset": query.offset,
    }))))
}

/// GET /api/v1/logs/files
pub async fn get_log_files(
    State(state): State<AppState>,
) -> Json<ApiResponse<serde_json::Value>> {
    let files = state.log_store.get_log_files().await
        .unwrap_or_default();

    let file_list: Vec<serde_json::Value> = files.into_iter().map(|f| {
        serde_json::json!({
            "name": f.name,
            "path": f.path,
            "sizeBytes": f.size_bytes,
            "isCompressed": f.is_compressed,
            "modifiedAt": f.modified_at.to_rfc3339(),
        })
    }).collect();

    Json(ApiResponse::success(serde_json::json!({
        "files": file_list,
        "count": file_list.len(),
    })))
}