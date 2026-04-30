use napi_derive::napi;
use crate::enums::*;

#[napi(object)]
pub struct ProviderOnboarding {
    pub description: String,
    pub signup_url: String,
    pub plans_comparison_url: Option<String>,
    pub get_key_url: Option<String>,
    pub setup_guide_url: Option<String>,
    pub faq_url: Option<String>,
    pub agent_setup_guides: Vec<AgentSetupGuide>,
}

#[napi(object)]
pub struct AgentSetupGuide {
    pub agent_id: String,
    pub agent_name: String,
    pub auto_config_supported: bool,
    pub auto_config_script: Option<String>,
    pub manual_steps: Vec<SetupStep>,
    pub config_file_paths: PlatformPaths,
    pub env_vars: Vec<EnvVarConfig>,
}

#[napi(object)]
pub struct SetupStep {
    pub step_number: u32,
    pub description: String,
    pub command: Option<String>,
    pub copyable_text: Option<String>,
    pub note: Option<String>,
}

#[napi(object)]
pub struct EnvVarConfig {
    pub name: String,
    pub value: String,
    pub description: String,
}

#[napi(object)]
pub struct PlatformPaths {
    pub macos: Option<String>,
    pub linux: Option<String>,
    pub windows: Option<String>,
}

#[napi(object)]
pub struct ProviderInfo {
    pub provider_id: String,
    pub name: String,
    pub description: String,
    pub logo_url: Option<String>,
    pub homepage: String,
    pub docs_url: String,
    pub get_api_key_url: Option<String>,
    pub setup_guide_url: Option<String>,
    pub api_format: ApiFormat,
    pub requires_api_key: bool,
    pub onboarding: ProviderOnboarding,
    pub coding_plans: Vec<CodingPlanInfo>,
    pub models: Vec<ModelInfo>,
    pub supported_agents: Vec<AgentRefInfo>,
    pub version: String,
}

#[napi(object)]
pub struct CodingPlanInfo {
    pub plan_id: String,
    pub name: String,
    pub description: String,
    pub tier: PlanTier,
    pub supported_model_ids: Vec<String>,
    pub supported_agent_ids: Vec<String>,
    pub default_model_id: String,
    pub default_agent_id: String,
    pub quota_daily: Option<u32>,
    pub quota_monthly: Option<u32>,
    pub rpm_limit: Option<u32>,
    pub price: Option<String>,
    pub features: Vec<String>,
}

#[napi(object)]
pub struct ModelInfo {
    pub model_id: String,
    pub name: String,
    pub description: Option<String>,
    pub context_length: Option<u32>,
    pub capabilities: Vec<ModelCapability>,
    pub provider_id: String,
}

#[napi(object)]
pub struct AgentRefInfo {
    pub agent_id: String,
    pub name: String,
}

#[napi(object)]
pub struct PlanInfo {
    pub id: String,
    pub provider_id: String,
    pub plan_id: String,
    pub name: String,
    pub api_key_masked: String,
    pub selected_model_id: String,
    pub bound_agents: Vec<AgentBindingInfo>,
    pub enabled: bool,
    pub priority: u32,
    pub custom_quota_daily: Option<u32>,
    pub custom_quota_monthly: Option<u32>,
    pub custom_rpm_limit: Option<u32>,
    pub alert_threshold: Option<f64>,
    pub notes: Option<String>,
    pub created_at: String,
    pub last_health_check: Option<String>,
    pub health_status: HealthStatus,
}

#[napi(object)]
pub struct AgentBindingInfo {
    pub agent_id: String,
    pub configured: bool,
    pub config_status: AgentConfigStatus,
    pub last_connected: Option<String>,
    pub error_message: Option<String>,
}

#[napi(object)]
pub struct FallbackConfigInfo {
    pub enabled: bool,
    pub max_attempts: u32,
    pub priority_order: Vec<String>,
}

#[napi(object)]
pub struct QuotaUsageInfo {
    pub plan_id: String,
    pub daily_used: f64,
    pub monthly_used: f64,
    pub rpm_used: u32,
}

#[napi(object)]
pub struct QuotaLimitsInfo {
    pub daily: Option<f64>,
    pub monthly: Option<f64>,
    pub rpm: Option<u32>,
}

#[napi(object)]
pub struct QuotaInfo {
    pub plan_id: String,
    pub usage: QuotaUsageInfo,
    pub limits: QuotaLimitsInfo,
}

#[napi(object)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

#[napi(object)]
pub struct CreatePlanInput {
    pub provider_id: String,
    pub plan_id: String,
    pub name: String,
    pub api_key: String,
    pub selected_model_id: String,
    pub custom_quota_daily: Option<u32>,
    pub custom_quota_monthly: Option<u32>,
    pub custom_rpm_limit: Option<u32>,
    pub notes: Option<String>,
}

#[napi(object)]
pub struct UpdatePlanInput {
    pub name: Option<String>,
    pub api_key: Option<String>,
    pub selected_model_id: Option<String>,
    pub enabled: Option<bool>,
    pub priority: Option<u32>,
    pub custom_quota_daily: Option<u32>,
    pub custom_quota_monthly: Option<u32>,
    pub custom_rpm_limit: Option<u32>,
    pub alert_threshold: Option<f64>,
    pub notes: Option<String>,
}

#[napi(object)]
pub struct TestConnectionResult {
    pub plan_id: String,
    pub success: bool,
    pub message: String,
    pub latency_ms: Option<f64>,
}

