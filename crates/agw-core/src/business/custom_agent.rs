//! 自定义 Agent 工具管理

use std::sync::Arc;
use dashmap::DashMap;
use uuid::Uuid;

use crate::model::{CustomAgent, CustomAgentsConfig};
use crate::storage::ConfigStore;

/// 自定义 Agent 管理器
pub struct CustomAgentManager {
    config_store: Arc<ConfigStore>,
    /// 内存缓存
    cache: Arc<DashMap<String, CustomAgent>>,
}

impl CustomAgentManager {
    /// 创建新的管理器
    pub fn new(config_store: Arc<ConfigStore>) -> Self {
        Self {
            config_store,
            cache: Arc::new(DashMap::new()),
        }
    }

    /// 加载所有自定义 Agent
    pub async fn load_all(&self) -> anyhow::Result<Vec<CustomAgent>> {
        let config = self.config_store.load_custom_agents().await?;
        for agent in &config.custom_agents {
            self.cache.insert(agent.id.clone(), agent.clone());
        }
        Ok(config.custom_agents)
    }

    /// 获取单个自定义 Agent
    pub async fn get(&self, id: &str) -> Option<CustomAgent> {
        // 先从缓存获取
        if let Some(agent) = self.cache.get(id) {
            return Some(agent.clone());
        }
        // 尝试从存储加载
        self.load_all().await.ok()?;
        self.cache.get(id).map(|a| a.clone())
    }

    /// 通过 agent_id 获取
    pub async fn get_by_agent_id(&self, agent_id: &str) -> Option<CustomAgent> {
        self.load_all().await.ok()?;
        self.cache.iter()
            .find(|r| r.value().agent_id == agent_id)
            .map(|r| r.value().clone())
    }

    /// 添加自定义 Agent
    pub async fn add(&self, agent: CustomAgent) -> anyhow::Result<CustomAgent> {
        // 先加载已有数据到缓存
        self.load_all().await?;

        // 检查 agent_id 是否已存在
        if self.get_by_agent_id(&agent.agent_id).await.is_some() {
            anyhow::bail!("Agent ID already exists: {}", agent.agent_id);
        }

        self.cache.insert(agent.id.clone(), agent.clone());
        self.save_all().await?;
        Ok(agent)
    }

    /// 创建新的自定义 Agent
    pub async fn create(
        &self,
        agent_id: String,
        name: String,
        version: String,
        logo_url: Option<String>,
        description: Option<String>,
    ) -> anyhow::Result<CustomAgent> {
        let id = Uuid::new_v4().to_string();
        let mut agent = CustomAgent::new(id, agent_id, name, version);
        agent.logo_url = logo_url;
        agent.description = description;
        self.add(agent).await
    }

    /// 更新自定义 Agent
    pub async fn update(&self, id: &str, updates: CustomAgentUpdate) -> anyhow::Result<CustomAgent> {
        let mut agent = self.get(id).await
            .ok_or_else(|| anyhow::anyhow!("Custom agent not found: {}", id))?;

        if let Some(name) = updates.name {
            agent.name = name;
        }
        if let Some(version) = updates.version {
            agent.version = version;
        }
        if let Some(logo_url) = updates.logo_url {
            agent.logo_url = Some(logo_url);
        }
        if let Some(description) = updates.description {
            agent.description = Some(description);
        }
        agent.updated_at = chrono::Utc::now();

        self.cache.insert(id.to_string(), agent.clone());
        self.save_all().await?;
        Ok(agent)
    }

    /// 删除自定义 Agent
    pub async fn delete(&self, id: &str) -> anyhow::Result<()> {
        // 先加载已有数据到缓存
        self.load_all().await?;

        if self.cache.get(id).is_none() {
            anyhow::bail!("Custom agent not found: {}", id);
        }

        self.cache.remove(id);
        self.save_all().await
    }

    /// 列出所有自定义 Agent
    pub async fn list(&self) -> anyhow::Result<Vec<CustomAgent>> {
        self.load_all().await
    }

    /// 保存所有自定义 Agent 到存储
    async fn save_all(&self) -> anyhow::Result<()> {
        let agents: Vec<CustomAgent> = self.cache.iter().map(|r| r.value().clone()).collect();
        let config = CustomAgentsConfig {
            version: "1.0".to_string(),
            custom_agents: agents,
        };
        self.config_store.save_custom_agents(&config).await
    }
}

/// 自定义 Agent 更新请求
#[derive(Debug, Clone, Default)]
pub struct CustomAgentUpdate {
    pub name: Option<String>,
    pub version: Option<String>,
    pub logo_url: Option<String>,
    pub description: Option<String>,
}