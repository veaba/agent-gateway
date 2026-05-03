//! 自定义 Provider 管理

use std::sync::Arc;
use dashmap::DashMap;
use uuid::Uuid;

use crate::model::{CustomProvider, CustomProvidersConfig, CustomModel, ApiFormat};
use crate::storage::ConfigStore;

/// 自定义 Provider 管理器
pub struct CustomProviderManager {
    config_store: Arc<ConfigStore>,
    /// 内存缓存
    cache: Arc<DashMap<String, CustomProvider>>,
}

impl CustomProviderManager {
    /// 创建新的管理器
    pub fn new(config_store: Arc<ConfigStore>) -> Self {
        Self {
            config_store,
            cache: Arc::new(DashMap::new()),
        }
    }

    /// 加载所有自定义 Provider
    pub async fn load_all(&self) -> anyhow::Result<Vec<CustomProvider>> {
        let config = self.config_store.load_custom_providers().await?;
        for provider in &config.custom_providers {
            self.cache.insert(provider.id.clone(), provider.clone());
        }
        Ok(config.custom_providers)
    }

    /// 获取单个自定义 Provider
    pub async fn get(&self, id: &str) -> Option<CustomProvider> {
        // 先从缓存获取
        if let Some(provider) = self.cache.get(id) {
            return Some(provider.clone());
        }
        // 尝试从存储加载
        self.load_all().await.ok()?;
        self.cache.get(id).map(|p| p.clone())
    }

    /// 通过 provider_id 获取
    pub async fn get_by_provider_id(&self, provider_id: &str) -> Option<CustomProvider> {
        self.load_all().await.ok()?;
        self.cache.iter()
            .find(|r| r.value().provider_id == provider_id)
            .map(|r| r.value().clone())
    }

    /// 添加自定义 Provider
    pub async fn add(&self, provider: CustomProvider) -> anyhow::Result<CustomProvider> {
        // 先加载已有数据到缓存
        self.load_all().await?;

        // 检查 provider_id 是否已存在
        if self.get_by_provider_id(&provider.provider_id).await.is_some() {
            anyhow::bail!("Provider ID already exists: {}", provider.provider_id);
        }

        self.cache.insert(provider.id.clone(), provider.clone());
        self.save_all().await?;
        Ok(provider)
    }

    /// 创建新的自定义 Provider
    pub async fn create(
        &self,
        provider_id: String,
        name: String,
        api_format: ApiFormat,
        base_url: String,
        requires_api_key: bool,
        description: Option<String>,
        logo_url: Option<String>,
        homepage: Option<String>,
        docs_url: Option<String>,
        get_api_key_url: Option<String>,
        models: Vec<CustomModel>,
    ) -> anyhow::Result<CustomProvider> {
        let id = Uuid::new_v4().to_string();
        let mut provider = CustomProvider::new(id, provider_id, name, api_format, base_url, requires_api_key);
        provider.description = description;
        provider.logo_url = logo_url;
        provider.homepage = homepage;
        provider.docs_url = docs_url;
        provider.get_api_key_url = get_api_key_url;
        provider.models = models;
        self.add(provider).await
    }

    /// 更新自定义 Provider
    pub async fn update(&self, id: &str, updates: CustomProviderUpdate) -> anyhow::Result<CustomProvider> {
        let mut provider = self.get(id).await
            .ok_or_else(|| anyhow::anyhow!("Custom provider not found: {}", id))?;

        if let Some(name) = updates.name {
            provider.name = name;
        }
        if let Some(description) = updates.description {
            provider.description = Some(description);
        }
        if let Some(logo_url) = updates.logo_url {
            provider.logo_url = Some(logo_url);
        }
        if let Some(homepage) = updates.homepage {
            provider.homepage = Some(homepage);
        }
        if let Some(docs_url) = updates.docs_url {
            provider.docs_url = Some(docs_url);
        }
        if let Some(get_api_key_url) = updates.get_api_key_url {
            provider.get_api_key_url = Some(get_api_key_url);
        }
        if let Some(base_url) = updates.base_url {
            provider.base_url = base_url;
        }
        if let Some(api_format) = updates.api_format {
            provider.api_format = api_format;
        }
        if let Some(requires_api_key) = updates.requires_api_key {
            provider.requires_api_key = requires_api_key;
        }
        if let Some(models) = updates.models {
            provider.models = models;
        }
        provider.updated_at = chrono::Utc::now();

        self.cache.insert(id.to_string(), provider.clone());
        self.save_all().await?;
        Ok(provider)
    }

    /// 删除自定义 Provider
    pub async fn delete(&self, id: &str) -> anyhow::Result<()> {
        // 先加载已有数据到缓存
        self.load_all().await?;

        if self.cache.get(id).is_none() {
            anyhow::bail!("Custom provider not found: {}", id);
        }

        self.cache.remove(id);
        self.save_all().await
    }

    /// 列出所有自定义 Provider
    pub async fn list(&self) -> anyhow::Result<Vec<CustomProvider>> {
        self.load_all().await
    }

    /// 保存所有自定义 Provider 到存储
    async fn save_all(&self) -> anyhow::Result<()> {
        let providers: Vec<CustomProvider> = self.cache.iter().map(|r| r.value().clone()).collect();
        let config = CustomProvidersConfig {
            version: "1.0".to_string(),
            custom_providers: providers,
        };
        self.config_store.save_custom_providers(&config).await
    }
}

/// 自定义 Provider 更新请求
#[derive(Debug, Clone, Default)]
pub struct CustomProviderUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub logo_url: Option<String>,
    pub homepage: Option<String>,
    pub docs_url: Option<String>,
    pub get_api_key_url: Option<String>,
    pub base_url: Option<String>,
    pub api_format: Option<ApiFormat>,
    pub requires_api_key: Option<bool>,
    pub models: Option<Vec<CustomModel>>,
}