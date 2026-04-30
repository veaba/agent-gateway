//! 套餐管理

use std::sync::Arc;
use dashmap::DashMap;

use crate::model::{UserPlan, UserPlansConfig};
use crate::storage::{ConfigStore, SqliteStore};
use crate::business::{ProviderEngine, health_checker::HealthChecker};

/// 套餐管理器
pub struct PlanManager {
    config_store: Arc<ConfigStore>,
    /// 内存缓存
    cache: Arc<DashMap<String, UserPlan>>,
    /// Provider 引擎（可选，用于健康检查）
    provider_engine: Option<Arc<ProviderEngine>>,
    /// SQLite 存储（可选，用于健康检查）
    sqlite_store: Option<Arc<SqliteStore>>,
}

impl PlanManager {
    /// 创建新的套餐管理器
    pub fn new(config_store: Arc<ConfigStore>) -> Self {
        Self {
            config_store,
            cache: Arc::new(DashMap::new()),
            provider_engine: None,
            sqlite_store: None,
        }
    }

    /// 设置 Provider 引擎和 SQLite 存储（用于健康检查）
    pub fn with_health_check_deps(
        &mut self,
        provider_engine: Arc<ProviderEngine>,
        sqlite_store: Arc<SqliteStore>,
    ) {
        self.provider_engine = Some(provider_engine);
        self.sqlite_store = Some(sqlite_store);
    }

    /// 加载所有套餐
    pub async fn load_all(&self) -> anyhow::Result<Vec<UserPlan>> {
        let config = self.config_store.load_user_plans().await?;
        for plan in &config.user_plans {
            self.cache.insert(plan.id.clone(), plan.clone());
        }
        Ok(config.user_plans)
    }

    /// 获取套餐
    pub async fn get(&self, id: &str) -> Option<UserPlan> {
        // 先从缓存获取
        if let Some(plan) = self.cache.get(id) {
            return Some(plan.clone());
        }
        // 尝试从存储加载
        self.load_all().await.ok()?;
        self.cache.get(id).map(|p| p.clone())
    }

    /// 添加套餐
    pub async fn add(&self, plan: UserPlan) -> anyhow::Result<()> {
        // 先加载已有套餐到缓存
        self.load_all().await?;
        // 再添加新套餐
        self.cache.insert(plan.id.clone(), plan.clone());
        self.save_all().await
    }

    /// 更新套餐
    pub async fn update(&self, plan: UserPlan) -> anyhow::Result<()> {
        // 先加载已有套餐到缓存
        self.load_all().await?;
        self.cache.insert(plan.id.clone(), plan);
        self.save_all().await
    }

    /// 删除套餐
    pub async fn delete(&self, id: &str) -> anyhow::Result<()> {
        // 先加载已有套餐到缓存
        self.load_all().await?;
        self.cache.remove(id);
        self.save_all().await
    }

    /// 获取默认套餐
    pub async fn get_default(&self) -> Option<UserPlan> {
        let config = self.config_store.load_user_plans().await.ok()?;
        let default_id = config.default_user_plan_id?;
        self.get(&default_id).await
    }

    /// 设置默认套餐
    pub async fn set_default(&self, id: &str) -> anyhow::Result<()> {
        if self.get(id).await.is_none() {
            anyhow::bail!("Plan not found: {}", id);
        }
        self.config_store.set_default_plan(id).await
    }

    /// 绑定 Agent 到套餐
    pub async fn bind_agent(&self, plan_id: &str, agent_id: &str) -> anyhow::Result<()> {
        let mut plan = self.get(plan_id).await
            .ok_or_else(|| anyhow::anyhow!("Plan not found: {}", plan_id))?;

        if plan.bound_agents.iter().any(|b| b.agent_id == agent_id) {
            anyhow::bail!("Agent already bound: {}", agent_id);
        }

        plan.bound_agents.push(crate::model::AgentBinding {
            agent_id: agent_id.to_string(),
            configured: false,
            config_status: crate::model_types::AgentConfigStatus::NotConfigured,
            last_connected: None,
            error_message: None,
        });

        self.update(plan).await
    }

