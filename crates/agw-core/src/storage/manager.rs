//! 存储管理器 - 统一管理所有存储层

use std::path::PathBuf;
use std::sync::Arc;
use anyhow::Result;

use super::{ConfigStore, SqliteStore, RequestLogStore, RequestLogEntry};

/// 存储管理器
#[derive(Clone)]
pub struct StorageManager {
    pub config: Arc<ConfigStore>,
    pub sqlite: Arc<SqliteStore>,
    pub request_logs: Arc<RequestLogStore>,
}

impl StorageManager {
    /// 创建存储管理器
    pub async fn new() -> Result<Self> {
        let config = Arc::new(ConfigStore::new()?);

        // 初始化数据目录
        config.init_data_dir().await?;

        // 创建数据目录路径
        let data_dir = config.data_dir();
        let log_dir = data_dir.join("logs");

        // 创建存储实例
        let sqlite_path = data_dir.join("agent_gateway.db");
        let sqlite = Arc::new(SqliteStore::new(sqlite_path)?);

        let request_logs = Arc::new(RequestLogStore::new(log_dir));

        Ok(Self {
            config,
            sqlite,
            request_logs,
        })
    }

    /// 创建存储管理器 (带自定义路径)
    pub async fn with_paths(
        config_dir: PathBuf,
        data_dir: PathBuf,
    ) -> Result<Self> {
        let config = Arc::new(ConfigStore::with_path(config_dir.clone())?);

        // 初始化数据目录
        tokio::fs::create_dir_all(&data_dir).await?;
        tokio::fs::create_dir_all(data_dir.join("logs")).await?;
        tokio::fs::create_dir_all(data_dir.join("plugins")).await?;

        // 创建存储实例
        let sqlite_path = data_dir.join("agent_gateway.db");
        let sqlite = Arc::new(SqliteStore::new(sqlite_path)?);

        let log_dir = data_dir.join("logs");
        let request_logs = Arc::new(RequestLogStore::new(log_dir));

        Ok(Self {
            config,
            sqlite,
            request_logs,
        })
    }

    /// 获取数据目录
    pub fn data_dir(&self) -> PathBuf {
        self.config.data_dir()
    }

    /// 获取日志目录
    pub fn log_dir(&self) -> PathBuf {
        self.data_dir().join("logs")
    }
}

/// 日志写入器 (用于 tracing Layer)
pub struct LogWriter {
    store: Arc<RequestLogStore>,
}

impl LogWriter {
    /// 创建日志写入器
    pub fn new(store: Arc<RequestLogStore>) -> Self {
        Self { store }
    }

    /// 写入日志
    pub fn write(&self, entry: &RequestLogEntry) -> anyhow::Result<()> {
        self.store.write_sync(entry)
    }
}

/// 指标收集器
#[derive(Debug, Clone, Default)]
pub struct MetricsCollector {
    /// 总请求数
    pub total_requests: u64,
    /// 总错误数
    pub total_errors: u64,
    /// 总输入 tokens
    pub total_input_tokens: u64,
    /// 总输出 tokens
    pub total_output_tokens: u64,
    /// 总延迟 (ms)
    pub total_latency_ms: u64,
    /// 按状态码统计
    pub status_codes: std::collections::HashMap<u16, u64>,
    /// 按 plan_id 统计
    pub by_plan: std::collections::HashMap<String, PlanMetrics>,
}

#[derive(Debug, Clone, Default)]
pub struct PlanMetrics {
    pub requests: u64,
    pub errors: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub avg_latency_ms: f64,
}

impl MetricsCollector {
    /// 添加请求记录
    pub fn record_request(&mut self, entry: &RequestLogEntry) {
        self.total_requests += 1;

        // 更新状态码统计
        if let Some(status) = entry.status_code {
            *self.status_codes.entry(status).or_insert(0) += 1;
            if status >= 400 {
                self.total_errors += 1;
            }
        }

        // 更新 plan 统计
        if let Some(plan_id) = &entry.plan_id {
            let metrics = self.by_plan.entry(plan_id.clone()).or_default();
            metrics.requests += 1;

            // 更新延迟
            if let Some(latency) = entry.latency_ms {
                self.total_latency_ms += latency;
                let count = metrics.requests;
                metrics.avg_latency_ms =
                    (metrics.avg_latency_ms * (count - 1) as f64 + latency as f64) / count as f64;
            }
        }
    }

    /// 获取成功率
    pub fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            return 1.0;
        }
        (self.total_requests - self.total_errors) as f64 / self.total_requests as f64
    }

    /// 获取平均延迟
    pub fn avg_latency_ms(&self) -> f64 {
        if self.total_requests == 0 {
            return 0.0;
        }
        self.total_latency_ms as f64 / self.total_requests as f64
    }
}