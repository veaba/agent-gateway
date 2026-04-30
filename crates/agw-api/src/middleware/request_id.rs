use axum::{
    extract::Request,
    http::header::HeaderValue,
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

const REQUEST_ID_HEADER: &str = "x-request-id";

/// 为每个请求生成唯一 Request ID，并添加到响应头
///
/// Request ID 同时存入请求扩展，handler 中可通过 `req.extensions().get::<String>()` 读取。
pub async fn request_id(req: Request, next: Next) -> Response {
    let request_id = Uuid::new_v4().to_string();

    let mut req = req;
    req.extensions_mut().insert(request_id.clone());

    let mut response = next.run(req).await;

    if let Ok(value) = HeaderValue::from_str(&request_id) {
        response.headers_mut().insert(REQUEST_ID_HEADER, value);
    }

    response
}
