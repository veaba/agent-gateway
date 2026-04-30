//! 套餐管理

use std::sync::Arc;
use dashmap::DashMap;

use crate::model::{UserPlan, UserPlansConfig};
use crate::storage::ConfigStore;

/// 套餐管理器
pub struct PlanManager {
    config_store: Arc<ConfigStore>,
    /// 内存缓存
    cache: Arc<DashMap<String, UserPlan>>,
}

impl PlanManager {
    /// 创建新的套餐管理器
    pub fn new(config_store: Arc<ConfigStore>) -> Self {
        Self {
            config_store,
            cache: Arc::new(DashMap::new()),
        }
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
        self.cache.insert(plan.id.clone(), plan.clone());
        self.save_all().await
    }

    /// 更新套餐
    pub async fn update(&self, plan: UserPlan) -> anyhow::Result<()> {
        self.cache.insert(plan.id.clone(), plan);
        self.save_all().await
    }

    /// 删除套餐
    pub async fn delete(&self, id: &str) -> anyhow::Result<()> {
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

        // TODO: 实现实际的连接测试
        tracing::info!("Testing connection for plan: {} ({})", plan.name, plan.provider_id);
        Ok(true)
    }
}