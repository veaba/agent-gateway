//! SQLite 存储

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use anyhow::Result;
use rusqlite::{Connection, params};
use chrono::{DateTime, Utc};

/// SQLite 存储
#[derive(Clone)]
pub struct SqliteStore {
    conn: Arc<Mutex<Connection>>,
}

// 显式实现 Send + Sync，因为 Mutex<Connection> 是 Send + Sync
unsafe impl Send for SqliteStore {}
unsafe impl Sync for SqliteStore {}

impl SqliteStore {
    /// 创建 SQLite 存储
    pub fn new(path: PathBuf) -> Result<Self> {
        let conn = Connection::open(&path)?;
        let store = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        store.init_schema()?;
        Ok(store)
    }

    /// 创建内存数据库
    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let store = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        store.init_schema()?;
        Ok(store)
    }

    /// 初始化数据库表
    fn init_schema(&self) -> Result<()> {
        let conn = self.conn.lock()
            .map_err(|_| anyhow::anyhow!("Cannot acquire connection lock"))?;

        conn.execute_batch(
            r#"
            -- 请求日志表
            CREATE TABLE IF NOT EXISTS request_logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                request_id TEXT NOT NULL,
                plan_id TEXT NOT NULL,
                agent_id TEXT,
                model_id TEXT NOT NULL,
                input_tokens INTEGER,
                output_tokens INTEGER,
                status_code INTEGER,
                error_message TEXT,
                latency_ms INTEGER,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );

            -- 健康检查表
            CREATE TABLE IF NOT EXISTS health_checks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                plan_id TEXT NOT NULL,
                status TEXT NOT NULL,
                response_time_ms INTEGER,
                error_message TEXT,
                checked_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );

            -- 配额使用表（用于更精确的追踪）
            CREATE TABLE IF NOT EXISTS quota_usage (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                plan_id TEXT NOT NULL,
                quota_type TEXT NOT NULL,
                used INTEGER NOT NULL,
                period_start DATETIME NOT NULL,
                period_end DATETIME NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                UNIQUE(plan_id, quota_type, period_start)
            );

            -- Fallback 事件追踪表
            CREATE TABLE IF NOT EXISTS fallback_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                request_id TEXT NOT NULL,
                triggered_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                trigger_code INTEGER,
                trigger_type TEXT NOT NULL,
                source_plan_id TEXT NOT NULL,
                source_provider_id TEXT,
                target_plan_id TEXT,
                target_provider_id TEXT,
                attempt_index INTEGER DEFAULT 0,
                protocol_converted BOOLEAN DEFAULT 0,
                error_message TEXT,
                latency_ms INTEGER,
                recovered_at DATETIME,
                recovery_latency_ms INTEGER,
                resolved BOOLEAN DEFAULT 0
            );

            -- 创建索引
            CREATE INDEX IF NOT EXISTS idx_request_logs_plan_id ON request_logs(plan_id);
            CREATE INDEX IF NOT EXISTS idx_request_logs_created_at ON request_logs(created_at);
            CREATE INDEX IF NOT EXISTS idx_health_checks_plan_id ON health_checks(plan_id);
            CREATE INDEX IF NOT EXISTS idx_quota_usage_plan_id ON quota_usage(plan_id);
            CREATE INDEX IF NOT EXISTS idx_fallback_events_source_plan ON fallback_events(source_plan_id);
            CREATE INDEX IF NOT EXISTS idx_fallback_events_source_provider ON fallback_events(source_provider_id);
            CREATE INDEX IF NOT EXISTS idx_fallback_events_triggered_at ON fallback_events(triggered_at);
            CREATE INDEX IF NOT EXISTS idx_fallback_events_resolved ON fallback_events(resolved);
            "#
        )?;
        Ok(())
    }

    /// 记录请求日志
    pub async fn log_request(&self, params: RequestLogParams) -> Result<()> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock()
                .map_err(|_| anyhow::anyhow!("Cannot acquire connection lock"))?;
            conn.execute(
                "INSERT INTO request_logs (request_id, plan_id, agent_id, model_id, input_tokens, output_tokens, status_code, error_message, latency_ms) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                rusqlite::params![
                    &params.request_id,
                    &params.plan_id,
                    &params.agent_id,
                    &params.model_id,
                    &params.input_tokens,
                    &params.output_tokens,
                    &params.status_code,
                    &params.error_message,
                    &params.latency_ms
                ]
            )?;
            Ok(())
        }).await?
    }

    /// 记录健康检查
    pub async fn log_health_check(
        &self,
        plan_id: String,
        status: String,
        response_time_ms: Option<i64>,
        error_message: Option<String>,
    ) -> Result<()> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock()
                .map_err(|_| anyhow::anyhow!("Cannot acquire connection lock"))?;
            conn.execute(
                "INSERT INTO health_checks (plan_id, status, response_time_ms, error_message) VALUES (?1, ?2, ?3, ?4)",
                params![plan_id, status, response_time_ms, error_message]
            )?;
            Ok(())
        }).await?
    }

    /// 获取最近的请求日志
    pub async fn get_recent_logs(&self, limit: i64) -> Result<Vec<RequestLog>> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock()
                .map_err(|_| anyhow::anyhow!("Cannot acquire connection lock"))?;
            let mut stmt = conn.prepare(
                "SELECT request_id, plan_id, agent_id, model_id, status_code, latency_ms, created_at FROM request_logs ORDER BY created_at DESC LIMIT ?1"
            )?;

            let logs = stmt.query_map([limit], |row| {
                Ok(RequestLog {
                    request_id: row.get(0)?,
                    plan_id: row.get(1)?,
                    agent_id: row.get(2)?,
                    model_id: row.get(3)?,
                    status_code: row.get(4)?,
                    latency_ms: row.get(5)?,
                    created_at: row.get(6)?,
                })
            })?.collect::<Result<Vec<_>, _>>()?;

            Ok(logs)
        }).await?
    }

    /// 获取请求统计
    pub async fn get_request_stats(
        &self,
        plan_id: Option<String>,
        since: Option<DateTime<Utc>>,
    ) -> Result<RequestStats> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock()
                .map_err(|_| anyhow::anyhow!("Cannot acquire connection lock"))?;

            let mut where_clause = String::from("1=1");
            let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

            if let Some(pid) = plan_id {
                where_clause.push_str(" AND plan_id = ?");
                params_vec.push(Box::new(pid));
            }

            if let Some(since_dt) = since {
                where_clause.push_str(" AND created_at >= ?");
                params_vec.push(Box::new(since_dt.to_rfc3339()));
            }

            let sql = format!(
                "SELECT COUNT(*) as total, AVG(latency_ms) as avg_latency, SUM(input_tokens) as total_input_tokens, SUM(output_tokens) as total_output_tokens, SUM(CASE WHEN status_code >= 400 THEN 1 ELSE 0 END) as error_count FROM request_logs WHERE {}",
                where_clause
            );

            let mut stmt = conn.prepare(&sql)?;
            let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();

            let stats = stmt.query_row(params_refs.as_slice(), |row| {
                Ok(RequestStats {
                    total_requests: row.get::<_, i64>(0)?,
                    avg_latency_ms: row.get::<_, Option<f64>>(1)?,
                    total_input_tokens: row.get::<_, Option<i64>>(2)?,
                    total_output_tokens: row.get::<_, Option<i64>>(3)?,
                    error_count: row.get::<_, i64>(4)?,
                })
            })?;

            Ok(stats)
        }).await?
    }

    /// 获取健康检查历史
    pub async fn get_health_history(
        &self,
        plan_id: Option<String>,
        limit: i64,
    ) -> Result<Vec<HealthCheckRecord>> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock()
                .map_err(|_| anyhow::anyhow!("Cannot acquire connection lock"))?;

            let (sql, params_vec): (String, Vec<Box<dyn rusqlite::ToSql>>) = if let Some(pid) = plan_id {
                (
                    "SELECT plan_id, status, response_time_ms, error_message, checked_at FROM health_checks WHERE plan_id = ? ORDER BY checked_at DESC LIMIT ?".to_string(),
                    vec![Box::new(pid), Box::new(limit)],
                )
            } else {
                (
                    "SELECT plan_id, status, response_time_ms, error_message, checked_at FROM health_checks ORDER BY checked_at DESC LIMIT ?".to_string(),
                    vec![Box::new(limit)],
                )
            };

            let mut stmt = conn.prepare(&sql)?;
            let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();

            let records = stmt.query_map(params_refs.as_slice(), |row| {
                Ok(HealthCheckRecord {
                    plan_id: row.get(0)?,
                    status: row.get(1)?,
                    response_time_ms: row.get(2)?,
                    error_message: row.get(3)?,
                    checked_at: row.get(4)?,
                })
            })?.collect::<Result<Vec<_>, _>>()?;

            Ok(records)
        }).await?
    }

    /// 记录配额使用
    pub async fn record_quota_usage(
        &self,
        plan_id: String,
        quota_type: String,
        used: i64,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
    ) -> Result<()> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock()
                .map_err(|_| anyhow::anyhow!("Cannot acquire connection lock"))?;
            conn.execute(
                "INSERT INTO quota_usage (plan_id, quota_type, used, period_start, period_end) VALUES (?1, ?2, ?3, ?4, ?5) ON CONFLICT(plan_id, quota_type, period_start) DO UPDATE SET used = excluded.used",
                params![plan_id, quota_type, used, period_start.to_rfc3339(), period_end.to_rfc3339()]
            )?;
            Ok(())
        }).await?
    }

    /// 获取配额使用统计
    pub async fn get_quota_usage(&self, plan_id: String) -> Result<Vec<QuotaUsageRecord>> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock()
                .map_err(|_| anyhow::anyhow!("Cannot acquire connection lock"))?;
            let mut stmt = conn.prepare(
                "SELECT plan_id, quota_type, used, period_start, period_end, created_at FROM quota_usage WHERE plan_id = ?1 ORDER BY period_start DESC"
            )?;

            let records = stmt.query_map([plan_id], |row| {
                Ok(QuotaUsageRecord {
                    plan_id: row.get(0)?,
                    quota_type: row.get(1)?,
                    used: row.get(2)?,
                    period_start: row.get(3)?,
                    period_end: row.get(4)?,
                    created_at: row.get(5)?,
                })
            })?.collect::<Result<Vec<_>, _>>()?;

            Ok(records)
        }).await?
    }

    /// 获取当前周期的配额使用
    pub async fn get_current_quota_usage(
        &self,
        plan_id: String,
        quota_type: String,
    ) -> Result<Option<i64>> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock()
                .map_err(|_| anyhow::anyhow!("Cannot acquire connection lock"))?;
            let now = Utc::now().to_rfc3339();

            let mut stmt = conn.prepare(
                "SELECT used FROM quota_usage WHERE plan_id = ?1 AND quota_type = ?2 AND period_start <= ?3 AND period_end >= ?3 ORDER BY period_start DESC LIMIT 1"
            )?;

            let result = stmt.query_row(params![plan_id, quota_type, now], |row| {
                row.get::<_, i64>(0)
            });

            match result {
                Ok(used) => Ok(Some(used)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(e.into()),
            }
        }).await?
    }

    /// 重置指定 plan 在当前周期的配额记录（设为 0）
    pub async fn reset_quota_usage(
        &self,
        plan_id: String,
        quota_type: String,
    ) -> Result<()> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock()
                .map_err(|_| anyhow::anyhow!("Cannot acquire connection lock"))?;
            let now = Utc::now().to_rfc3339();

            conn.execute(
                "UPDATE quota_usage SET used = 0 WHERE plan_id = ?1 AND quota_type = ?2 AND period_start <= ?3 AND period_end >= ?3",
                params![plan_id, quota_type, now]
            )?;
            Ok(())
        }).await?
    }

    // ============================================================================
    // Fallback 事件追踪
    // ============================================================================

    /// 记录 Fallback 事件
    pub async fn log_fallback_event(&self, params: FallbackEventParams) -> Result<i64> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock()
                .map_err(|_| anyhow::anyhow!("Cannot acquire connection lock"))?;
            conn.execute(
                "INSERT INTO fallback_events (
                    request_id, trigger_code, trigger_type, source_plan_id, source_provider_id,
                    target_plan_id, target_provider_id, attempt_index, protocol_converted,
                    error_message, latency_ms
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                rusqlite::params![
                    &params.request_id,
                    &params.trigger_code,
                    &params.trigger_type,
                    &params.source_plan_id,
                    &params.source_provider_id,
                    &params.target_plan_id,
                    &params.target_provider_id,
                    &params.attempt_index,
                    &params.protocol_converted,
                    &params.error_message,
                    &params.latency_ms
                ]
            )?;
            let id = conn.last_insert_rowid();
            Ok(id)
        }).await?
    }

    /// 标记 Fallback 事件已恢复
    pub async fn mark_fallback_recovered(
        &self,
        event_id: i64,
        recovery_latency_ms: i64,
    ) -> Result<()> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock()
                .map_err(|_| anyhow::anyhow!("Cannot acquire connection lock"))?;
            conn.execute(
                "UPDATE fallback_events SET resolved = 1, recovered_at = CURRENT_TIMESTAMP, recovery_latency_ms = ?1 WHERE id = ?2",
                params![recovery_latency_ms, event_id]
            )?;
            Ok(())
        }).await?
    }

    /// 按 source_plan 和最近时间查找未解决的 fallback 事件并标记恢复
    pub async fn resolve_fallback_events_by_plan(
        &self,
        plan_id: String,
    ) -> Result<usize> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock()
                .map_err(|_| anyhow::anyhow!("Cannot acquire connection lock"))?;
            let now = Utc::now().to_rfc3339();
            let rows = conn.execute(
                "UPDATE fallback_events SET resolved = 1, recovered_at = ?1 WHERE source_plan_id = ?2 AND resolved = 0",
                params![now, plan_id]
            )?;
            Ok(rows)
        }).await?
    }

    /// 获取 Fallback 事件列表
    pub async fn get_fallback_events(
        &self,
        plan_id: Option<String>,
        provider_id: Option<String>,
        limit: i64,
    ) -> Result<Vec<FallbackEvent>> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock()
                .map_err(|_| anyhow::anyhow!("Cannot acquire connection lock"))?;

            let mut sql = String::from(
                "SELECT id, request_id, triggered_at, trigger_code, trigger_type,
                 source_plan_id, source_provider_id, target_plan_id, target_provider_id,
                 attempt_index, protocol_converted, error_message, latency_ms,
                 recovered_at, recovery_latency_ms, resolved
                 FROM fallback_events WHERE 1=1"
            );
            let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

            if let Some(pid) = plan_id {
                sql.push_str(" AND source_plan_id = ?");
                params_vec.push(Box::new(pid));
            }
            if let Some(prov_id) = provider_id {
                sql.push_str(" AND source_provider_id = ?");
                params_vec.push(Box::new(prov_id));
            }
            sql.push_str(" ORDER BY triggered_at DESC LIMIT ?");
            params_vec.push(Box::new(limit));

            let mut stmt = conn.prepare(&sql)?;
            let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();

            let events = stmt.query_map(params_refs.as_slice(), |row| {
                Ok(FallbackEvent {
                    id: row.get(0)?,
                    request_id: row.get(1)?,
                    triggered_at: row.get(2)?,
                    trigger_code: row.get(3)?,
                    trigger_type: row.get(4)?,
                    source_plan_id: row.get(5)?,
                    source_provider_id: row.get(6)?,
                    target_plan_id: row.get(7)?,
                    target_provider_id: row.get(8)?,
                    attempt_index: row.get(9)?,
                    protocol_converted: row.get::<_, i32>(10)? != 0,
                    error_message: row.get(11)?,
                    latency_ms: row.get(12)?,
                    recovered_at: row.get(13)?,
                    recovery_latency_ms: row.get(14)?,
                    resolved: row.get::<_, i32>(15)? != 0,
                })
            })?.collect::<Result<Vec<_>, _>>()?;

            Ok(events)
        }).await?
    }

    /// 获取 Fallback 总体统计
    pub async fn get_fallback_stats(
        &self,
        since: Option<DateTime<Utc>>,
    ) -> Result<FallbackStats> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock()
                .map_err(|_| anyhow::anyhow!("Cannot acquire connection lock"))?;

            let mut where_clause = String::from("1=1");
            let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

            if let Some(since_dt) = since {
                where_clause.push_str(" AND triggered_at >= ?");
                params_vec.push(Box::new(since_dt.to_rfc3339()));
            }

            // 总体统计
            let sql = format!(
                "SELECT COUNT(*) as total, COALESCE(SUM(CASE WHEN resolved = 1 THEN 1 ELSE 0 END), 0) as resolved, AVG(recovery_latency_ms) as avg_recovery FROM fallback_events WHERE {}",
                where_clause
            );
            let mut stmt = conn.prepare(&sql)?;
            let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();

            let (total, resolved, avg_recovery): (i64, i64, Option<f64>) = stmt.query_row(
                params_refs.as_slice(),
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            )?;

            // 按触发类型统计
            let type_sql = format!(
                "SELECT trigger_type, COUNT(*) as count FROM fallback_events WHERE {} GROUP BY trigger_type ORDER BY count DESC",
                where_clause
            );
            let mut type_stmt = conn.prepare(&type_sql)?;
            let type_params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();

            let by_trigger_type = type_stmt.query_map(type_params_refs.as_slice(), |row| {
                Ok(TriggerTypeCount {
                    trigger_type: row.get(0)?,
                    count: row.get(1)?,
                })
            })?.collect::<Result<Vec<_>, _>>()?;

            Ok(FallbackStats {
                total_events: total,
                total_resolved: resolved,
                total_unresolved: total - resolved,
                avg_recovery_latency_ms: avg_recovery,
                by_trigger_type,
            })
        }).await?
    }

    /// 获取 Provider 性能指标（基于 request_logs 和 fallback_events）
    pub async fn get_provider_performance_metrics(
        &self,
        since: Option<DateTime<Utc>>,
    ) -> Result<Vec<ProviderPerformanceMetrics>> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock()
                .map_err(|_| anyhow::anyhow!("Cannot acquire connection lock"))?;

            let since_clause = if let Some(since_dt) = since {
                format!(" AND created_at >= '{}'", since_dt.to_rfc3339())
            } else {
                String::new()
            };
            let fb_since_clause = if let Some(since_dt) = since {
                format!(" AND triggered_at >= '{}'", since_dt.to_rfc3339())
            } else {
                String::new()
            };

            // 按 provider 统计请求数、错误数、平均延迟
            let mut stmt = conn.prepare(&format!(
                "SELECT plan_id, COUNT(*) as total, AVG(latency_ms) as avg_latency,
                 SUM(CASE WHEN status_code >= 400 OR status_code = 0 THEN 1 ELSE 0 END) as errors
                 FROM request_logs WHERE 1=1 {} GROUP BY plan_id",
                since_clause
            ))?;

            let plan_stats: std::collections::HashMap<String, (i64, Option<f64>, i64)> = stmt
                .query_map([], |row| {
                    let plan_id: String = row.get(0)?;
                    let total: i64 = row.get(1)?;
                    let avg_latency: Option<f64> = row.get(2)?;
                    let errors: i64 = row.get(3)?;
                    Ok((plan_id, (total, avg_latency, errors)))
                })?.collect::<Result<std::collections::HashMap<_, _>, _>>()?;

            // 按 source_provider_id 统计 fallback 事件
            let mut fb_stmt = conn.prepare(&format!(
                "SELECT source_provider_id, COUNT(*) as fb_count,
                 AVG(recovery_latency_ms) as avg_recovery,
                 MAX(triggered_at) as last_fallback
                 FROM fallback_events WHERE source_provider_id IS NOT NULL {} GROUP BY source_provider_id",
                fb_since_clause
            ))?;

            let _provider_fallbacks: std::collections::HashMap<String, (i64, Option<f64>, Option<String>)> = fb_stmt
                .query_map([], |row| {
                    let provider_id: String = row.get(0)?;
                    let count: i64 = row.get(1)?;
                    let avg_recovery: Option<f64> = row.get(2)?;
                    let last: Option<String> = row.get(3)?;
                    Ok((provider_id, (count, avg_recovery, last)))
                })?.collect::<Result<std::collections::HashMap<_, _>, _>>()?;

            // 注意：这里需要外部传入 provider_name 映射，因此返回部分数据由调用方补充
            let mut metrics = Vec::new();
            for (plan_id, (total, avg_latency, errors)) in plan_stats {
                let success_rate = if total > 0 {
                    (total - errors) as f64 / total as f64
                } else { 1.0 };
                let avg_latency_ms = avg_latency.unwrap_or(0.0);

                // 查找该 plan 是否有 fallback 记录（plan_id 作为 source_plan_id 的关联）
                let mut fb_for_plan = conn.prepare(&format!(
                    "SELECT COUNT(*) FROM fallback_events WHERE source_plan_id = ?1 {}",
                    fb_since_clause
                ))?;
                let fb_count: i64 = fb_for_plan.query_row(params![&plan_id], |row| row.get(0))?;

                metrics.push(ProviderPerformanceMetrics {
                    provider_id: plan_id.clone(), // 暂时用 plan_id，外部调用时会替换为 provider_id
                    provider_name: String::new(),
                    total_requests: total,
                    fallback_events: fb_count,
                    fallback_rate: if total > 0 { fb_count as f64 / total as f64 } else { 0.0 },
                    avg_latency_ms,
                    success_rate,
                    estimated_recovery_time_ms: None,
                    last_fallback_at: None,
                    health_score: calculate_health_score(success_rate, avg_latency_ms, fb_count as f64 / (total as f64).max(1.0)),
                });
            }

            Ok(metrics)
        }).await?
    }
}

