//! SQLite 存储

use std::path::PathBuf;
use anyhow::Result;
use rusqlite::{Connection, params};

/// SQLite 存储
pub struct SqliteStore {
    conn: Connection,
}

impl SqliteStore {
    /// 创建 SQLite 存储
    pub fn new(path: PathBuf) -> Result<Self> {
        let conn = Connection::open(&path)?;
        let store = Self { conn };
        store.init_schema()?;
        Ok(store)
    }

    /// 创建内存数据库
    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let store = Self { conn };
        store.init_schema()?;
        Ok(store)
    }

    /// 初始化数据库表
    fn init_schema(&self) -> Result<()> {
        self.conn.execute_batch(
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

            -- 创建索引
            CREATE INDEX IF NOT EXISTS idx_request_logs_plan_id ON request_logs(plan_id);
            CREATE INDEX IF NOT EXISTS idx_request_logs_created_at ON request_logs(created_at);
            CREATE INDEX IF NOT EXISTS idx_health_checks_plan_id ON health_checks(plan_id);
            CREATE INDEX IF NOT EXISTS idx_quota_usage_plan_id ON quota_usage(plan_id);
            "#
        )?;
        Ok(())
    }

    /// 记录请求日志
    pub fn log_request(
        &self,
        request_id: &str,
        plan_id: &str,
        agent_id: Option<&str>,
        model_id: &str,
        input_tokens: Option<i64>,
        output_tokens: Option<i64>,
        status_code: Option<i32>,
        error_message: Option<&str>,
        latency_ms: Option<i64>,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO request_logs (request_id, plan_id, agent_id, model_id, input_tokens, output_tokens, status_code, error_message, latency_ms) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![request_id, plan_id, agent_id, model_id, input_tokens, output_tokens, status_code, error_message, latency_ms]
        )?;
        Ok(())
    }

    /// 记录健康检查
    pub fn log_health_check(
        &self,
        plan_id: &str,
        status: &str,
        response_time_ms: Option<i64>,
        error_message: Option<&str>,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO health_checks (plan_id, status, response_time_ms, error_message) VALUES (?1, ?2, ?3, ?4)",
            params![plan_id, status, response_time_ms, error_message]
        )?;
        Ok(())
    }

    /// 获取最近的请求日志
    pub fn get_recent_logs(&self, limit: i64) -> Result<Vec<RequestLog>> {
        let mut stmt = self.conn.prepare(
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
    }
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