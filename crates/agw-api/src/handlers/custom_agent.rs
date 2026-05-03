//! 自定义 Agent 处理器

use axum::{
    extract::{State, Path, Json as AxumJson},
    Json,
};

use crate::state::AppState;
use crate::error::ApiError;
use crate::types::{
    ApiResponse, CustomAgentListResponse, CustomAgentResponse,
    CreateCustomAgentRequest, UpdateCustomAgentRequest,
};
use agw_core::business::CustomAgentUpdate;

/// GET /api/v1/custom-agents
/// 列出所有自定义 Agent
pub async fn list_custom_agents(
    State(state): State<AppState>,
) -> Json<ApiResponse<CustomAgentListResponse>> {
    let agents = state.custom_agent_manager.list().await
        .unwrap_or_else(|e| {
            tracing::warn!("Failed to load custom agents: {}", e);
            Vec::new()
        });

    let responses = agents.into_iter()
        .map(CustomAgentResponse::from)
        .collect();

    Json(ApiResponse::success(CustomAgentListResponse { custom_agents: responses }))
}

/// POST /api/v1/custom-agents
/// 创建自定义 Agent
pub async fn create_custom_agent(
    State(state): State<AppState>,
    AxumJson(payload): AxumJson<CreateCustomAgentRequest>,
) -> Result<Json<ApiResponse<CustomAgentResponse>>, ApiError> {
    // 验证必填字段
    if payload.agent_id.trim().is_empty() {
        return Err(ApiError::Validation("agent_id is required".to_string()));
    }
    if payload.name.trim().is_empty() {
        return Err(ApiError::Validation("name is required".to_string()));
    }
    if payload.version.trim().is_empty() {
        return Err(ApiError::Validation("version is required".to_string()));
    }

    let agent = state.custom_agent_manager.create(
        payload.agent_id.trim().to_string(),
        payload.name.trim().to_string(),
        payload.version.trim().to_string(),
        payload.logo_url,
        payload.description,
    ).await.map_err(|e| {
        if e.to_string().contains("already exists") {
            ApiError::Conflict(e.to_string())
        } else {
            ApiError::Internal(e.to_string())
        }
    })?;

    Ok(Json(ApiResponse::success(CustomAgentResponse::from(agent))))
}

/// GET /api/v1/custom-agents/:id
/// 获取单个自定义 Agent
pub async fn get_custom_agent(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<CustomAgentResponse>>, ApiError> {
    let agent = state.custom_agent_manager.get(&id).await
        .ok_or_else(|| ApiError::NotFound(format!("Custom agent not found: {}", id)))?;

    Ok(Json(ApiResponse::success(CustomAgentResponse::from(agent))))
}

/// PUT /api/v1/custom-agents/:id
/// 更新自定义 Agent
pub async fn update_custom_agent(
    State(state): State<AppState>,
    Path(id): Path<String>,
    AxumJson(payload): AxumJson<UpdateCustomAgentRequest>,
) -> Result<Json<ApiResponse<CustomAgentResponse>>, ApiError> {
    let updates = CustomAgentUpdate {
        name: payload.name.map(|n| n.trim().to_string()),
        version: payload.version.map(|v| v.trim().to_string()),
        logo_url: payload.logo_url,
        description: payload.description,
    };

    let agent = state.custom_agent_manager.update(&id, updates).await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(ApiResponse::success(CustomAgentResponse::from(agent))))
}

/// DELETE /api/v1/custom-agents/:id
/// 删除自定义 Agent
pub async fn delete_custom_agent(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<()>>, ApiError> {
    state.custom_agent_manager.delete(&id).await
        .map_err(|e| ApiError::NotFound(e.to_string()))?;

    Ok(Json(ApiResponse::success(())))
}