/// 计算 Provider 健康评分 (0-100)
fn calculate_health_score(success_rate: f64, avg_latency_ms: f64, fallback_rate: f64) -> f64 {
    let success_score = success_rate * 40.0; // 成功率占 40 分
    let latency_score = if avg_latency_ms < 200.0 { 30.0 }
        else if avg_latency_ms < 500.0 { 20.0 }
        else if avg_latency_ms < 1000.0 { 10.0 }
        else { 0.0 }; // 延迟占 30 分
    let stability_score = (1.0 - fallback_rate.min(1.0)) * 30.0; // 稳定性占 30 分
    (success_score + latency_score + stability_score).max(0.0).min(100.0)
}

/// 请求日志
#[derive(Debug)]
pub struct RequestLog {
    pub request_id: String,
    pub plan_id: String,
    pub agent_id: Option<String>,
    pub model_id: String,
    pub status_code: Option<i32>,
    pub latency_ms: Option<i64>,
    pub created_at: String,
}

/// 请求日志参数
#[derive(Debug, Clone)]
pub struct RequestLogParams {
    pub request_id: String,
    pub plan_id: String,
    pub agent_id: Option<String>,
    pub model_id: String,
    pub input_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
    pub status_code: Option<i32>,
    pub error_message: Option<String>,
    pub latency_ms: Option<i64>,
}

