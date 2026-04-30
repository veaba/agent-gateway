//! Provider 引擎

use std::sync::Arc;
use dashmap::DashMap;

use crate::model::{ProviderTemplate, CodingPlanTemplate};

/// Provider 内置模板
#[allow(dead_code)]
pub struct ProviderEngine {
    /// 内置 Provider 模板
    builtin: Arc<DashMap<String, ProviderTemplate>>,
    /// 用户自定义 Provider
    custom: Arc<DashMap<String, ProviderTemplate>>,
    /// 远程 registry URL
    registry_url: Option<String>,
    /// 本地版本
    local_version: String,
}

impl ProviderEngine {
    /// 创建新的 Provider 引擎
    pub fn new() -> Self {
        let engine = Self {
            builtin: Arc::new(DashMap::new()),
            custom: Arc::new(DashMap::new()),
            registry_url: None,
            local_version: "0.1.0".to_string(),
        };
        engine.load_builtin_providers();
        engine
    }

    /// 获取所有可用 Provider
    pub async fn list_providers(&self) -> Vec<ProviderTemplate> {
        let mut all = Vec::new();
        all.extend(self.builtin.iter().map(|r| r.value().clone()));
        all.extend(self.custom.iter().map(|r| r.value().clone()));
        all
    }

    /// 获取指定 Provider
    pub async fn get_provider(&self, provider_id: &str) -> Option<ProviderTemplate> {
        self.builtin
            .get(provider_id)
            .map(|p| p.clone())
            .or_else(|| self.custom.get(provider_id).map(|p| p.clone()))
    }

    /// 获取 Provider 的 Coding Plan
    pub async fn get_plan_template(
        &self,
        provider_id: &str,
        plan_id: &str,
    ) -> Option<CodingPlanTemplate> {
        let provider = self.get_provider(provider_id).await?;
        provider.coding_plans.into_iter().find(|p| p.plan_id == plan_id)
    }

    /// 获取模型模板
    pub async fn get_model_template(
        &self,
        provider_id: &str,
        model_id: &str,
    ) -> Option<crate::model::ModelTemplate> {
        let provider = self.get_provider(provider_id).await?;
        provider.models.into_iter().find(|m| m.model_id == model_id)
    }

