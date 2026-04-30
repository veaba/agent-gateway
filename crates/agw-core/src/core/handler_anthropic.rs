//! Anthropic 处理器

use std::sync::Arc;
use hyper::Request;
use axum::{
    extract::State,
    response::Response,
    body::Body,
};

use crate::core::GatewayState;

/// 处理 Anthropic Messages API 请求
pub async fn handle_anthropic_request(
    _state: State<Arc<GatewayState>>,
    _request: Request<Body>,
) -> Response {
    tracing::debug!("Handling Anthropic request");

    // TODO: 实现完整的 Anthropic 请求处理
    // 1. 解析请求体
    // 2. 验证必需字段
    // 3. 确定使用的模型
    // 4. 配额检查
    // 5. 转发到后端 Provider
    // 6. SSE 流式响应处理

    Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .body(Body::from(r#"{"type":"message","id":"msg_001","role":"assistant","content":[{"type":"text","text":"Hello from agent-gateway"}]}"#))
        .unwrap()
}