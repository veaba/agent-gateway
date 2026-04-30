//! 配额追踪

use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc, Datelike};
use anyhow::Result;
use serde::Serialize;

use crate::storage::SqliteStore;

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

/// 告警类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AlertType {
    /// 日配额超过阈值
    DailyThreshold,
    /// 月配额超过阈值
    MonthlyThreshold,
    /// 日配额已超额
    DailyExceeded,
    /// 月配额已超额
    MonthlyExceeded,
}

/// 配额告警
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuotaAlert {
    pub plan_id: String,
    pub alert_type: AlertType,
    pub triggered_at: DateTime<Utc>,
    pub usage_percent: f32,
    pub message: String,
}

/// 配额追踪器
pub struct QuotaTracker {
    /// 配额记录
    records: Arc<RwLock<HashMap<String, QuotaRecord>>>,
    /// 默认配额限制
    default_limits: Arc<RwLock<HashMap<String, QuotaLimit>>>,
    /// SQLite 存储（可选）
    sqlite_store: Option<Arc<SqliteStore>>,
    /// 当前活跃告警
    alerts: Arc<RwLock<HashMap<String, QuotaAlert>>>,
}

impl QuotaTracker {
    /// 创建新的配额追踪器（纯内存模式）
    pub fn new() -> Self {
        Self {
            records: Arc::new(RwLock::new(HashMap::new())),
            default_limits: Arc::new(RwLock::new(HashMap::new())),
            sqlite_store: None,
            alerts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 创建带 SQLite 持久化的配额追踪器
    pub fn with_sqlite(sqlite_store: Arc<SqliteStore>) -> Self {
        Self {
            records: Arc::new(RwLock::new(HashMap::new())),
            default_limits: Arc::new(RwLock::new(HashMap::new())),
            sqlite_store: Some(sqlite_store),
            alerts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 从 SQLite 加载现有配额数据（初始化时调用）
    pub async fn load_from_sqlite(&self, plan_ids: &[String]) -> Result<()> {
        if let Some(store) = &self.sqlite_store {
            let mut records = self.records.write().await;

            for plan_id in plan_ids {
                // 加载日配额
                let daily_used = store.get_current_quota_usage(plan_id.clone(), "daily".to_string()).await?;
                // 加载月配额
                let monthly_used = store.get_current_quota_usage(plan_id.clone(), "monthly".to_string()).await?;

                if daily_used.is_some() || monthly_used.is_some() {
                    records.insert(plan_id.clone(), QuotaRecord {
                        plan_id: plan_id.clone(),
                        daily_used: daily_used.unwrap_or(0) as u64,
                        monthly_used: monthly_used.unwrap_or(0) as u64,
                        rpm_used: 0, // RPM 是实时计数，不从 SQLite 加载
                        last_reset: Utc::now(),
                    });
                }
            }
        }
        Ok(())
    }

    /// 检查并消耗配额
    pub async fn check_and_consume(&self, plan_id: &str) -> bool {
        // 内存操作
        let (success, daily_used, monthly_used) = {
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

            (true, record.daily_used, record.monthly_used)
        };

        // 成功消耗后，持久化到 SQLite
        if success && self.sqlite_store.is_some() {
            self.persist_quota(plan_id, daily_used, monthly_used).await;
        }

        success
    }

    /// 持久化配额到 SQLite
    async fn persist_quota(&self, plan_id: &str, daily_used: u64, monthly_used: u64) {
        if let Some(store) = &self.sqlite_store {
            let now = Utc::now();

            // 日配额：当天 00:00:00 到 23:59:59
            let day_start = now.date_naive().and_hms_opt(0, 0, 0).unwrap();
            let day_end = now.date_naive().and_hms_opt(23, 59, 59).unwrap();
            let day_start_dt = DateTime::<Utc>::from_naive_utc_and_offset(day_start, Utc);
            let day_end_dt = DateTime::<Utc>::from_naive_utc_and_offset(day_end, Utc);

            // 月配额：当月第一天到最后一天
            let month_start = get_month_start(now);
            let month_end = get_month_end(now);

            // 写入日配额
            if let Err(e) = store.record_quota_usage(
                plan_id.to_string(),
                "daily".to_string(),
                daily_used as i64,
                day_start_dt,
                day_end_dt,
            ).await {
                tracing::warn!("Failed to persist daily quota: {}", e);
            }

            // 写入月配额
            if let Err(e) = store.record_quota_usage(
                plan_id.to_string(),
                "monthly".to_string(),
                monthly_used as i64,
                month_start,
                month_end,
            ).await {
                tracing::warn!("Failed to persist monthly quota: {}", e);
            }
        }
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
        // 同时持久化到 SQLite
        if let Some(store) = &self.sqlite_store {
            if let Err(e) = store.reset_quota_usage(plan_id.to_string(), "daily".to_string()).await {
                tracing::warn!("Failed to reset daily quota in SQLite: {}", e);
            }
            if let Err(e) = store.reset_quota_usage(plan_id.to_string(), "monthly".to_string()).await {
                tracing::warn!("Failed to reset monthly quota in SQLite: {}", e);
            }
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

    /// 检查并生成告警（传入阈值，如 0.8 表示 80%）
    pub async fn check_alert(&self, plan_id: &str, threshold: f32) -> Option<QuotaAlert> {
        let usage = self.get_usage(plan_id).await?;
        let limits = self.get_limits(plan_id).await?;

        let threshold = threshold.max(0.0).min(1.0);
        let mut new_alert: Option<QuotaAlert> = None;

        // 优先检查日配额（更严格）
        if let Some(daily_limit) = limits.daily {
            if daily_limit > 0 {
                let daily_pct = usage.daily_used as f32 / daily_limit as f32;
                if daily_pct >= 1.0 {
                    new_alert = Some(QuotaAlert {
                        plan_id: plan_id.to_string(),
                        alert_type: AlertType::DailyExceeded,
                        triggered_at: Utc::now(),
                        usage_percent: daily_pct,
                        message: format!(
                            "Daily quota exceeded: {} / {} requests ({:.1}%)",
                            usage.daily_used, daily_limit, daily_pct * 100.0
                        ),
                    });
                } else if daily_pct >= threshold {
                    new_alert = Some(QuotaAlert {
                        plan_id: plan_id.to_string(),
                        alert_type: AlertType::DailyThreshold,
                        triggered_at: Utc::now(),
                        usage_percent: daily_pct,
                        message: format!(
                            "Daily quota alert: {} / {} requests ({:.1}%)",
                            usage.daily_used, daily_limit, daily_pct * 100.0
                        ),
                    });
                }
            }
        }

        // 若日配额未告警，检查月配额
        if new_alert.is_none() {
            if let Some(monthly_limit) = limits.monthly {
                if monthly_limit > 0 {
                    let monthly_pct = usage.monthly_used as f32 / monthly_limit as f32;
                    if monthly_pct >= 1.0 {
                        new_alert = Some(QuotaAlert {
                            plan_id: plan_id.to_string(),
                            alert_type: AlertType::MonthlyExceeded,
                            triggered_at: Utc::now(),
                            usage_percent: monthly_pct,
                            message: format!(
                                "Monthly quota exceeded: {} / {} requests ({:.1}%)",
                                usage.monthly_used, monthly_limit, monthly_pct * 100.0
                            ),
                        });
                    } else if monthly_pct >= threshold {
                        new_alert = Some(QuotaAlert {
                            plan_id: plan_id.to_string(),
                            alert_type: AlertType::MonthlyThreshold,
                            triggered_at: Utc::now(),
                            usage_percent: monthly_pct,
                            message: format!(
                                "Monthly quota alert: {} / {} requests ({:.1}%)",
                                usage.monthly_used, monthly_limit, monthly_pct * 100.0
                            ),
                        });
                    }
                }
            }
        }

        let mut alerts = self.alerts.write().await;
        match new_alert {
            Some(ref alert) => {
                // 存储或更新告警
                alerts.insert(plan_id.to_string(), alert.clone());
                new_alert.clone()
            }
            None => {
                // 未触发告警，清除已有告警
                alerts.remove(plan_id);
                None
            }
        }
    }

    /// 获取指定 Plan 的当前告警
    pub async fn get_alert(&self, plan_id: &str) -> Option<QuotaAlert> {
        let alerts = self.alerts.read().await;
        alerts.get(plan_id).cloned()
    }

    /// 清除指定 Plan 的告警
    pub async fn clear_alert(&self, plan_id: &str) {
        let mut alerts = self.alerts.write().await;
        alerts.remove(plan_id);
    }

    /// 获取所有活跃告警
    pub async fn get_all_alerts(&self) -> Vec<QuotaAlert> {
        let alerts = self.alerts.read().await;
        alerts.values().cloned().collect()
    }
}

impl Default for QuotaTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// 获取月份开始时间
fn get_month_start(now: DateTime<Utc>) -> DateTime<Utc> {
    let naive = now.date_naive();
    let month_start = naive.with_day(1).unwrap().and_hms_opt(0, 0, 0).unwrap();
    DateTime::<Utc>::from_naive_utc_and_offset(month_start, Utc)
}

/// 获取月份结束时间（当月最后一天的 23:59:59）
fn get_month_end(now: DateTime<Utc>) -> DateTime<Utc> {
    let year = now.year();
    let month = now.month();

    // 计算下个月的第一天
    let (next_year, next_month) = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };

    // 下个月第一天 00:00:00，减一秒得到当月最后一秒 23:59:59
    let next_month_start = chrono::NaiveDate::from_ymd_opt(next_year, next_month, 1)
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap();
    let month_end = next_month_start - chrono::Duration::seconds(1);

    DateTime::<Utc>::from_naive_utc_and_offset(month_end, Utc)
}