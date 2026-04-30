//! 统计处理器

use axum::{
    extract::{State, Path, Query},
    Json,
};
use serde::Deserialize;

use crate::state::AppState;
use crate::types::{ApiResponse, GlobalStatsResponse, PlanStatsResponse, UsageTrendResponse, UsageTrendPoint, ProviderStatsResponse, HealthCheckResponse};
use crate::error::ApiError;

/// 查询参数
#[derive(Debug, Deserialize)]
pub struct StatsQuery {
    #[serde(default = "default_granularity")]
    pub granularity: String,
    #[serde(default)]
    pub from: Option<String>,
    #[serde(default)]
    pub to: Option<String>,
}

fn default_granularity() -> String {
    "hour".to_string()
}

/// GET /api/v1/stats
pub async fn get_global_stats(
    State(state): State<AppState>,
) -> Json<ApiResponse<GlobalStatsResponse>> {
    // 获取所有 plans
    let plans = state.plan_manager.load_all().await.unwrap_or_default();

    // 获取所有 providers
    let providers = state.provider_engine.list_providers().await;

    // 计算统计数据（基于日志）
    let logs = state.log_store.read(10000, 0, None, None).await.unwrap_or_default();

    let total_requests = logs.len() as u64;
    let total_errors = logs.iter().filter(|l| l.status_code.map(|s| s >= 400).unwrap_or(false)).count() as u64;
    let success_rate = if total_requests > 0 {
        (total_requests - total_errors) as f64 / total_requests as f64
    } else {
        1.0
    };

    let total_latency: u64 = logs.iter().filter_map(|l| l.latency_ms).sum();
    let avg_latency_ms = if total_requests > 0 {
        total_latency as f64 / total_requests as f64
    } else {
        0.0
    };

    // 统计 agents
    let mut active_agents = std::collections::HashSet::new();
    for plan in &plans {
        for binding in &plan.bound_agents {
            if binding.configured {
                active_agents.insert(&binding.agent_id);
            }
        }
    }

    Json(ApiResponse::success(GlobalStatsResponse {
        total_requests,
        total_errors,
        success_rate,
        avg_latency_ms,
        total_input_tokens: 0, // 需要从详细日志中获取
        total_output_tokens: 0,
        plans_count: plans.len(),
        providers_count: providers.len(),
        active_agents: active_agents.len(),
    }))
}

/// GET /api/v1/stats/providers
pub async fn get_provider_stats(
    State(state): State<AppState>,
) -> Json<ApiResponse<Vec<ProviderStatsResponse>>> {
    let plans = state.plan_manager.load_all().await.unwrap_or_default();
    let providers = state.provider_engine.list_providers().await;

    let mut stats = Vec::new();

    for provider in providers {
        let provider_plans: Vec<_> = plans.iter()
            .filter(|p| p.provider_id == provider.provider_id)
            .collect();

        // 统计该 provider 的所有请求
        let mut provider_logs = Vec::new();
        for plan in &provider_plans {
            let logs = state.log_store.read(1000, 0, None, Some(plan.id.clone())).await.unwrap_or_default();
            provider_logs.extend(logs);
        }

        let total_requests = provider_logs.len() as u64;
        let total_errors = provider_logs.iter().filter(|l| l.status_code.map(|s| s >= 400).unwrap_or(false)).count() as u64;
        let success_rate = if total_requests > 0 {
            (total_requests - total_errors) as f64 / total_requests as f64
        } else {
            1.0
        };

        let total_latency: u64 = provider_logs.iter().filter_map(|l| l.latency_ms).sum();
        let avg_latency_ms = if total_requests > 0 {
            total_latency as f64 / total_requests as f64
        } else {
            0.0
        };

        stats.push(ProviderStatsResponse {
            provider_id: provider.provider_id.clone(),
            provider_name: provider.name.clone(),
            total_requests,
            plans_count: provider_plans.len(),
            avg_latency_ms,
            success_rate,
        });
    }

    Json(ApiResponse::success(stats))
}

/// GET /api/v1/stats/:plan_id
pub async fn get_plan_stats(
    State(state): State<AppState>,
    Path(plan_id): Path<String>,
) -> Result<Json<ApiResponse<PlanStatsResponse>>, ApiError> {
    let plan = state.plan_manager.get(&plan_id).await
        .ok_or_else(|| ApiError::NotFound(format!("Plan not found: {}", plan_id)))?;

    // 获取该 plan 的日志
    let logs = state.log_store.read(1000, 0, None, Some(plan_id.clone())).await.unwrap_or_default();

    let total_requests = logs.len() as u64;
    let total_errors = logs.iter().filter(|l| l.status_code.map(|s| s >= 400).unwrap_or(false)).count() as u64;
    let success_rate = if total_requests > 0 {
        (total_requests - total_errors) as f64 / total_requests as f64
    } else {
        1.0
    };

    let total_latency: u64 = logs.iter().filter_map(|l| l.latency_ms).sum();
    let avg_latency_ms = if total_requests > 0 {
        total_latency as f64 / total_requests as f64
    } else {
        0.0
    };

    // 获取配额使用
    let record = state.quota_tracker.get_usage(&plan_id).await
        .unwrap_or_else(|| agw_core::business::quota::QuotaRecord {
            plan_id: plan_id.clone(),
            daily_used: 0,
            monthly_used: 0,
            rpm_used: 0,
            last_reset: chrono::Utc::now(),
        });

    let limits = state.quota_tracker.get_limits(&plan_id).await;

    let quota_usage = crate::types::stats::QuotaUsageStats {
        daily_used: record.daily_used,
        daily_limit: limits.as_ref().and_then(|l| l.daily),
        daily_percent: calculate_percent(record.daily_used, limits.as_ref().and_then(|l| l.daily)),
        monthly_used: record.monthly_used,
        monthly_limit: limits.as_ref().and_then(|l| l.monthly),
        monthly_percent: calculate_percent(record.monthly_used, limits.as_ref().and_then(|l| l.monthly)),
        rpm_used: record.rpm_used,
        rpm_limit: limits.as_ref().and_then(|l| l.rpm),
        rpm_percent: calculate_percent_u32(record.rpm_used, limits.as_ref().and_then(|l| l.rpm)),
    };

    let _provider = state.provider_engine.get_provider(&plan.provider_id).await;

    Ok(Json(ApiResponse::success(PlanStatsResponse {
        plan_id: plan_id.clone(),
        plan_name: plan.name.clone(),
        provider_id: plan.provider_id.clone(),
        total_requests,
        total_errors,
        success_rate,
        avg_latency_ms,
        input_tokens: 0,
        output_tokens: 0,
        quota_usage,
    })))
}

