//! 健康检查器模块
//!
//! 提供 Provider Plan 的主动健康探测功能

use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;
use chrono::Utc;
use reqwest::Client;

use crate::model::{ProviderTemplate, UserPlan};
use crate::model_types::{ApiFormat, HealthStatus};
use crate::business::{PlanManager, ProviderEngine};
use crate::storage::SqliteStore;

/// 健康检查结果
#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    /// 健康状态
    pub status: HealthStatus,
    /// 响应时间（毫秒）
    pub response_time_ms: i64,
    /// 错误信息
    pub error_message: Option<String>,
}

/// 健康检查器
pub struct HealthChecker {
    /// HTTP 客户端
    client: Client,
    /// Provider 引擎
    provider_engine: Arc<ProviderEngine>,
    /// SQLite 存储
    sqlite: Arc<SqliteStore>,
    /// Plan 管理器
    plan_manager: Arc<PlanManager>,
}

impl HealthChecker {
    /// 创建新的健康检查器
    pub fn new(
        provider_engine: Arc<ProviderEngine>,
        sqlite: Arc<SqliteStore>,
        plan_manager: Arc<PlanManager>,
    ) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            provider_engine,
            sqlite,
            plan_manager,
        }
    }

    /// 检查单个 Plan 的健康状态
    pub async fn check_plan(&self, plan_id: &str) -> Result<HealthCheckResult> {
        let plan = self.plan_manager.get(plan_id).await
            .ok_or_else(|| anyhow::anyhow!("Plan not found: {}", plan_id))?;

        // 如果 Plan 未启用，直接返回 Disabled
        if !plan.enabled {
            return Ok(HealthCheckResult {
                status: HealthStatus::Disabled,
                response_time_ms: 0,
                error_message: Some("Plan is disabled".to_string()),
            });
        }

        // 获取 Provider 信息
        let provider = self.provider_engine.get_provider(&plan.provider_id).await
            .ok_or_else(|| anyhow::anyhow!("Provider not found: {}", plan.provider_id))?;

        // 获取 Base URL
        let base_url = self.get_base_url(&provider, &plan)?;

        // 根据 API 格式选择探测方法
        let start = Instant::now();
        let probe_result = match provider.api_format {
            ApiFormat::Anthropic => self.probe_anthropic(&base_url, &plan).await,
            ApiFormat::OpenAi => self.probe_openai(&base_url, &plan).await,
            ApiFormat::Custom => {
                // 对于自定义格式，尝试 Anthropic 格式探测
                self.probe_anthropic(&base_url, &plan).await
            }
        };

        let response_time_ms = start.elapsed().as_millis() as i64;

        // 根据探测结果判定健康状态
        let result = match probe_result {
            Ok(status_code) => {
                let status = self.classify_status_from_code(status_code);
                HealthCheckResult {
                    status,
                    response_time_ms,
                    error_message: None,
                }
            }
            Err(e) => {
                let error_msg = e.to_string();
                // 判断错误类型
                let status = if error_msg.contains("timeout") || error_msg.contains("Timeout") {
                    HealthStatus::Error
                } else if error_msg.contains("connection") || error_msg.contains("Connection") {
                    HealthStatus::Error
                } else {
                    HealthStatus::Error
                };

                HealthCheckResult {
                    status,
                    response_time_ms,
                    error_message: Some(error_msg),
                }
            }
        };

        // 记录健康检查到 SQLite
        self.sqlite.log_health_check(
            plan_id.to_string(),
            result.status.to_string(),
            Some(result.response_time_ms),
            result.error_message.clone(),
        ).await?;

        // 更新 Plan 的健康状态
        self.update_plan_health(plan_id, &result).await?;

        Ok(result)
    }

    /// 获取 Base URL
    fn get_base_url(&self, provider: &ProviderTemplate, plan: &UserPlan) -> Result<String> {
        if let Some(template) = &provider.base_url_template {
            // 替换模板中的变量
            let url = template.replace("{plan_id}", &plan.plan_id);
            Ok(url)
        } else if let Some(base) = &provider.base_url {
            Ok(base.clone())
        } else {
            anyhow::bail!("No base URL configured for provider {}", provider.provider_id);
        }
    }

    /// Anthropic API 探测
    async fn probe_anthropic(&self, base_url: &str, plan: &UserPlan) -> Result<u16> {
        // 构建探测请求 - 最小化请求以节省 token
        let url = format!("{}/v1/messages", base_url);

        // 使用 plan 的 selected_model_id 或默认模型
        let model_id = if plan.selected_model_id.starts_with("claude") {
            plan.selected_model_id.clone()
        } else {
            // 如果不是 Claude 模型，使用默认的探测模型
            "claude-sonnet-4-5".to_string()
        };

        let body = serde_json::json!({
            "model": model_id,
            "max_tokens": 1,
            "messages": [
                {
                    "role": "user",
                    "content": "hi"
                }
            ]
        });

        tracing::debug!("Probing Anthropic API at {}", url);

        let response = self.client
            .post(&url)
            .header("x-api-key", &plan.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await?;

        let status = response.status().as_u16();

        // 即使返回错误状态码，也算探测成功（知道了服务状态）
        tracing::debug!("Anthropic probe response: {}", status);

        Ok(status)
    }

    /// OpenAI API 探测
    async fn probe_openai(&self, base_url: &str, plan: &UserPlan) -> Result<u16> {
        // 构建探测请求
        let url = format!("{}/v1/chat/completions", base_url);

        // 使用 plan 的 selected_model_id 或默认模型
        let model_id = if plan.selected_model_id.starts_with("gpt") {
            plan.selected_model_id.clone()
        } else {
            // 如果不是 GPT 模型，使用默认的探测模型
            "gpt-4o-mini".to_string()
        };

        let body = serde_json::json!({
            "model": model_id,
            "max_tokens": 1,
            "messages": [
                {
                    "role": "user",
                    "content": "hi"
                }
            ]
        });

        tracing::debug!("Probing OpenAI API at {}", url);

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", plan.api_key))
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await?;

        let status = response.status().as_u16();

        tracing::debug!("OpenAI probe response: {}", status);

        Ok(status)
    }

    /// 根据 HTTP 状态码判定健康状态
    pub fn classify_status_from_code(&self, code: u16) -> HealthStatus {
        if code >= 200 && code < 300 {
            HealthStatus::Healthy
        } else if code == 429 {
            // Rate limited but reachable
            HealthStatus::Warning
        } else if code >= 500 {
            HealthStatus::Error
        } else if code >= 400 {
            // Client errors might indicate auth issues
            HealthStatus::Warning
        } else {
            HealthStatus::Unknown
        }
    }

    /// 更新 Plan 的健康状态
    async fn update_plan_health(&self, plan_id: &str, result: &HealthCheckResult) -> Result<()> {
        let mut plan = self.plan_manager.get(plan_id).await
            .ok_or_else(|| anyhow::anyhow!("Plan not found: {}", plan_id))?;

        let old_status = plan.health_status;
        plan.health_status = result.status;
        plan.last_health_check = Some(Utc::now());

        // 状态变化时记录日志
        if old_status != result.status {
            if result.status == HealthStatus::Healthy && old_status == HealthStatus::Error {
                tracing::info!("Plan {} recovered from error to healthy", plan_id);
            } else if result.status == HealthStatus::Error {
                tracing::warn!("Plan {} health degraded to error: {:?}", plan_id, result.error_message);
            } else {
                tracing::info!("Plan {} health status changed: {} -> {}", plan_id, old_status, result.status);
            }
        }

        self.plan_manager.update(plan).await?;

        Ok(())
    }
}