/// 请求统计
#[derive(Debug, Clone)]
pub struct RequestStats {
    pub total_requests: i64,
    pub avg_latency_ms: Option<f64>,
    pub total_input_tokens: Option<i64>,
    pub total_output_tokens: Option<i64>,
    pub error_count: i64,
}

/// 健康检查记录
#[derive(Debug, Clone)]
pub struct HealthCheckRecord {
    pub plan_id: String,
    pub status: String,
    pub response_time_ms: Option<i64>,
    pub error_message: Option<String>,
    pub checked_at: String,
}

/// 配额使用记录
#[derive(Debug, Clone)]
pub struct QuotaUsageRecord {
    pub plan_id: String,
    pub quota_type: String,
    pub used: i64,
    pub period_start: String,
    pub period_end: String,
    pub created_at: String,
}

/// Fallback 事件
#[derive(Debug, Clone)]
pub struct FallbackEvent {
    pub id: i64,
    pub request_id: String,
    pub triggered_at: String,
    pub trigger_code: Option<i32>,
    pub trigger_type: String,
    pub source_plan_id: String,
    pub source_provider_id: Option<String>,
    pub target_plan_id: Option<String>,
    pub target_provider_id: Option<String>,
    pub attempt_index: i32,
    pub protocol_converted: bool,
    pub error_message: Option<String>,
    pub latency_ms: Option<i64>,
    pub recovered_at: Option<String>,
    pub recovery_latency_ms: Option<i64>,
    pub resolved: bool,
}

