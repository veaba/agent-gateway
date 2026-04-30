use axum::{
    extract::Request,
    http::header::HeaderValue,
    middleware::Next,
    response::Response,
};

/// 安全响应头中间件
///
/// 为所有响应添加以下安全头：
/// - `X-Content-Type-Options: nosniff`
/// - `X-Frame-Options: DENY`
/// - `X-XSS-Protection: 1; mode=block`
/// - `Referrer-Policy: strict-origin-when-cross-origin`
pub async fn security_headers(req: Request, next: Next) -> Response {
    let mut response = next.run(req).await;

    let headers = response.headers_mut();
    headers.insert("x-content-type-options", HeaderValue::from_static("nosniff"));
    headers.insert("x-frame-options", HeaderValue::from_static("DENY"));
    headers.insert("x-xss-protection", HeaderValue::from_static("1; mode=block"));
    headers.insert(
        "referrer-policy",
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );

    response
}