    /// 解绑 Agent
    pub async fn unbind_agent(&self, plan_id: &str, agent_id: &str) -> anyhow::Result<()> {
        let mut plan = self.get(plan_id).await
            .ok_or_else(|| anyhow::anyhow!("Plan not found: {}", plan_id))?;

        let before = plan.bound_agents.len();
        plan.bound_agents.retain(|b| b.agent_id != agent_id);

        if plan.bound_agents.len() == before {
            anyhow::bail!("Agent not bound: {}", agent_id);
        }

        self.update(plan).await
    }

    /// 自动配置 Agent
    pub async fn auto_config_agent(&self, plan_id: &str, agent_id: &str) -> anyhow::Result<bool> {
        let mut plan = self.get(plan_id).await
            .ok_or_else(|| anyhow::anyhow!("Plan not found: {}", plan_id))?;

        // 先获取配置状态
        let _binding_result = {
            let binding = plan.bound_agents.iter()
                .find(|b| b.agent_id == agent_id)
                .ok_or_else(|| anyhow::anyhow!("Agent not bound: {}", agent_id))?;
            binding.configured
        };

        tracing::info!("Auto-configuring agent {} for plan {}", agent_id, plan_id);

        let config_result = super::AgentAutoConfig::configure(agent_id, "127.0.0.1:8080").await;
        let final_configured = match config_result {
            Ok(report) => {
                tracing::info!(
                    "Agent {} auto-configured via {}: {:?}",
                    report.agent,
                    report.method,
                    report.paths
                );
                if report.requires_reload {
                    if let Some(cmd) = &report.reload_command {
                        tracing::info!("Reload command: {}", cmd);
                    }
                }
                // 修改 binding
                if let Some(binding) = plan.bound_agents.iter_mut().find(|b| b.agent_id == agent_id) {
                    binding.configured = true;
                    binding.config_status = crate::model_types::AgentConfigStatus::AutoConfigured;
                    binding.last_connected = Some(chrono::Utc::now());
                    binding.error_message = None;
                }
                true
            }
            Err(e) => {
                tracing::warn!("Auto-config failed for agent {}: {}", agent_id, e);
                if let Some(binding) = plan.bound_agents.iter_mut().find(|b| b.agent_id == agent_id) {
                    binding.configured = false;
                    binding.config_status = crate::model_types::AgentConfigStatus::ConfigError;
                    binding.error_message = Some(e.to_string());
                }
                false
            }
        };

        self.update(plan).await?;
        Ok(final_configured)
    }

    /// 保存所有套餐到存储
    async fn save_all(&self) -> anyhow::Result<()> {
        let plans: Vec<UserPlan> = self.cache.iter().map(|r| r.value().clone()).collect();
        let config = UserPlansConfig {
            version: "2.0".to_string(),
            default_user_plan_id: None,
            user_plans: plans,
        };
        self.config_store.save_user_plans(&config).await
    }

    /// 测试套餐连接
    pub async fn test_connection(&self, id: &str) -> anyhow::Result<bool> {
        let plan = self.get(id).await
            .ok_or_else(|| anyhow::anyhow!("Plan not found: {}", id))?;

        // 如果没有配置健康检查依赖，返回基于当前状态的结果
        if self.provider_engine.is_none() || self.sqlite_store.is_none() {
            tracing::warn!("Health check dependencies not configured for PlanManager");
            return Ok(plan.health_status == crate::model_types::HealthStatus::Healthy
                || plan.health_status == crate::model_types::HealthStatus::Unknown);
        }

        // 创建 HealthChecker 并执行检查
        let checker = HealthChecker::new(
            self.provider_engine.clone().unwrap(),
            self.sqlite_store.clone().unwrap(),
            // 创建一个新的 PlanManager 用于健康检查（避免循环依赖）
            Arc::new(PlanManager::new(self.config_store.clone())),
        );

        let result = checker.check_plan(id).await?;

        // 更新本缓存的 Plan 健康状态
        if let Some(mut cached_plan) = self.cache.get_mut(id) {
            cached_plan.health_status = result.status;
            cached_plan.last_health_check = Some(chrono::Utc::now());
        }

        tracing::info!(
            "Connection test for plan {} ({}): status={}, response_time={}ms",
            plan.name,
            plan.provider_id,
            result.status,
            result.response_time_ms
        );

        Ok(result.status == crate::model_types::HealthStatus::Healthy)
    }
}