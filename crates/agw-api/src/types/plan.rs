//! Plan 套餐 DTO 类型

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use agw_core::model::{UserPlan, AgentBinding};

/// 创建套餐请求
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePlanRequest {
    pub provider_id: String,
    pub plan_id: String,
    pub name: String,
    pub api_key: String,
    pub selected_model_id: String,
    #[serde(default)]
    pub custom_quota_daily: Option<u64>,
    #[serde(default)]
    pub custom_quota_monthly: Option<u64>,
    #[serde(default)]
    pub custom_rpm_limit: Option<u32>,
    #[serde(default)]
    pub notes: Option<String>,
}

/// 更新套餐请求
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePlanRequest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub selected_model_id: Option<String>,
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub priority: Option<u32>,
    #[serde(default)]
    pub custom_quota_daily: Option<u64>,
    #[serde(default)]
    pub custom_quota_monthly: Option<u64>,
    #[serde(default)]
    pub custom_rpm_limit: Option<u32>,
    #[serde(default)]
    pub alert_threshold: Option<f32>,
    #[serde(default)]
    pub notes: Option<String>,
}

/// 套餐列表响应
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanListResponse {
    pub plans: Vec<PlanResponse>,
}

/// 套餐响应（脱敏 API Key）
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanResponse {
    pub id: String,
    pub provider_id: String,
    pub plan_id: String,
    pub name: String,
    pub api_key_masked: String,
    pub selected_model_id: String,
    pub bound_agents: Vec<AgentBindingResponse>,
    pub enabled: bool,
    pub priority: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_quota_daily: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_quota_monthly: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_rpm_limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alert_threshold: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_health_check: Option<DateTime<Utc>>,
    pub health_status: String,
}

/// Agent 绑定响应
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentBindingResponse {
    pub agent_id: String,
    pub configured: bool,
    pub config_status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_connected: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

/// 连接测试结果
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TestConnectionResponse {
    pub plan_id: String,
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u64>,
}

/// 脱敏 API Key
fn mask_api_key(key: &str) -> String {
    if key.len() <= 8 {
        "*".repeat(key.len())
    } else {
        format!("{}...{}", &key[..4], &key[key.len() - 4..])
    }
}

impl From<UserPlan> for PlanResponse {
    fn from(plan: UserPlan) -> Self {
        Self {
            id: plan.id,
            provider_id: plan.provider_id,
            plan_id: plan.plan_id,
            name: plan.name,
            api_key_masked: mask_api_key(&plan.api_key),
            selected_model_id: plan.selected_model_id,
            bound_agents: plan.bound_agents.into_iter().map(AgentBindingResponse::from).collect(),
            enabled: plan.enabled,
            priority: plan.priority,
            custom_quota_daily: plan.custom_quota_daily,
            custom_quota_monthly: plan.custom_quota_monthly,
            custom_rpm_limit: plan.custom_rpm_limit,
            alert_threshold: plan.alert_threshold,
            notes: plan.notes,
            created_at: plan.created_at,
            last_health_check: plan.last_health_check,
            health_status: plan.health_status.to_string(),
        }
    }
}

impl From<AgentBinding> for AgentBindingResponse {
    fn from(binding: AgentBinding) -> Self {
        Self {
            agent_id: binding.agent_id,
            configured: binding.configured,
            config_status: binding.config_status.to_string(),
            last_connected: binding.last_connected,
            error_message: binding.error_message,
        }
    }
}
