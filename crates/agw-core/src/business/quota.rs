//! 配额追踪

use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc, Datelike};

/// 配额记录
#[derive(Debug, Clone)]
pub struct QuotaRecord {
    pub plan_id: String,
    pub daily_used: u64,
    pub monthly_used: u64,
    pub rpm_used: u32,
    pub last_reset: DateTime<Utc>,
}

/// 配额限制
#[derive(Debug, Clone)]
pub struct QuotaLimit {
    pub daily: Option<u64>,
    pub monthly: Option<u64>,
    pub rpm: Option<u32>,
}

/// 配额追踪器
pub struct QuotaTracker {
    /// 配额记录
    records: Arc<RwLock<HashMap<String, QuotaRecord>>>,
    /// 默认配额限制
    default_limits: Arc<RwLock<HashMap<String, QuotaLimit>>>,
}

impl QuotaTracker {
    /// 创建新的配额追踪器
    pub fn new() -> Self {
        Self {
            records: Arc::new(RwLock::new(HashMap::new())),
            default_limits: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 检查并消耗配额
    pub async fn check_and_consume(&self, plan_id: &str) -> bool {
        let mut records = self.records.write().await;
        let record = records.entry(plan_id.to_string()).or_insert_with(|| {
            QuotaRecord {
                plan_id: plan_id.to_string(),
                daily_used: 0,
                monthly_used: 0,
                rpm_used: 0,
                last_reset: Utc::now(),
            }
        });

        // 检查是否需要重置
        self.check_and_reset(record).await;

        // 检查配额
        let limits = self.default_limits.read().await;
        let limit = limits.get(plan_id);

        if let Some(limit) = limit {
            if let Some(daily) = limit.daily {
                if record.daily_used >= daily {
                    return false;
                }
            }
            if let Some(monthly) = limit.monthly {
                if record.monthly_used >= monthly {
                    return false;
                }
            }
        }

        // 消耗配额
        record.daily_used += 1;
        record.monthly_used += 1;
        record.rpm_used += 1;

        true
    }

    /// 检查并重置配额
    async fn check_and_reset(&self, record: &mut QuotaRecord) {
        let now = Utc::now();

        // 检查是否需要重置日配额
        let day_start = now.date_naive().and_hms_opt(0, 0, 0).unwrap();
        let last_day = record.last_reset.date_naive().and_hms_opt(0, 0, 0).unwrap();

        if day_start > last_day {
            record.daily_used = 0;
        }

        // 检查是否需要重置月配额
        let now_naive = now.naive_local();
        let last_naive = record.last_reset.naive_local();
        if now_naive.date().month() != last_naive.date().month() || now_naive.date().year() != last_naive.date().year() {
            record.monthly_used = 0;
        }

        record.last_reset = now;
    }

    /// 获取配额使用情况
    pub async fn get_usage(&self, plan_id: &str) -> Option<QuotaRecord> {
        let records = self.records.read().await;
        records.get(plan_id).cloned()
    }

    /// 获取配额限制
    pub async fn get_limits(&self, plan_id: &str) -> Option<QuotaLimit> {
        let limits = self.default_limits.read().await;
        limits.get(plan_id).cloned()
    }

    /// 设置配额限制
    pub async fn set_limits(&self, plan_id: &str, limits: QuotaLimit) {
        let mut default_limits = self.default_limits.write().await;
        default_limits.insert(plan_id.to_string(), limits);
    }

    /// 重置配额
    pub async fn reset(&self, plan_id: &str) {
        let mut records = self.records.write().await;
        if let Some(record) = records.get_mut(plan_id) {
            record.daily_used = 0;
            record.monthly_used = 0;
            record.rpm_used = 0;
            record.last_reset = Utc::now();
        }
    }

    /// 获取使用百分比
    pub async fn get_usage_percent(&self, plan_id: &str) -> Option<(f32, f32, f32)> {
        let usage = self.get_usage(plan_id).await?;
        let limits = self.get_limits(plan_id).await?;

        let daily_pct = match (limits.daily, usage.daily_used) {
            (Some(limit), _) if limit > 0 => usage.daily_used as f32 / limit as f32,
            _ => 0.0,
        };

        let monthly_pct = match (limits.monthly, usage.monthly_used) {
            (Some(limit), _) if limit > 0 => usage.monthly_used as f32 / limit as f32,
            _ => 0.0,
        };

        let rpm_pct = match (limits.rpm, usage.rpm_used) {
            (Some(limit), _) if limit > 0 => usage.rpm_used as f32 / limit as f32,
            _ => 0.0,
        };

        Some((daily_pct, monthly_pct, rpm_pct))
    }
}

impl Default for QuotaTracker {
    fn default() -> Self {
        Self::new()
    }
}