    /// 加载内置 Provider 模板
    fn load_builtin_providers(&self) {
        // Alaya Provider
        self.builtin.insert("alaya".to_string(), crate::model::ProviderTemplate {
            provider_id: "alaya".to_string(),
            name: "Alaya".to_string(),
            description: "Alaya AI Coding Platform".to_string(),
            logo_url: None,
            homepage: "https://alaya.ai".to_string(),
            docs_url: "https://docs.alaya.ai".to_string(),
            get_api_key_url: Some("https://console.alaya.ai/settings/api-keys".to_string()),
            setup_guide_url: Some("https://docs.alaya.ai/getting-started".to_string()),
            api_format: crate::model_types::ApiFormat::Anthropic,
            base_url_template: Some("https://api.alaya.com/coding/{plan_id}".to_string()),
            base_url: None,
            requires_api_key: true,
            onboarding: crate::model::ProviderOnboarding {
                description: "Alaya 是中国团队开发的AI编程平台".to_string(),
                signup_url: "https://alaya.ai/signup".to_string(),
                plans_comparison_url: Some("https://alaya.ai/pricing".to_string()),
                get_key_url: Some("https://console.alaya.ai/settings/api-keys".to_string()),
                setup_guide_url: Some("https://docs.alaya.ai/getting-started".to_string()),
                faq_url: None,
                agent_setup_guides: vec![],
            },
            coding_plans: vec![
                crate::model::CodingPlanTemplate {
                    plan_id: "alaya-lite".to_string(),
                    name: "Lite".to_string(),
                    description: "轻量版，适合个人日常开发".to_string(),
                    tier: crate::model_types::PlanTier::Free,
                    supported_model_ids: vec!["minimax-2.5".to_string()],
                    supported_agent_ids: vec!["claude-code".to_string()],
                    default_model_id: "minimax-2.5".to_string(),
                    default_agent_id: "claude-code".to_string(),
                    quota_daily: Some(100),
                    quota_monthly: Some(2000),
                    rpm_limit: Some(20),
                    price: Some("免费".to_string()),
                    features: vec!["基础代码生成".to_string()],
                },
                crate::model::CodingPlanTemplate {
                    plan_id: "alaya-plus".to_string(),
                    name: "Plus".to_string(),
                    description: "进阶版，适合专业开发者".to_string(),
                    tier: crate::model_types::PlanTier::Pro,
                    supported_model_ids: vec!["minimax-2.5".to_string(), "glm-5".to_string()],
                    supported_agent_ids: vec!["claude-code".to_string(), "kimi-cli".to_string()],
                    default_model_id: "glm-5".to_string(),
                    default_agent_id: "claude-code".to_string(),
                    quota_daily: Some(500),
                    quota_monthly: Some(10000),
                    rpm_limit: Some(60),
                    price: Some("¥29/月".to_string()),
                    features: vec!["多模型切换".to_string(), "专业开发".to_string()],
                },
                crate::model::CodingPlanTemplate {
                    plan_id: "alaya-max".to_string(),
                    name: "Max".to_string(),
                    description: "旗舰版，适合团队和高频使用".to_string(),
                    tier: crate::model_types::PlanTier::Enterprise,
                    supported_model_ids: vec!["minimax-2.5".to_string(), "glm-5".to_string(), "deepseek-v4-pro".to_string()],
                    supported_agent_ids: vec!["claude-code".to_string(), "kimi-cli".to_string(), "opencode".to_string(), "kilo-cli".to_string()],
                    default_model_id: "deepseek-v4-pro".to_string(),
                    default_agent_id: "claude-code".to_string(),
                    quota_daily: Some(2000),
                    quota_monthly: Some(50000),
                    rpm_limit: Some(200),
                    price: Some("¥99/月".to_string()),
                    features: vec!["全模型".to_string(), "全工具".to_string()],
                },
            ],
            models: vec![
                crate::model::ModelTemplate {
                    model_id: "minimax-2.5".to_string(),
                    name: "MiniMax-2.5".to_string(),
                    description: None,
                    context_length: Some(256000),
                    capabilities: vec![crate::model_types::ModelCapability::Code, crate::model_types::ModelCapability::Reasoning],
                    provider_id: "alaya".to_string(),
                },
                crate::model::ModelTemplate {
                    model_id: "glm-5".to_string(),
                    name: "GLM-5".to_string(),
                    description: None,
                    context_length: Some(128000),
                    capabilities: vec![crate::model_types::ModelCapability::Code, crate::model_types::ModelCapability::Reasoning, crate::model_types::ModelCapability::ChineseOptimized],
                    provider_id: "alaya".to_string(),
                },
            ],
            supported_agents: vec![
                crate::model::AgentToolRef { agent_id: "claude-code".to_string(), name: "Claude Code".to_string() },
                crate::model::AgentToolRef { agent_id: "kimi-cli".to_string(), name: "Kimi CLI".to_string() },
            ],
            version: "0.1.0".to_string(),
        });

        // Anthropic Provider
        self.builtin.insert("anthropic".to_string(), crate::model::ProviderTemplate {
            provider_id: "anthropic".to_string(),
            name: "Anthropic".to_string(),
            description: "Claude API 官方直连".to_string(),
            logo_url: None,
            homepage: "https://anthropic.com".to_string(),
            docs_url: "https://docs.anthropic.com".to_string(),
            get_api_key_url: Some("https://console.anthropic.com/settings/keys".to_string()),
            setup_guide_url: Some("https://docs.anthropic.com/claude-code/setup".to_string()),
            api_format: crate::model_types::ApiFormat::Anthropic,
            base_url: Some("https://api.anthropic.com".to_string()),
            base_url_template: None,
            requires_api_key: true,
            onboarding: crate::model::ProviderOnboarding {
                description: "Anthropic 官方 Claude API".to_string(),
                signup_url: "https://console.anthropic.com".to_string(),
                plans_comparison_url: None,
                get_key_url: Some("https://console.anthropic.com/settings/keys".to_string()),
                setup_guide_url: Some("https://docs.anthropic.com/claude-code/setup".to_string()),
                faq_url: None,
                agent_setup_guides: vec![],
            },
            coding_plans: vec![
                crate::model::CodingPlanTemplate {
                    plan_id: "anthropic-default".to_string(),
                    name: "Anthropic API".to_string(),
                    description: "Anthropic官方API直连".to_string(),
                    tier: crate::model_types::PlanTier::Custom,
                    supported_model_ids: vec!["claude-sonnet-4-5".to_string(), "claude-opus-4".to_string()],
                    supported_agent_ids: vec!["claude-code".to_string()],
                    default_model_id: "claude-sonnet-4-5".to_string(),
                    default_agent_id: "claude-code".to_string(),
                    quota_daily: None,
                    quota_monthly: None,
                    rpm_limit: None,
                    price: None,
                    features: vec!["官方直连".to_string()],
                },
            ],
            models: vec![
                crate::model::ModelTemplate {
                    model_id: "claude-sonnet-4-5".to_string(),
                    name: "Claude Sonnet 4.5".to_string(),
                    description: None,
                    context_length: Some(200000),
                    capabilities: vec![crate::model_types::ModelCapability::Code, crate::model_types::ModelCapability::Reasoning],
                    provider_id: "anthropic".to_string(),
                },
                crate::model::ModelTemplate {
                    model_id: "claude-opus-4".to_string(),
                    name: "Claude Opus 4".to_string(),
                    description: None,
                    context_length: Some(200000),
                    capabilities: vec![crate::model_types::ModelCapability::Code, crate::model_types::ModelCapability::Reasoning],
                    provider_id: "anthropic".to_string(),
                },
            ],
            supported_agents: vec![
                crate::model::AgentToolRef { agent_id: "claude-code".to_string(), name: "Claude Code".to_string() },
            ],
            version: "0.1.0".to_string(),
        });

        tracing::info!("Loaded {} builtin providers", self.builtin.len());
    }

    /// 检查远程更新
    pub async fn check_update(&self) -> anyhow::Result<Option<String>> {
        // TODO: 实现远程更新检查
        Ok(None)
    }
}

impl Default for ProviderEngine {
    fn default() -> Self {
        Self::new()
    }
}