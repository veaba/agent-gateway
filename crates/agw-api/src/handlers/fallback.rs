//! Fallback 处理器

use axum::{
    extract::{State, Json as AxumJson, Query},
    Json,
};
use chrono::Utc;

use crate::state::AppState;
use crate::error::ApiError;
use crate::types::{
    ApiResponse, FallbackResponse, UpdateFallbackRequest,
    FallbackEventResponse, FallbackStatsResponse, FallbackEventQuery,
    TriggerTypeCountResponse, ProviderPerformanceResponse,
};

/// GET /api/v1/fallback
pub async fn get_fallback(
    State(state): State<AppState>,
) -> Json<ApiResponse<FallbackResponse>> {
    let config = state.fallback_config.read().await.clone();
    Json(ApiResponse::success(FallbackResponse::from(config)))
}

/// PUT /api/v1/fallback
pub async fn update_fallback(
    State(state): State<AppState>,
    AxumJson(payload): AxumJson<UpdateFallbackRequest>,
) -> Result<Json<ApiResponse<FallbackResponse>>, ApiError> {
    let mut config = state.fallback_config.write().await;

    if let Some(enabled) = payload.enabled {
        config.enabled = enabled;
    }
    if let Some(max_attempts) = payload.max_attempts {
        config.max_attempts = max_attempts;
    }
    if let Some(priority_order) = payload.priority_order {
        config.priority_order = priority_order;
    }

    // 持久化配置
    state.config_store.save_fallback_config(&config).await?;

    let response = FallbackResponse::from(config.clone());
    Ok(Json(ApiResponse::success(response)))
}

/// GET /api/v1/fallback/events
pub async fn get_fallback_events(
    State(state): State<AppState>,
    Query(query): Query<FallbackEventQuery>,
) -> Result<Json<ApiResponse<Vec<FallbackEventResponse>>>, ApiError> {
    let events = state.sqlite_store.get_fallback_events(
        query.plan_id,
        query.provider_id,
        query.limit,
    ).await.map_err(|e| ApiError::Internal(e.to_string()))?;

    let responses: Vec<FallbackEventResponse> = events.into_iter().map(|e| FallbackEventResponse {
        id: e.id,
        request_id: e.request_id,
        triggered_at: e.triggered_at,
        trigger_code: e.trigger_code,
        trigger_type: e.trigger_type,
        source_plan_id: e.source_plan_id,
        source_provider_id: e.source_provider_id,
        target_plan_id: e.target_plan_id,
        target_provider_id: e.target_provider_id,
        attempt_index: e.attempt_index,
        protocol_converted: e.protocol_converted,
        error_message: e.error_message,
        latency_ms: e.latency_ms,
        recovered_at: e.recovered_at,
        recovery_latency_ms: e.recovery_latency_ms,
        resolved: e.resolved,
    }).collect();

    Ok(Json(ApiResponse::success(responses)))
}

/// GET /api/v1/fallback/stats
pub async fn get_fallback_stats(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<FallbackStatsResponse>>, ApiError> {
    let since = Some(Utc::now() - chrono::Duration::days(30));
    let stats = state.sqlite_store.get_fallback_stats(since).await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let response = FallbackStatsResponse {
        total_events: stats.total_events,
        total_resolved: stats.total_resolved,
        total_unresolved: stats.total_unresolved,
        avg_recovery_latency_ms: stats.avg_recovery_latency_ms,
        by_trigger_type: stats.by_trigger_type.into_iter().map(|t| TriggerTypeCountResponse {
            trigger_type: t.trigger_type,
            count: t.count,
        }).collect(),
    };

    Ok(Json(ApiResponse::success(response)))
}

/// GET /api/v1/fallback/performance
pub async fn get_fallback_performance(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<ProviderPerformanceResponse>>>, ApiError> {
    let since = Some(Utc::now() - chrono::Duration::days(30));
    let metrics = state.sqlite_store.get_provider_performance_metrics(since).await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    // 获取所有 plan 以补充 provider_id 和 provider_name
    let plans = state.plan_manager.load_all().await.unwrap_or_default();
    let providers = state.provider_engine.list_providers().await;

    let provider_name_map: std::collections::HashMap<String, String> = providers
        .into_iter()
        .map(|p| (p.provider_id.clone(), p.name.clone()))
        .collect();

    let plan_provider_map: std::collections::HashMap<String, String> = plans
        .iter()
        .map(|p| (p.id.clone(), p.provider_id.clone()))
        .collect();

    // 按 provider_id 聚合
    let mut aggregated: std::collections::HashMap<String, ProviderPerformanceResponse> =
        std::collections::HashMap::new();

    for m in metrics {
        let provider_id = plan_provider_map.get(&m.provider_id)
            .cloned()
            .unwrap_or_else(|| m.provider_id.clone());
        let provider_name = provider_name_map.get(&provider_id)
            .cloned()
            .unwrap_or_else(|| provider_id.clone());

        let entry = aggregated.entry(provider_id.clone()).or_insert_with(|| ProviderPerformanceResponse {
            provider_id: provider_id.clone(),
            provider_name: provider_name.clone(),
            total_requests: 0,
            fallback_events: 0,
            fallback_rate: 0.0,
            avg_latency_ms: 0.0,
            success_rate: 1.0,
            estimated_recovery_time_ms: None,
            last_fallback_at: None,
            health_score: 100.0,
        });

        entry.total_requests += m.total_requests;
        entry.fallback_events += m.fallback_events;
        entry.avg_latency_ms = (entry.avg_latency_ms + m.avg_latency_ms) / 2.0;
        entry.success_rate = (entry.success_rate + m.success_rate) / 2.0;
        entry.health_score = (entry.health_score + m.health_score) / 2.0;
    }

    // 计算 fallback_rate
    for entry in aggregated.values_mut() {
        if entry.total_requests > 0 {
            entry.fallback_rate = entry.fallback_events as f64 / entry.total_requests as f64;
        }
    }

    let mut result: Vec<ProviderPerformanceResponse> = aggregated.into_values().collect();
    result.sort_by(|a, b| b.health_score.partial_cmp(&a.health_score).unwrap());

    Ok(Json(ApiResponse::success(result)))
}
