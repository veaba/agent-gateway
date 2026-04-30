//! Provider 引擎

use std::sync::Arc;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use anyhow::Result;

use crate::model::{ProviderTemplate, CodingPlanTemplate};

/// 远程 Registry 索引
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryIndex {
    /// 版本号
    pub version: String,
    /// 变更日志
    pub changelog: Vec<String>,
    /// 更新时间
    pub updated_at: String,
    /// Provider 列表摘要
    pub providers: Vec<ProviderSummary>,
}

/// Provider 摘要（索引中使用）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderSummary {
    pub provider_id: String,
    pub name: String,
    pub version: String,
}

/// 更新报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateReport {
    /// 新版本号
    pub new_version: String,
    /// 变更内容
    pub changes: Vec<String>,
    /// 更新的 Provider ID 列表
    pub updated_providers: Vec<String>,
}

/// Provider 内置模板
#[allow(dead_code)]
pub struct ProviderEngine {
    /// 内置 Provider 模板
    builtin: Arc<DashMap<String, ProviderTemplate>>,
    /// 用户自定义 Provider
    custom: Arc<DashMap<String, ProviderTemplate>>,
    /// 远程 registry URL
    registry_url: String,
    /// 本地版本
    local_version: String,
}

impl ProviderEngine {
    /// 创建新的 Provider 引擎
    pub fn new() -> Self {
        Self::with_registry("https://registry.agent-gateway.dev")
    }

    /// 使用自定义 registry URL 创建
    pub fn with_registry(registry_url: &str) -> Self {
        let engine = Self {
            builtin: Arc::new(DashMap::new()),
            custom: Arc::new(DashMap::new()),
            registry_url: registry_url.to_string(),
            local_version: "0.1.0".to_string(),
        };
        engine.load_builtin_providers();
        engine
    }