/// GET /api/v1/stats/usage
pub async fn get_usage_trend(
    State(state): State<AppState>,
    Query(query): Query<StatsQuery>,
) -> Json<ApiResponse<UsageTrendResponse>> {
    let logs = state.log_store.read(5000, 0, None, None).await.unwrap_or_default();

    // 按时间分组
    let mut hourly_stats: std::collections::HashMap<String, (u64, u64, u64, u64)> = std::collections::HashMap::new();

    for log in &logs {
        // log.timestamp is already a DateTime<Utc>
        let timestamp = &log.timestamp;

        let hour_key = match query.granularity.as_str() {
            "minute" => timestamp.format("%Y-%m-%dT%H:%M:00Z").to_string(),
            "hour" => timestamp.format("%Y-%m-%dT%H:00:00Z").to_string(),
            "day" => timestamp.format("%Y-%m-%dT00:00:00Z").to_string(),
            _ => timestamp.format("%Y-%m-%dT%H:00:00Z").to_string(),
        };

        let entry = hourly_stats.entry(hour_key).or_insert((0, 0, 0, 0));
        entry.0 += 1; // requests
        if log.status_code.map(|s| s >= 400).unwrap_or(false) {
            entry.1 += 1; // errors
        }
        entry.2 += log.latency_ms.unwrap_or(0); // latency
        entry.3 += 1; // count for avg
    }

    let mut points: Vec<UsageTrendPoint> = hourly_stats.into_iter()
        .map(|(timestamp, (requests, errors, latency, count))| UsageTrendPoint {
            timestamp,
            requests,
            errors,
            avg_latency_ms: if count > 0 { latency as f64 / count as f64 } else { 0.0 },
            input_tokens: 0,
            output_tokens: 0,
        })
        .collect();

    points.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

    Json(ApiResponse::success(UsageTrendResponse {
        points,
        granularity: query.granularity,
    }))
}

/// GET /api/v1/health/:plan_id
pub async fn get_plan_health(
    State(state): State<AppState>,
    Path(plan_id): Path<String>,
) -> Result<Json<ApiResponse<HealthCheckResponse>>, ApiError> {
    // 获取最近一次健康检查历史
    let history = state.sqlite_store.get_health_history(Some(plan_id.clone()), 1).await
        .unwrap_or_default();

    let plan = state.plan_manager.get(&plan_id).await
        .ok_or_else(|| ApiError::NotFound(format!("Plan not found: {}", plan_id)))?;

    // 如果有历史记录，使用历史记录的详细信息
    let (response_time_ms, checked_at) = if let Some(record) = history.first() {
        // Convert i64 to u64
        (record.response_time_ms.map(|v| v as u64), record.checked_at.clone())
    } else {
        (None, plan.last_health_check
            .map(|dt| dt.to_rfc3339())
            .unwrap_or_else(|| chrono::Utc::now().to_rfc3339()))
    };

    let status = plan.health_status.to_string();

    Ok(Json(ApiResponse::success(HealthCheckResponse {
        plan_id: plan_id.clone(),
        status,
        response_time_ms,
        checked_at,
    })))
}

/// POST /api/v1/health/:plan_id/check - 触发主动健康检查
pub async fn trigger_health_check(
    State(state): State<AppState>,
    Path(plan_id): Path<String>,
) -> Result<Json<ApiResponse<HealthCheckResponse>>, ApiError> {
    // 执行健康检查
    let result = state.health_checker.check_plan(&plan_id).await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(ApiResponse::success(HealthCheckResponse {
        plan_id: plan_id.clone(),
        status: result.status.to_string(),
        response_time_ms: Some(result.response_time_ms as u64),
        checked_at: chrono::Utc::now().to_rfc3339(),
    })))
}

fn calculate_percent(used: u64, limit: Option<u64>) -> f64 {
    match limit {
        Some(l) if l > 0 => (used as f64 / l as f64).min(1.0),
        _ => 0.0,
    }
}

fn calculate_percent_u32(used: u32, limit: Option<u32>) -> f64 {
    match limit {
        Some(l) if l > 0 => (used as f64 / l as f64).min(1.0),
        _ => 0.0,
    }
}