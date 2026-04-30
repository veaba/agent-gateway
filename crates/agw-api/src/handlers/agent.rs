//! Agent 处理器

use axum::{
    extract::{State, Path, Json as AxumJson},
    Json,
};

use crate::state::AppState;
use crate::error::ApiError;
use crate::types::{
    ApiResponse, AgentListResponse, AgentResponse, AgentBindResponse,
    AgentUnbindResponse, AgentAutoConfigResponse, BindAgentRequest,
};

/// GET /api/v1/agents
/// 列出所有可用的 Agent（从所有 Provider 聚合）
pub async fn list_agents(
    State(state): State<AppState>,
) -> Json<ApiResponse<AgentListResponse>> {
    let providers = state.provider_engine.list_providers().await;
    let mut agents: Vec<AgentResponse> = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for provider in providers {
        for agent_ref in provider.supported_agents {
            if seen.insert(agent_ref.agent_id.clone()) {
                agents.push(AgentResponse::from(agent_ref));
            }
        }
    }

    Json(ApiResponse::success(AgentListResponse { agents }))
}

/// POST /api/v1/plans/:id/agents/:agent_id/bind
/// 绑定 Agent 到 Plan
pub async fn bind_agent(
    State(state): State<AppState>,
    Path((plan_id, agent_id)): Path<(String, String)>,
    AxumJson(payload): AxumJson<BindAgentRequest>,
) -> Result<Json<ApiResponse<AgentBindResponse>>, ApiError> {
    // 验证 plan 存在
    let plan = state.plan_manager.get(&plan_id).await
        .ok_or_else(|| ApiError::NotFound(format!("Plan not found: {}", plan_id)))?;

    // 验证 agent 是否被该 provider 支持
    let provider = state.provider_engine.get_provider(&plan.provider_id).await
        .ok_or_else(|| ApiError::NotFound(format!("Provider not found: {}", plan.provider_id)))?;

    if !provider.supported_agents.iter().any(|a| a.agent_id == agent_id) {
        return Err(ApiError::Validation(format!(
            "Agent '{}' is not supported by provider '{}'",
            agent_id, provider.provider_id
        )));
    }

    state.plan_manager.bind_agent(&plan_id, &agent_id).await
        .map_err(|e| ApiError::Conflict(e.to_string()))?;

    // 如果需要自动配置
    if payload.auto_config {
        let _ = state.plan_manager.auto_config_agent(&plan_id, &agent_id).await;
    }

    Ok(Json(ApiResponse::success(AgentBindResponse {
        plan_id,
        agent_id,
        bound: true,
    })))
}

/// DELETE /api/v1/plans/:id/agents/:agent_id/unbind
/// 解绑 Agent
pub async fn unbind_agent(
    State(state): State<AppState>,
    Path((plan_id, agent_id)): Path<(String, String)>,
) -> Result<Json<ApiResponse<AgentUnbindResponse>>, ApiError> {
    state.plan_manager.unbind_agent(&plan_id, &agent_id).await
        .map_err(|e| ApiError::NotFound(e.to_string()))?;

    Ok(Json(ApiResponse::success(AgentUnbindResponse {
        plan_id,
        agent_id,
        unbound: true,
    })))
}

/// POST /api/v1/plans/:id/agents/:agent_id/auto-config
/// 自动配置 Agent
pub async fn auto_config_agent(
    State(state): State<AppState>,
    Path((plan_id, agent_id)): Path<(String, String)>,
) -> Result<Json<ApiResponse<AgentAutoConfigResponse>>, ApiError> {
    let success = state.plan_manager.auto_config_agent(&plan_id, &agent_id).await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let message = if success {
        "Agent auto-configured successfully".to_string()
    } else {
        "Agent auto-configuration failed".to_string()
    };

    Ok(Json(ApiResponse::success(AgentAutoConfigResponse {
        plan_id,
        agent_id,
        success,
        message,
    })))
}