    /// 设置本地版本
    pub fn set_local_version(&mut self, version: String) {
        self.local_version = version;
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
                agent_setup_guides: vec![
                    crate::model::AgentSetupGuide {
                        agent_id: "claude-code".to_string(),
                        agent_name: "Claude Code".to_string(),
                        auto_config_supported: true,
                        auto_config_script: Some("# Set environment variables for Claude Code\nexport ANTHROPIC_BASE_URL=http://127.0.0.1:8080\nexport ANTHROPIC_API_KEY=dummy".to_string()),
                        manual_steps: vec![
                            crate::model::SetupStep {
                                step_number: 1,
                                description: "设置环境变量".to_string(),
                                command: Some("export ANTHROPIC_BASE_URL=http://127.0.0.1:8080".to_string()),
                                copyable_text: Some("export ANTHROPIC_BASE_URL=http://127.0.0.1:8080".to_string()),
                                note: Some("将此行添加到 ~/.zshrc 或 ~/.bashrc 以持久化".to_string()),
                            },
                            crate::model::SetupStep {
                                step_number: 2,
                                description: "设置 API Key（任意值，网关会替换为实际Key）".to_string(),
                                command: Some("export ANTHROPIC_API_KEY=dummy".to_string()),
                                copyable_text: Some("export ANTHROPIC_API_KEY=dummy".to_string()),
                                note: Some("网关会自动使用您配置的实际 API Key".to_string()),
                            },
                            crate::model::SetupStep {
                                step_number: 3,
                                description: "启动 Claude Code".to_string(),
                                command: Some("claude".to_string()),
                                copyable_text: None,
                                note: None,
                            },
                        ],
                        config_file_paths: crate::model::PlatformPaths {
                            macos: Some("~/Library/Application Support/Claude/settings.json".to_string()),
                            linux: Some("~/.config/claude/settings.json".to_string()),
                            windows: Some("%APPDATA%\\Claude\\settings.json".to_string()),
                        },
                        env_vars: vec![
                            crate::model::EnvVarConfig {
                                name: "ANTHROPIC_BASE_URL".to_string(),
                                value: "http://127.0.0.1:8080".to_string(),
                                description: "Claude Code 网关地址".to_string(),
                            },
                            crate::model::EnvVarConfig {
                                name: "ANTHROPIC_API_KEY".to_string(),
                                value: "dummy".to_string(),
                                description: "任意值，网关会替换为实际Key".to_string(),
                            },
                        ],
                    },
                    crate::model::AgentSetupGuide {
                        agent_id: "kimi-cli".to_string(),
                        agent_name: "Kimi CLI".to_string(),
                        auto_config_supported: true,
                        auto_config_script: Some("# Create Kimi CLI config directory\nmkdir -p ~/.config/kimi\ncat > ~/.config/kimi/config.yaml << 'EOF'\napi: anthropic-messages\nbaseUrl: http://127.0.0.1:8080/v1\napiKey: dummy\nEOF".to_string()),
                        manual_steps: vec![
                            crate::model::SetupStep {
                                step_number: 1,
                                description: "创建配置目录".to_string(),
                                command: Some("mkdir -p ~/.config/kimi".to_string()),
                                copyable_text: None,
                                note: None,
                            },
                            crate::model::SetupStep {
                                step_number: 2,
                                description: "创建配置文件".to_string(),
                                command: None,
                                copyable_text: Some("api: anthropic-messages\nbaseUrl: http://127.0.0.1:8080/v1\napiKey: dummy".to_string()),
                                note: Some("保存为 ~/.config/kimi/config.yaml".to_string()),
                            },
                            crate::model::SetupStep {
                                step_number: 3,
                                description: "启动 Kimi CLI".to_string(),
                                command: Some("kimi".to_string()),
                                copyable_text: None,
                                note: None,
                            },
                        ],
                        config_file_paths: crate::model::PlatformPaths {
                            macos: Some("~/.config/kimi/config.yaml".to_string()),
                            linux: Some("~/.config/kimi/config.yaml".to_string()),
                            windows: Some("%APPDATA%\\kimi\\config.yaml".to_string()),
                        },
                        env_vars: vec![
                            crate::model::EnvVarConfig {
                                name: "KIMI_BASE_URL".to_string(),
                                value: "http://127.0.0.1:8080/v1".to_string(),
                                description: "Kimi CLI 网关地址".to_string(),
                            },
                        ],
                    },
                ],
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
                crate::model::ModelTemplate {
                    model_id: "deepseek-v4-pro".to_string(),
                    name: "DeepSeek-V4-Pro".to_string(),
                    description: None,
                    context_length: Some(128000),
                    capabilities: vec![crate::model_types::ModelCapability::Code, crate::model_types::ModelCapability::Reasoning, crate::model_types::ModelCapability::Math],
                    provider_id: "alaya".to_string(),
                },
            ],
            supported_agents: vec![
                crate::model::AgentToolRef { agent_id: "claude-code".to_string(), name: "Claude Code".to_string() },
                crate::model::AgentToolRef { agent_id: "kimi-cli".to_string(), name: "Kimi CLI".to_string() },
                crate::model::AgentToolRef { agent_id: "opencode".to_string(), name: "OpenCode".to_string() },
                crate::model::AgentToolRef { agent_id: "kilo-cli".to_string(), name: "Kilo CLI".to_string() },
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
                agent_setup_guides: vec![
                    crate::model::AgentSetupGuide {
                        agent_id: "claude-code".to_string(),
                        agent_name: "Claude Code".to_string(),
                        auto_config_supported: true,
                        auto_config_script: Some("# Set environment variables for Claude Code\nexport ANTHROPIC_BASE_URL=http://127.0.0.1:8080\nexport ANTHROPIC_API_KEY=dummy".to_string()),
                        manual_steps: vec![
                            crate::model::SetupStep {
                                step_number: 1,
                                description: "设置 Anthropic 基础URL".to_string(),
                                command: Some("export ANTHROPIC_BASE_URL=http://127.0.0.1:8080".to_string()),
                                copyable_text: Some("export ANTHROPIC_BASE_URL=http://127.0.0.1:8080".to_string()),
                                note: Some("将此行添加到 shell 配置文件以持久化".to_string()),
                            },
                            crate::model::SetupStep {
                                step_number: 2,
                                description: "设置 API Key（网关会替换为实际Key）".to_string(),
                                command: Some("export ANTHROPIC_API_KEY=dummy".to_string()),
                                copyable_text: Some("export ANTHROPIC_API_KEY=dummy".to_string()),
                                note: Some("网关会自动使用您配置的实际 API Key".to_string()),
                            },
                            crate::model::SetupStep {
                                step_number: 3,
                                description: "配置文件方式（可选）".to_string(),
                                command: None,
                                copyable_text: Some("{\n  \"anthropicBaseUrl\": \"http://127.0.0.1:8080\"\n}".to_string()),
                                note: Some("保存为 Claude 配置目录下的 settings.json".to_string()),
                            },
                            crate::model::SetupStep {
                                step_number: 4,
                                description: "启动 Claude Code".to_string(),
                                command: Some("claude".to_string()),
                                copyable_text: None,
                                note: None,
                            },
                        ],
                        config_file_paths: crate::model::PlatformPaths {
                            macos: Some("~/Library/Application Support/Claude/settings.json".to_string()),
                            linux: Some("~/.config/claude/settings.json".to_string()),
                            windows: Some("%APPDATA%\\Claude\\settings.json".to_string()),
                        },
                        env_vars: vec![
                            crate::model::EnvVarConfig {
                                name: "ANTHROPIC_BASE_URL".to_string(),
                                value: "http://127.0.0.1:8080".to_string(),
                                description: "网关代理地址".to_string(),
                            },
                            crate::model::EnvVarConfig {
                                name: "ANTHROPIC_API_KEY".to_string(),
                                value: "dummy".to_string(),
                                description: "占位Key，网关会替换为实际Key".to_string(),
                            },
                        ],
                    },
                ],
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
    pub async fn check_update(&self) -> Result<Option<UpdateReport>> {
        let index_url = format!("{}/providers/index.json", self.registry_url);

        tracing::info!("Checking provider updates from {}", index_url);

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()?;

        let response = client
            .get(&index_url)
            .send()
            .await?;

        if !response.status().is_success() {
            tracing::warn!("Failed to fetch registry index: {}", response.status());
            return Ok(None);
        }

        let remote_index: RegistryIndex = response.json().await?;

        // 检查版本是否更新
        if remote_index.version == self.local_version {
            tracing::info!("Provider templates are up to date (version: {})", self.local_version);
            return Ok(None);
        }

        tracing::info!(
            "New provider version available: {} -> {}",
            self.local_version,
            remote_index.version
        );

        Ok(Some(UpdateReport {
            new_version: remote_index.version.clone(),
            changes: remote_index.changelog.clone(),
            updated_providers: remote_index.providers.iter().map(|p| p.provider_id.clone()).collect(),
        }))
    }

    /// 应用远程更新
    pub async fn apply_update(&self) -> Result<Vec<String>> {
        let update = self.check_update().await?;

        if update.is_none() {
            tracing::info!("No updates available");
            return Ok(vec![]);
        }

        let update = update.unwrap();
        let providers_url = format!("{}/providers/latest.yaml", self.registry_url);

        tracing::info!("Downloading provider updates from {}", providers_url);

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        let response = client
            .get(&providers_url)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to download provider updates: {}", response.status());
        }

        let yaml_content = response.text().await?;

        // 解析 YAML
        let updated_providers: Vec<ProviderTemplate> = serde_yaml::from_str(&yaml_content)?;

        let mut updated_ids = Vec::new();
        for provider in updated_providers {
            // 只更新内置 Provider，不覆盖自定义 Provider
            if self.custom.contains_key(&provider.provider_id) {
                tracing::info!(
                    "Skipping custom provider {} (user-defined)",
                    provider.provider_id
                );
                continue;
            }

            let provider_id = provider.provider_id.clone();
            self.builtin.insert(provider_id.clone(), provider);
            updated_ids.push(provider_id.clone());
            tracing::info!("Updated provider: {}", provider_id);
        }

        tracing::info!(
            "Applied {} provider updates (version: {})",
            updated_ids.len(),
            update.new_version
        );

        Ok(updated_ids)
    }

    /// 添加自定义 Provider
    pub fn add_custom(&self, provider: ProviderTemplate) {
        let provider_id = provider.provider_id.clone();
        self.custom.insert(provider_id.clone(), provider);
        tracing::info!("Added custom provider: {}", provider_id);
    }

    /// 移除自定义 Provider
    pub fn remove_custom(&self, provider_id: &str) -> bool {
        let removed = self.custom.remove(provider_id);
        if removed.is_some() {
            tracing::info!("Removed custom provider: {}", provider_id);
            true
        } else {
            false
        }
    }
}

impl Default for ProviderEngine {
    fn default() -> Self {
        Self::new()
    }
}