impl From<agw_core::ProviderTemplate> for ProviderInfo {
    fn from(p: agw_core::ProviderTemplate) -> Self {
        ProviderInfo {
            provider_id: p.provider_id,
            name: p.name,
            description: p.description,
            logo_url: p.logo_url,
            homepage: p.homepage,
            docs_url: p.docs_url,
            get_api_key_url: p.get_api_key_url,
            setup_guide_url: p.setup_guide_url,
            api_format: p.api_format.into(),
            requires_api_key: p.requires_api_key,
            onboarding: p.onboarding.into(),
            coding_plans: p.coding_plans.into_iter().map(Into::into).collect(),
            models: p.models.into_iter().map(Into::into).collect(),
            supported_agents: p.supported_agents.into_iter().map(Into::into).collect(),
            version: p.version,
        }
    }
}

impl From<agw_core::ProviderOnboarding> for ProviderOnboarding {
    fn from(o: agw_core::ProviderOnboarding) -> Self {
        ProviderOnboarding {
            description: o.description,
            signup_url: o.signup_url,
            plans_comparison_url: o.plans_comparison_url,
            get_key_url: o.get_key_url,
            setup_guide_url: o.setup_guide_url,
            faq_url: o.faq_url,
            agent_setup_guides: o.agent_setup_guides.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<agw_core::AgentSetupGuide> for AgentSetupGuide {
    fn from(g: agw_core::AgentSetupGuide) -> Self {
        AgentSetupGuide {
            agent_id: g.agent_id,
            agent_name: g.agent_name,
            auto_config_supported: g.auto_config_supported,
            auto_config_script: g.auto_config_script,
            manual_steps: g.manual_steps.into_iter().map(Into::into).collect(),
            config_file_paths: g.config_file_paths.into(),
            env_vars: g.env_vars.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<agw_core::SetupStep> for SetupStep {
    fn from(s: agw_core::SetupStep) -> Self {
        SetupStep {
            step_number: s.step_number,
            description: s.description,
            command: s.command,
            copyable_text: s.copyable_text,
            note: s.note,
        }
    }
}

impl From<agw_core::EnvVarConfig> for EnvVarConfig {
    fn from(e: agw_core::EnvVarConfig) -> Self {
        EnvVarConfig { name: e.name, value: e.value, description: e.description }
    }
}

impl From<agw_core::PlatformPaths> for PlatformPaths {
    fn from(p: agw_core::PlatformPaths) -> Self {
        PlatformPaths { macos: p.macos, linux: p.linux, windows: p.windows }
    }
}

impl From<agw_core::CodingPlanTemplate> for CodingPlanInfo {
    fn from(p: agw_core::CodingPlanTemplate) -> Self {
        CodingPlanInfo {
            plan_id: p.plan_id,
            name: p.name,
            description: p.description,
            tier: p.tier.into(),
            supported_model_ids: p.supported_model_ids,
            supported_agent_ids: p.supported_agent_ids,
            default_model_id: p.default_model_id,
            default_agent_id: p.default_agent_id,
            quota_daily: p.quota_daily.map(|v| v as u32),
            quota_monthly: p.quota_monthly.map(|v| v as u32),
            rpm_limit: p.rpm_limit,
            price: p.price,
            features: p.features,
        }
    }
}

impl From<agw_core::ModelTemplate> for ModelInfo {
    fn from(m: agw_core::ModelTemplate) -> Self {
        ModelInfo {
            model_id: m.model_id,
            name: m.name,
            description: m.description,
            context_length: m.context_length.map(|v| v as u32),
            capabilities: m.capabilities.into_iter().map(Into::into).collect(),
            provider_id: m.provider_id,
        }
    }
}

impl From<agw_core::AgentToolRef> for AgentRefInfo {
    fn from(a: agw_core::AgentToolRef) -> Self {
        AgentRefInfo { agent_id: a.agent_id, name: a.name }
    }
}

impl From<agw_core::UserPlan> for PlanInfo {
    fn from(p: agw_core::UserPlan) -> Self {
        let mask_key = |key: &str| -> String {
            if key.len() <= 8 { "****".to_string() } else { format!("{}...{}", &key[..4], &key[key.len()-4..]) }
        };
        PlanInfo {
            id: p.id,
            provider_id: p.provider_id,
            plan_id: p.plan_id,
            name: p.name,
            api_key_masked: mask_key(&p.api_key),
            selected_model_id: p.selected_model_id,
            bound_agents: p.bound_agents.into_iter().map(Into::into).collect(),
            enabled: p.enabled,
            priority: p.priority,
            custom_quota_daily: p.custom_quota_daily.map(|v| v as u32),
            custom_quota_monthly: p.custom_quota_monthly.map(|v| v as u32),
            custom_rpm_limit: p.custom_rpm_limit,
            alert_threshold: p.alert_threshold.map(|v| v as f64),
            notes: p.notes,
            created_at: p.created_at.to_rfc3339(),
            last_health_check: p.last_health_check.map(|t| t.to_rfc3339()),
            health_status: p.health_status.into(),
        }
    }
}

impl From<agw_core::AgentBinding> for AgentBindingInfo {
    fn from(b: agw_core::AgentBinding) -> Self {
        AgentBindingInfo {
            agent_id: b.agent_id,
            configured: b.configured,
            config_status: b.config_status.into(),
            last_connected: b.last_connected.map(|t| t.to_rfc3339()),
            error_message: b.error_message,
        }
    }
}

impl From<agw_core::FallbackConfig> for FallbackConfigInfo {
    fn from(c: agw_core::FallbackConfig) -> Self {
        FallbackConfigInfo {
            enabled: c.enabled,
            max_attempts: c.max_attempts,
            priority_order: c.priority_order,
        }
    }
}