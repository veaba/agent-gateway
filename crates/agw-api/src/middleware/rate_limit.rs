use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use tokio::sync::RwLock;

use crate::error::ApiError;

#[derive(Clone, Copy)]
struct RateLimitEntry {
    count: usize,
    window_start: Instant,
}

/// 基于客户端标识的内存速率限制器
///
/// 注意：当前实现为进程内内存存储，多实例部署时不共享状态。
/// 生产环境高可用场景建议改用 Redis 等外部存储。
#[derive(Clone)]
pub struct RateLimiter {
    inner: Arc<RwLock<HashMap<String, RateLimitEntry>>>,
    max_requests: usize,
    window: Duration,
}

impl RateLimiter {
    /// 创建速率限制器
    ///
    /// # Arguments
    /// * `max_requests` - 每个时间窗口内允许的最大请求数
    /// * `window_secs` - 时间窗口长度（秒）
    pub fn new(max_requests: usize, window_secs: u64) -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
            max_requests,
            window: Duration::from_secs(window_secs),
        }
    }

    /// 检查 key 是否允许通过，同时更新计数
    pub async fn check(&self, key: &str) -> bool {
        let mut map = self.inner.write().await;
        let now = Instant::now();

        match map.get_mut(key) {
            Some(entry) => {
                if now.duration_since(entry.window_start) > self.window {
                    entry.count = 1;
                    entry.window_start = now;
                    true
                } else if entry.count < self.max_requests {
                    entry.count += 1;
                    true
                } else {
                    false
                }
            }
            None => {
                map.insert(
                    key.to_string(),
                    RateLimitEntry {
                        count: 1,
                        window_start: now,
                    },
                );
                true
            }
        }
    }
}

/// 速率限制中间件
///
/// 基于客户端标识进行限制（优先使用 `X-Forwarded-For`，其次 `X-Real-Ip`）。
/// `/health` 端点不受限制。
pub async fn rate_limit(
    State(limiter): State<RateLimiter>,
    req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    if req.uri().path() == "/health" {
        return Ok(next.run(req).await);
    }

    let key = req
        .headers()
        .get("x-forwarded-for")
        .or_else(|| req.headers().get("x-real-ip"))
        .and_then(|h| h.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or(s).trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    if !limiter.check(&key).await {
        return Err(ApiError::Validation(
            "Rate limit exceeded. Please try again later.".into(),
        ));
    }

    Ok(next.run(req).await)
}
