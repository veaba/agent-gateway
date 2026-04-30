//! API 中间件
//!
//! 提供以下中间件：
//! - `request_id` — 为每个请求生成唯一 ID
//! - `auth` — Bearer Token 认证（基于 `AGW_API_TOKEN` 环境变量）
//! - `rate_limit` — IP 级速率限制
//! - `timing` — 请求耗时统计
//! - `security` — 安全响应头

pub mod auth;
pub mod rate_limit;
pub mod request_id;
pub mod security;
pub mod timing;

use axum::{
    Router,
    middleware::{from_fn, from_fn_with_state},
};

use rate_limit::RateLimiter;

/// 为 Router 应用所有自定义中间件
///
/// 中间件执行顺序（请求方向，从外到内）：
/// 1. Auth（认证）
/// 2. Rate Limit（速率限制）
/// 3. Security Headers（安全头）
/// 4. Timing（计时）
/// 5. Request ID（请求 ID）
///
/// 由于 tower 的洋葱模型，`layer` 越晚添加，越先接触请求。
pub fn apply(router: Router, rate_limiter: RateLimiter) -> Router {
    router
        .layer(from_fn(request_id::request_id))
        .layer(from_fn(timing::request_timing))
        .layer(from_fn(security::security_headers))
        .layer(from_fn_with_state(rate_limiter, rate_limit::rate_limit))
        .layer(from_fn(auth::require_auth))
}
