use std::time::Instant;

use axum::{
    extract::Request,
    http::header::HeaderValue,
    middleware::Next,
    response::Response,
};

const TIMING_HEADER: &str = "x-response-time-ms";

/// 请求计时中间件
///
/// 在响应头中添加 `x-response-time-ms`，表示请求处理耗时（毫秒）。
pub async fn request_timing(req: Request, next: Next) -> Response {
    let start = Instant::now();
    let mut response = next.run(req).await;
    let elapsed = start.elapsed().as_millis() as u64;

    if let Ok(value) = HeaderValue::from_str(&elapsed.to_string()) {
        response.headers_mut().insert(TIMING_HEADER, value);
    }

    response
}
