//! 统计数据 DTO 类型

use serde::Serialize;

/// 全局统计响应
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobalStatsResponse {
    pub total_requests: u64,
    pub total_errors: u64,
    pub success_rate: f64,
    pub avg_latency_ms: f64,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub plans_count: usize,
    pub providers_count: usize,
    pub active_agents: usize,
}

/// Plan 统计响应
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanStatsResponse {
    pub plan_id: String,
    pub plan_name: String,
    pub provider_id: String,
    pub total_requests: u64,
    pub total_errors: u64,
    pub success_rate: f64,
    pub avg_latency_ms: f64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub quota_usage: QuotaUsageStats,
}

/// 配额使用统计
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuotaUsageStats {
    pub daily_used: u64,
    pub daily_limit: Option<u64>,
    pub daily_percent: f64,
    pub monthly_used: u64,
    pub monthly_limit: Option<u64>,
    pub monthly_percent: f64,
    pub rpm_used: u32,
    pub rpm_limit: Option<u32>,
    pub rpm_percent: f64,
}

/// 使用趋势数据点
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageTrendPoint {
    pub timestamp: String,
    pub requests: u64,
    pub errors: u64,
    pub avg_latency_ms: f64,
    pub input_tokens: u64,
    pub output_tokens: u64,
}

/// 使用趋势响应
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageTrendResponse {
    pub points: Vec<UsageTrendPoint>,
    pub granularity: String,
}

/// 按 Provider 统计
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderStatsResponse {
    pub provider_id: String,
    pub provider_name: String,
    pub total_requests: u64,
    pub plans_count: usize,
    pub avg_latency_ms: f64,
    pub success_rate: f64,
}

/// 健康检查记录
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthCheckResponse {
    pub plan_id: String,
    pub status: String,
    pub response_time_ms: Option<u64>,
    pub checked_at: String,
}