/// Fallback 事件记录参数
#[derive(Debug, Clone)]
pub struct FallbackEventParams {
    pub request_id: String,
    pub trigger_code: Option<i32>,
    pub trigger_type: String,
    pub source_plan_id: String,
    pub source_provider_id: Option<String>,
    pub target_plan_id: Option<String>,
    pub target_provider_id: Option<String>,
    pub attempt_index: i32,
    pub protocol_converted: bool,
    pub error_message: Option<String>,
    pub latency_ms: Option<i64>,
}

/// Fallback 触发类型统计
#[derive(Debug, Clone)]
pub struct TriggerTypeCount {
    pub trigger_type: String,
    pub count: i64,
}

/// Fallback 总体统计
#[derive(Debug, Clone)]
pub struct FallbackStats {
    pub total_events: i64,
    pub total_resolved: i64,
    pub total_unresolved: i64,
    pub avg_recovery_latency_ms: Option<f64>,
    pub by_trigger_type: Vec<TriggerTypeCount>,
}

/// Provider 性能指标
#[derive(Debug, Clone)]
pub struct ProviderPerformanceMetrics {
    pub provider_id: String,
    pub provider_name: String,
    pub total_requests: i64,
    pub fallback_events: i64,
    pub fallback_rate: f64,
    pub avg_latency_ms: f64,
    pub success_rate: f64,
    pub estimated_recovery_time_ms: Option<i64>,
    pub last_fallback_at: Option<String>,
    pub health_score: f64,
}