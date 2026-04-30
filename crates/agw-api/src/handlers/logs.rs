//! Logs 处理器

use axum::{
    extract::{Query, State},
    Json,
};
use serde::{Deserialize, Serialize};

use crate::state::AppState;
use crate::types::ApiResponse;
use agw_core::storage::{RequestLogEntry, LogLevel};

/// 查询参数
#[derive(Debug, Deserialize)]
pub struct LogsQuery {
    #[serde(default = "default_limit")]
    pub limit: usize,
    #[serde(default)]
    pub offset: usize,
    #[serde(default)]
    pub level: Option<String>,
    #[serde(default)]
    pub plan_id: Option<String>,
}

fn default_limit() -> usize { 100 }

/// 日志条目响应
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogEntryResponse {
    pub id: String,
    pub timestamp: String,
    pub level: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_code: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl From<RequestLogEntry> for LogEntryResponse {
    fn from(entry: RequestLogEntry) -> Self {
        Self {
            id: entry.id,
            timestamp: entry.timestamp.to_rfc3339(),
            level: entry.level.to_string(),
            message: entry.message,
            target: entry.target,
            plan_id: entry.plan_id,
            request_id: entry.request_id,
            agent_id: entry.agent_id,
            model_id: entry.model_id,
            status_code: entry.status_code,
            latency_ms: entry.latency_ms,
            error: entry.error,
        }
    }
}

/// 日志列表响应
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogsListResponse {
    pub logs: Vec<LogEntryResponse>,
    pub total: usize,
}

/// 解析日志级别
fn parse_level(level: &str) -> Option<LogLevel> {
    match level.to_uppercase().as_str() {
        "DEBUG" => Some(LogLevel::Debug),
        "INFO" => Some(LogLevel::Info),
        "WARN" | "WARNING" => Some(LogLevel::Warn),
        "ERROR" => Some(LogLevel::Error),
        _ => None,
    }
}

/// GET /api/v1/logs
pub async fn get_logs(
    State(state): State<AppState>,
    Query(query): Query<LogsQuery>,
) -> Json<ApiResponse<LogsListResponse>> {
    let limit = query.limit.min(1000).max(1);
    let offset = query.offset;

    // 解析日志级别过滤
    let level_filter = query.level.as_ref().and_then(|l| parse_level(l));

    // 读取日志
    let entries = state.log_store.read(limit, offset, level_filter, query.plan_id.clone())
        .await
        .unwrap_or_default();

    // 获取总数
    let total = state.log_store.count(level_filter, query.plan_id.clone())
        .await
        .unwrap_or(0);

    let logs: Vec<LogEntryResponse> = entries.into_iter().map(LogEntryResponse::from).collect();

    Json(ApiResponse::success(LogsListResponse {
        logs,
        total,
    }))
}
