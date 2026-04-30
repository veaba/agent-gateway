//! Fallback 引擎

use std::sync::Arc;

use crate::model::FallbackConfig;
use crate::business::{PlanManager, QuotaTracker};
use crate::model_types::HealthStatus;

/// Fallback 触发原因
#[derive(Debug, Clone)]
pub enum FallbackReason {
    RateLimit,
    ServerError(String),
    ConnectionFailure,
    Timeout,
    QuotaExceeded,
}

impl FallbackReason {
    pub fn from_status(status: u16, error_message: Option<String>) -> Option<Self> {
        match status {
            429 => Some(FallbackReason::RateLimit),
            500..=599 => Some(FallbackReason::ServerError(
                error_message.unwrap_or_else(|| status.to_string()),
            )),
            _ => None,
        }
    }

    pub fn from_error(error: &str) -> Self {
        let error_lower = error.to_lowercase();
        if error_lower.contains("timeout") || error_lower.contains("timed out") {
            FallbackReason::Timeout
        } else if error_lower.contains("connection") || error_lower.contains("refused") {
            FallbackReason::ConnectionFailure
        } else {
            FallbackReason::ServerError(error.to_string())
        }
    }
}

/// Fallback 引擎
pub struct FallbackEngine {
    config: FallbackConfig,
    plan_manager: Option<Arc<PlanManager>>,
    quota_tracker: Option<Arc<QuotaTracker>>,
}

impl FallbackEngine {
    /// 创建新的 Fallback 引擎
    pub fn new() -> Self {
        Self {
            config: FallbackConfig::default(),
            plan_manager: None,
            quota_tracker: None,
        }
    }

    /// 创建带配置的 Fallback 引擎
    pub fn with_config(config: FallbackConfig) -> Self {
        Self {
            config,
            plan_manager: None,
            quota_tracker: None,
        }
    }

    /// 创建带完整依赖的 Fallback 引擎
    pub fn with_dependencies(
        config: FallbackConfig,
        plan_manager: Arc<PlanManager>,
        quota_tracker: Arc<QuotaTracker>,
    ) -> Self {
        Self {
            config,
            plan_manager: Some(plan_manager),
            quota_tracker: Some(quota_tracker),
        }
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
    ///
    /// 按优先级顺序查找，并检查候选套餐的健康状态和配额：
    /// - 跳过被禁用 (enabled=false) 的套餐
    /// - 跳过健康状态为 Error 或 Disabled 的套餐
    /// - 跳过日配额或月配额已用尽的套餐
    pub async fn find_alternative(
        &self,
        current_plan_id: &str,
    ) -> Option<String> {
        if !self.config.enabled {
            return None;
        }

        // 按照优先级顺序查找备选
        for plan_id in &self.config.priority_order {
            if plan_id == current_plan_id {
                continue;
            }

            // 检查套餐是否存在且已启用
            if let Some(plan_manager) = &self.plan_manager {
                match plan_manager.get(plan_id).await {
                    Some(plan) => {
                        if !plan.enabled {
                            tracing::debug!(
                                plan_id = %plan_id,
                                "FallbackEngine: plan is disabled, skipping"
                            );
                            continue;
                        }
                        match plan.health_status {
                            HealthStatus::Error | HealthStatus::Disabled => {
                                tracing::debug!(
                                    plan_id = %plan_id,
                                    health_status = %plan.health_status,
                                    "FallbackEngine: plan health status is bad, skipping"
                                );
                                continue;
                            }
                            _ => {}
                        }
                    }
                    None => {
                        tracing::debug!(
                            plan_id = %plan_id,
                            "FallbackEngine: plan not found, skipping"
                        );
                        continue;
                    }
                }
            }

            // 检查配额是否充足（只检查，不消耗）
            if let Some(quota_tracker) = &self.quota_tracker {
                let usage = quota_tracker.get_usage(plan_id).await;
                let limits = quota_tracker.get_limits(plan_id).await;

                if let (Some(usage), Some(limits)) = (usage, limits) {
                    if let Some(daily_limit) = limits.daily {
                        if usage.daily_used >= daily_limit {
                            tracing::debug!(
                                plan_id = %plan_id,
                                used = usage.daily_used,
                                limit = daily_limit,
                                "FallbackEngine: daily quota exceeded, skipping"
                            );
                            continue;
                        }
                    }
                    if let Some(monthly_limit) = limits.monthly {
                        if usage.monthly_used >= monthly_limit {
                            tracing::debug!(
                                plan_id = %plan_id,
                                used = usage.monthly_used,
                                limit = monthly_limit,
                                "FallbackEngine: monthly quota exceeded, skipping"
                            );
                            continue;
                        }
                    }
                }
            }

            // 通过所有检查，返回该备选套餐
            tracing::info!(
                current_plan = %current_plan_id,
                fallback_plan = %plan_id,
                "FallbackEngine: selected alternative plan"
            );
            return Some(plan_id.clone());
        }

        tracing::warn!(
            current_plan = %current_plan_id,
            "FallbackEngine: no alternative plan available"
        );
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
