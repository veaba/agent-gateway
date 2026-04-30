//! 全局状态管理

use std::sync::Arc;
use tokio::sync::RwLock;
use dashmap::DashMap;

use crate::model::{UserPlan, FallbackConfig};

/// 全局状态
pub struct GlobalState {
    /// 当前活动的 UserPlan
    pub active_plan_id: Arc<RwLock<Option<String>>>,
    /// Agent 到 Plan 的映射
    pub agent_to_plan: Arc<DashMap<String, String>>,
    /// Fallback 配置
    pub fallback_config: Arc<RwLock<FallbackConfig>>,
    /// 套餐缓存
    pub plan_cache: Arc<DashMap<String, UserPlan>>,
}

impl GlobalState {
    /// 创建新的全局状态
    pub fn new() -> Self {
        Self {
            active_plan_id: Arc::new(RwLock::new(None)),
            agent_to_plan: Arc::new(DashMap::new()),
            fallback_config: Arc::new(RwLock::new(FallbackConfig::default())),
            plan_cache: Arc::new(DashMap::new()),
        }
    }

    /// 设置活动的套餐
    pub async fn set_active_plan(&self, plan_id: String) {
        let mut active = self.active_plan_id.write().await;
        *active = Some(plan_id);
    }

    /// 获取活动的套餐 ID
    pub async fn get_active_plan_id(&self) -> Option<String> {
        let active = self.active_plan_id.read().await;
        active.clone()
    }

    /// 绑定 Agent 到套餐
    pub fn bind_agent(&self, agent_id: &str, plan_id: &str) {
        self.agent_to_plan.insert(agent_id.to_string(), plan_id.to_string());
    }

    /// 解绑 Agent
    pub fn unbind_agent(&self, agent_id: &str) {
        self.agent_to_plan.remove(agent_id);
    }

    /// 获取 Agent 绑定的套餐
    pub fn get_plan_for_agent(&self, agent_id: &str) -> Option<String> {
        self.agent_to_plan.get(agent_id).map(|r| r.value().clone())
    }
}

impl Default for GlobalState {
    fn default() -> Self {
        Self::new()
    }
}