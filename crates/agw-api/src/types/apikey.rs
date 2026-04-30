//! API Key 类型定义

use serde::Serialize;

/// API Key 测试响应
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiKeyTestResponse {
    pub plan_id: String,
    pub provider_id: String,
    pub provider_name: String,
    pub valid: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u64>,
}