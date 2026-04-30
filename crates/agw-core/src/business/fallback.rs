//! Fallback 引擎


use crate::model::FallbackConfig;

/// Fallback 触发原因
#[derive(Debug, Clone)]
pub enum FallbackReason {
    RateLimit,
    ServerError(String),
    ConnectionFailure,
    Timeout,
    QuotaExceeded,
}

/// Fallback 引擎
pub struct FallbackEngine {
    config: FallbackConfig,
}

impl FallbackEngine {
    /// 创建新的 Fallback 引擎
    pub fn new() -> Self {
        Self {
            config: FallbackConfig::default(),
        }
    }

    /// 创建带配置的 Fallback 引擎
    pub fn with_config(config: FallbackConfig) -> Self {
        Self { config }
    }

    /// 检查是否应该触发 Fallback
    pub fn should_fallback(&self, reason: &FallbackReason) -> bool {
        if !self.config.enabled {
            return false;
        }

        match reason {
            FallbackReason::RateLimit => true,
            FallbackReason::ServerError(_) => true,
            FallbackReason::ConnectionFailure => true,
            FallbackReason::Timeout => true,
            FallbackReason::QuotaExceeded => true,
        }
    }

    /// 获取备选套餐
    pub async fn find_alternative(
        &self,
        current_plan_id: &str,
    ) -> Option<String> {
        if !self.config.enabled {
            return None;
        }

        // 按照优先级顺序查找备选
        for plan_id in &self.config.priority_order {
            if plan_id != current_plan_id {
                // TODO: 检查该套餐是否可用（健康状态、配额等）
                return Some(plan_id.clone());
            }
        }

        None
    }

    /// 启用/禁用 Fallback
    pub fn set_enabled(&mut self, enabled: bool) {
        self.config.enabled = enabled;
    }

    /// 设置优先级顺序
    pub fn set_priority(&mut self, priority: Vec<String>) {
        self.config.priority_order = priority;
    }

    /// 获取配置
    pub fn get_config(&self) -> &FallbackConfig {
        &self.config
    }

    /// 获取最大重试次数
    pub fn max_attempts(&self) -> u32 {
        self.config.max_attempts
    }
}

impl Default for FallbackEngine {
    fn default() -> Self {
        Self::new()
    }
}