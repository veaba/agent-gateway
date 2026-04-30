//! 套餐处理器

use axum::{
    extract::{State, Path, Json as AxumJson},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

use crate::state::AppState;
use crate::error::ApiError;
use crate::types::{ApiResponse, CreatePlanRequest, UpdatePlanRequest, PlanResponse, PlanListResponse, TestConnectionResponse};
use agw_core::model::UserPlan;

/// GET /api/v1/plans
pub async fn list_plans(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<PlanListResponse>>, ApiError> {
    let plans = state.plan_manager.load_all().await?;
    let responses: Vec<PlanResponse> = plans.into_iter().map(PlanResponse::from).collect();
    Ok(Json(ApiResponse::success(PlanListResponse { plans: responses })))
}

/// GET /api/v1/plans/{id}
pub async fn get_plan(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<PlanResponse>>, ApiError> {
    let plan = state.plan_manager.get(&id).await
        .ok_or_else(|| ApiError::NotFound(format!("Plan not found: {}", id)))?;
    Ok(Json(ApiResponse::success(PlanResponse::from(plan))))
}

/// POST /api/v1/plans
pub async fn create_plan(
    State(state): State<AppState>,
    AxumJson(payload): AxumJson<CreatePlanRequest>,
) -> Result<(StatusCode, Json<ApiResponse<PlanResponse>>), ApiError> {
    // 验证 Provider 是否存在
    let provider = state.provider_engine.get_provider(&payload.provider_id).await
        .ok_or_else(|| ApiError::Validation(format!("Provider not found: {}", payload.provider_id)))?;
    
    // 验证 Plan template 是否存在
    let _plan_template = provider.coding_plans.iter()
        .find(|p| p.plan_id == payload.plan_id)
        .ok_or_else(|| ApiError::Validation(format!("Plan template not found: {}", payload.plan_id)))?;

    // 生成套餐 ID
    let plan_id = format!("{}-{}", payload.provider_id, &Uuid::new_v4().to_string()[..8]);

    let plan = UserPlan::new(
        plan_id,
        payload.provider_id.clone(),
        payload.plan_id.clone(),
        payload.name.clone(),
        payload.api_key,
        payload.selected_model_id.clone(),
    );

    state.plan_manager.add(plan.clone()).await?;

    Ok((StatusCode::CREATED, Json(ApiResponse::success(PlanResponse::from(plan)))))
}

/// PUT /api/v1/plans/{id}
pub async fn update_plan(
    State(state): State<AppState>,
    Path(id): Path<String>,
    AxumJson(payload): AxumJson<UpdatePlanRequest>,
) -> Result<Json<ApiResponse<PlanResponse>>, ApiError> {
    let mut plan = state.plan_manager.get(&id).await
        .ok_or_else(|| ApiError::NotFound(format!("Plan not found: {}", id)))?;

    // 应用更新
    if let Some(name) = payload.name {
        plan.name = name;
    }
    if let Some(api_key) = payload.api_key {
        plan.api_key = api_key;
    }
    if let Some(model_id) = payload.selected_model_id {
        plan.selected_model_id = model_id;
    }
    if let Some(enabled) = payload.enabled {
        plan.enabled = enabled;
    }
    if let Some(priority) = payload.priority {
        plan.priority = priority;
    }
    if let Some(daily) = payload.custom_quota_daily {
        plan.custom_quota_daily = Some(daily);
    }
    if let Some(monthly) = payload.custom_quota_monthly {
        plan.custom_quota_monthly = Some(monthly);
    }
    if let Some(rpm) = payload.custom_rpm_limit {
        plan.custom_rpm_limit = Some(rpm);
    }
    if let Some(threshold) = payload.alert_threshold {
        plan.alert_threshold = Some(threshold);
    }
    if let Some(notes) = payload.notes {
        plan.notes = Some(notes);
    }

    state.plan_manager.update(plan.clone()).await?;

    Ok(Json(ApiResponse::success(PlanResponse::from(plan))))
}

/// DELETE /api/v1/plans/{id}
pub async fn delete_plan(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    state.plan_manager.get(&id).await
        .ok_or_else(|| ApiError::NotFound(format!("Plan not found: {}", id)))?;
    
    state.plan_manager.delete(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/v1/plans/{id}/default
pub async fn set_default_plan(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<PlanResponse>>, ApiError> {
    let plan = state.plan_manager.get(&id).await
        .ok_or_else(|| ApiError::NotFound(format!("Plan not found: {}", id)))?;

    state.plan_manager.set_default(&id).await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(ApiResponse::success(PlanResponse::from(plan))))
}
pub async fn test_plan(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<TestConnectionResponse>>, ApiError> {
    let plan = state.plan_manager.get(&id).await
        .ok_or_else(|| ApiError::NotFound(format!("Plan not found: {}", id)))?;

    let start = std::time::Instant::now();
    let result = state.plan_manager.test_connection(&id).await;
    let latency = start.elapsed().as_millis() as u64;

    let success = result.unwrap_or(false);
    let message = if success {
        "Connection successful".to_string()
    } else {
        "Connection failed".to_string()
    };

    Ok(Json(ApiResponse::success(TestConnectionResponse {
        plan_id: plan.id,
        success,
        message,
        latency_ms: Some(latency),
    })))
}
