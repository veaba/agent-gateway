use axum::{extract::Request, middleware::Next, response::Response};

use crate::error::ApiError;

const AUTH_HEADER: &str = "authorization";
const BEARER_PREFIX: &str = "Bearer ";

/// 获取期望的 API token（从环境变量 `AGW_API_TOKEN`）
fn get_expected_token() -> Option<String> {
    std::env::var("AGW_API_TOKEN").ok()
}

/// 认证中间件
///
/// 行为：
/// - `/health` 端点始终公开访问，不检查认证。
/// - 如果未设置 `AGW_API_TOKEN` 环境变量，视为开发模式，跳过认证。
/// - 否则要求请求头中包含 `Authorization: Bearer <token>`。
pub async fn require_auth(req: Request, next: Next) -> Result<Response, ApiError> {
    if req.uri().path() == "/health" {
        return Ok(next.run(req).await);
    }

    let expected = match get_expected_token() {
        Some(token) => token,
        None => return Ok(next.run(req).await),
    };

    let auth_header = req
        .headers()
        .get(AUTH_HEADER)
        .and_then(|h| h.to_str().ok());

    match auth_header {
        Some(header) if header.starts_with(BEARER_PREFIX) => {
            let token = &header[BEARER_PREFIX.len()..];
            if token == expected {
                Ok(next.run(req).await)
            } else {
                Err(ApiError::Validation("Invalid API token".into()))
            }
        }
        _ => Err(ApiError::Validation(
            "Missing or invalid Authorization header. Expected: Bearer <token>".into(),
        )),
    }
}
