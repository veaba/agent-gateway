//! Provider 引擎

use std::sync::Arc;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use anyhow::Result;

use crate::model::{ProviderTemplate, CodingPlanTemplate};

/// 内置 YAML 配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreConfig {
    pub providers: Vec<ProviderTemplate>,
    pub registry: RegistryConfig,
}

/// Registry 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryConfig {
    pub url: String,
    pub local_version: String,
}

/// YAML 配置根结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgwConfig {
    pub core: CoreConfig,
}

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

/// 内置配置 YAML 内容（嵌入编译时）
const BUILTIN_CONFIG_YAML: &str = include_str!("../agw.yaml");

impl ProviderEngine {
    /// 创建新的 Provider 引擎
    pub fn new() -> Self {
        // 从嵌入的 YAML 解析配置
        let config: AgwConfig = serde_yaml::from_str(BUILTIN_CONFIG_YAML)
            .expect("Failed to parse builtin agw.yaml");

        Self::with_registry_and_version(&config.core.registry.url, &config.core.registry.local_version, config.core.providers)
    }

    /// 使用自定义 registry URL 创建
    pub fn with_registry(registry_url: &str) -> Self {
        // 从嵌入的 YAML 解析配置
        let config: AgwConfig = serde_yaml::from_str(BUILTIN_CONFIG_YAML)
            .expect("Failed to parse builtin agw.yaml");

        Self::with_registry_and_version(registry_url, &config.core.registry.local_version, config.core.providers)
    }

    /// 使用完整配置创建
    fn with_registry_and_version(registry_url: &str, local_version: &str, providers: Vec<ProviderTemplate>) -> Self {
        let engine = Self {
            builtin: Arc::new(DashMap::new()),
            custom: Arc::new(DashMap::new()),
            registry_url: registry_url.to_string(),
            local_version: local_version.to_string(),
        };
        engine.load_providers_from_config(providers);
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

    /// 从配置加载 Provider 模板
    fn load_providers_from_config(&self, providers: Vec<ProviderTemplate>) {
        for provider in providers {
            let provider_id = provider.provider_id.clone();
            self.builtin.insert(provider_id.clone(), provider);
            tracing::debug!("Loaded builtin provider: {}", provider_id);
        }
        tracing::info!("Loaded {} builtin providers from agw.yaml", self.builtin.len());
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