// ============================================================================
// 后台健康监控
// ============================================================================

/// 启动后台健康监控任务
pub async fn start_health_monitor(
    health_checker: Arc<HealthChecker>,
    interval_secs: u64,
    recovery_check_interval_secs: u64,
) {
    tracing::info!(
        "Starting health monitor with interval {}s, recovery interval {}s",
        interval_secs,
        recovery_check_interval_secs
    );

    tokio::spawn(async move {
        loop {
            // 等待下一次检查周期
            tokio::time::sleep(Duration::from_secs(interval_secs)).await;

            tracing::debug!("Running periodic health check");

            // 加载所有 Plan
            let plans = match health_checker.plan_manager.load_all().await {
                Ok(plans) => plans,
                Err(e) => {
                    tracing::warn!("Failed to load plans for health check: {}", e);
                    continue;
                }
            };

            // 遍历所有启用的 Plan
            for plan in plans.iter().filter(|p| p.enabled) {
                // 如果 Plan 处于 Error 状态，使用更快的恢复检测频率
                if plan.health_status == HealthStatus::Error {
                    // 先等待恢复检测间隔
                    tokio::time::sleep(Duration::from_secs(recovery_check_interval_secs)).await;
                }

                // 执行健康检查
                let result = health_checker.check_plan(&plan.id).await;

                match result {
                    Ok(check_result) => {
                        // 状态恢复时记录
                        if check_result.status == HealthStatus::Healthy
                            && plan.health_status == HealthStatus::Error {
                            tracing::info!(
                                "Plan {} recovered! Response time: {}ms",
                                plan.id,
                                check_result.response_time_ms
                            );
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Health check failed for plan {}: {}", plan.id, e);
                    }
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_status() {
        let checker = HealthChecker::new(
            Arc::new(ProviderEngine::new()),
            Arc::new(SqliteStore::in_memory().unwrap()),
            Arc::new(PlanManager::new(Arc::new(crate::storage::ConfigStore::new().unwrap()))),
        );

        assert_eq!(checker.classify_status_from_code(200), HealthStatus::Healthy);
        assert_eq!(checker.classify_status_from_code(201), HealthStatus::Healthy);
        assert_eq!(checker.classify_status_from_code(429), HealthStatus::Warning);
        assert_eq!(checker.classify_status_from_code(500), HealthStatus::Error);
        assert_eq!(checker.classify_status_from_code(401), HealthStatus::Warning);
    }
}