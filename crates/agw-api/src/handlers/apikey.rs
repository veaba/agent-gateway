//! API Key 管理处理器

use axum::{
    extract::{State, Path},
    Json,
};

use crate::state::AppState;
use crate::types::{ApiResponse, ApiKeyTestResponse};
use crate::error::ApiError;

/// POST /api/v1/plans/:id/key/test
/// 测试 Plan 的 API Key 是否有效
pub async fn test_api_key(
    State(state): State<AppState>,
    Path(plan_id): Path<String>,
) -> Result<Json<ApiResponse<ApiKeyTestResponse>>, ApiError> {
    let plan = state.plan_manager.get(&plan_id).await
        .ok_or_else(|| ApiError::NotFound(format!("Plan not found: {}", plan_id)))?;

    let provider = state.provider_engine.get_provider(&plan.provider_id).await
        .ok_or_else(|| ApiError::NotFound(format!("Provider not found: {}", plan.provider_id)))?;

    let start = std::time::Instant::now();

    // 使用 Provider 的格式测试 API Key
    let result = test_key_with_provider(&provider.base_url, &plan.api_key, &plan.selected_model_id).await;

    let latency_ms = start.elapsed().as_millis() as u64;

    let (valid, message) = match result {
        Ok(_) => (true, "API Key is valid".to_string()),
        Err(e) => (false, e),
    };

    Ok(Json(ApiResponse::success(ApiKeyTestResponse {
        plan_id,
        provider_id: plan.provider_id.clone(),
        provider_name: provider.name.clone(),
        valid,
        message,
        latency_ms: Some(latency_ms),
    })))
}

/// 使用 Provider 的 API 测试 Key
async fn test_key_with_provider(
    base_url: &Option<String>,
    api_key: &str,
    model_id: &str,
) -> Result<(), String> {
    let base_url = base_url.as_ref()
        .ok_or_else(|| "Provider has no base URL configured".to_string())?;

    let client = reqwest::Client::new();

    // 根据 Provider 类型构造测试请求
    let url = format!("{}/v1/messages", base_url.trim_end_matches('/'));

    let response = client.post(&url)
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .json(&serde_json::json!({
            "model": model_id,
            "max_tokens": 10,
            "messages": [{"role": "user", "content": "hi"}]
        }))
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    let status = response.status();

    if status.is_success() {
        Ok(())
    } else if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
        Err("Invalid API Key".to_string())
    } else if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
        Err("Rate limit exceeded".to_string())
    } else {
        let body = response.text().await.unwrap_or_default();
        Err(format!("API error ({}): {}", status, body))
    }
}

/// GET /api/v1/plans/:id/key
/// 获取 Plan 的完整 API Key（需要特殊权限或密码验证）
/// 注意：这个端点应该添加额外的认证检查
pub async fn get_api_key(
    State(state): State<AppState>,
    Path(plan_id): Path<String>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    let plan = state.plan_manager.get(&plan_id).await
        .ok_or_else(|| ApiError::NotFound(format!("Plan not found: {}", plan_id)))?;

    // 注意：生产环境应该需要额外的验证
    // 这里简化处理，直接返回 key
    Ok(Json(ApiResponse::success(serde_json::json!({
        "plan_id": plan.id,
        "api_key": plan.api_key,
    }))))
}

/// PUT /api/v1/plans/:id/key
/// 更新 Plan 的 API Key
pub async fn update_api_key(
    State(state): State<AppState>,
    Path(plan_id): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<ApiResponse<crate::types::PlanResponse>>, ApiError> {
    let api_key = payload.get("api_key")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ApiError::Validation("api_key is required".to_string()))?
        .to_string();

    let mut plan = state.plan_manager.get(&plan_id).await
        .ok_or_else(|| ApiError::NotFound(format!("Plan not found: {}", plan_id)))?;

    // 更新 API Key（需要加密存储）
    plan.api_key = api_key;

    state.plan_manager.update(plan.clone()).await?;

    Ok(Json(ApiResponse::success(crate::types::PlanResponse::from(plan))))
}