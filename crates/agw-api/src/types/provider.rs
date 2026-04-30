//! Provider 服务商 DTO 类型

use serde::Serialize;

use agw_core::model::{ProviderTemplate, CodingPlanTemplate, ModelTemplate, AgentToolRef};

/// Provider 列表响应
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderListResponse {
    pub providers: Vec<ProviderResponse>,
}

/// Provider 响应
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderResponse {
    pub provider_id: String,
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logo_url: Option<String>,
    pub homepage: String,
    pub docs_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub get_api_key_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub setup_guide_url: Option<String>,
    pub api_format: String,
    pub requires_api_key: bool,
    pub coding_plans: Vec<CodingPlanSummary>,
    pub models: Vec<ModelSummary>,
    pub supported_agents: Vec<AgentSummary>,
}

/// Coding Plan 摘要
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodingPlanSummary {
    pub plan_id: String,
    pub name: String,
    pub description: String,
    pub tier: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_model_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quota_daily: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quota_monthly: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rpm_limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<String>,
    pub features: Vec<String>,
    pub supported_model_ids: Vec<String>,
    pub supported_agent_ids: Vec<String>,
}

/// 模型摘要
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelSummary {
    pub model_id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_length: Option<u64>,
    pub capabilities: Vec<String>,
    pub provider_id: String,
}

/// Agent 摘要
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentSummary {
    pub agent_id: String,
    pub name: String,
}

impl From<ProviderTemplate> for ProviderResponse {
    fn from(p: ProviderTemplate) -> Self {
        Self {
            provider_id: p.provider_id.clone(),
            name: p.name,
            description: p.description,
            logo_url: p.logo_url,
            homepage: p.homepage,
            docs_url: p.docs_url,
            get_api_key_url: p.get_api_key_url,
            setup_guide_url: p.setup_guide_url,
            api_format: p.api_format.to_string(),
            requires_api_key: p.requires_api_key,
            coding_plans: p.coding_plans.into_iter().map(CodingPlanSummary::from).collect(),
            models: p.models.into_iter().map(ModelSummary::from).collect(),
            supported_agents: p.supported_agents.into_iter().map(AgentSummary::from).collect(),
        }
    }
}

impl From<CodingPlanTemplate> for CodingPlanSummary {
    fn from(cp: CodingPlanTemplate) -> Self {
        Self {
            plan_id: cp.plan_id,
            name: cp.name,
            description: cp.description,
            tier: cp.tier.to_string(),
            default_model_id: Some(cp.default_model_id),
            quota_daily: cp.quota_daily,
            quota_monthly: cp.quota_monthly,
            rpm_limit: cp.rpm_limit,
            price: cp.price,
            features: cp.features,
            supported_model_ids: cp.supported_model_ids,
            supported_agent_ids: cp.supported_agent_ids,
        }
    }
}

impl From<ModelTemplate> for ModelSummary {
    fn from(m: ModelTemplate) -> Self {
        Self {
            model_id: m.model_id,
            name: m.name,
            description: m.description,
            context_length: m.context_length,
            capabilities: m.capabilities.iter().map(|c| c.to_string()).collect(),
            provider_id: m.provider_id,
        }
    }
}

impl From<AgentToolRef> for AgentSummary {
    fn from(a: AgentToolRef) -> Self {
        Self {
            agent_id: a.agent_id,
            name: a.name,
        }
    }
}
