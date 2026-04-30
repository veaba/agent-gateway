//! Fallback 故障转移 DTO 类型

use serde::{Deserialize, Serialize};

use agw_core::model::FallbackConfig;

/// Fallback 响应
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FallbackResponse {
    pub enabled: bool,
    pub max_attempts: u32,
    pub priority_order: Vec<String>,
}

/// Fallback 更新请求
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateFallbackRequest {
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub max_attempts: Option<u32>,
    #[serde(default)]
    pub priority_order: Option<Vec<String>>,
}

impl From<FallbackConfig> for FallbackResponse {
    fn from(config: FallbackConfig) -> Self {
        Self {
            enabled: config.enabled,
            max_attempts: config.max_attempts,
            priority_order: config.priority_order,
        }
    }
}

impl From<FallbackResponse> for FallbackConfig {
    fn from(resp: FallbackResponse) -> Self {
        Self {
            enabled: resp.enabled,
            max_attempts: resp.max_attempts,
            priority_order: resp.priority_order,
        }
    }
}

/// Fallback 事件响应
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FallbackEventResponse {
    pub id: i64,
    pub request_id: String,
    pub triggered_at: String,
    pub trigger_code: Option<i32>,
    pub trigger_type: String,
    pub source_plan_id: String,
    pub source_provider_id: Option<String>,
    pub target_plan_id: Option<String>,
    pub target_provider_id: Option<String>,
    pub attempt_index: i32,
    pub protocol_converted: bool,
    pub error_message: Option<String>,
    pub latency_ms: Option<i64>,
    pub recovered_at: Option<String>,
    pub recovery_latency_ms: Option<i64>,
    pub resolved: bool,
}

/// Fallback 统计响应
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FallbackStatsResponse {
    pub total_events: i64,
    pub total_resolved: i64,
    pub total_unresolved: i64,
    pub avg_recovery_latency_ms: Option<f64>,
    pub by_trigger_type: Vec<TriggerTypeCountResponse>,
}

/// 触发类型统计
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TriggerTypeCountResponse {
    pub trigger_type: String,
    pub count: i64,
}

/// Provider 性能指标响应
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderPerformanceResponse {
    pub provider_id: String,
    pub provider_name: String,
    pub total_requests: i64,
    pub fallback_events: i64,
    pub fallback_rate: f64,
    pub avg_latency_ms: f64,
    pub success_rate: f64,
    pub estimated_recovery_time_ms: Option<i64>,
    pub last_fallback_at: Option<String>,
    pub health_score: f64,
}

/// Fallback 事件查询参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FallbackEventQuery {
    #[serde(default)]
    pub plan_id: Option<String>,
    #[serde(default)]
    pub provider_id: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_limit() -> i64 {
    100
}
