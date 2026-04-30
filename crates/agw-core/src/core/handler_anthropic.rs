//! Anthropic 处理器

use std::sync::Arc;

use axum::{
    extract::{State, Request},
    response::Response,
    body::Body,
};
use http_body_util::BodyExt;
use serde_json::Value;

use crate::core::GatewayState;
use crate::model::UserPlan;
use crate::storage::{RequestLogParams, FallbackEventParams};

/// 处理 Anthropic Messages API 请求
pub async fn handle_anthropic_request(
    state: State<Arc<GatewayState>>,
    request: Request,
) -> Response {
    let start_time = std::time::Instant::now();

    // 1. 先克隆 headers（collect 会消费 request）
    let headers = request.headers().clone();

    // 2. 解析请求体
    let body_bytes = match request.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(e) => {
            tracing::error!("Failed to collect request body: {}", e);
            return create_error_response(400, "Failed to parse request body");
        }
    };

    let request_json: Value = match serde_json::from_slice(&body_bytes) {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("Failed to parse JSON: {}", e);
            return create_error_response(400, "Invalid JSON body");
        }
    };

    tracing::debug!("Anthropic request: model={}", request_json.get("model").and_then(|v| v.as_str()).unwrap_or("unknown"));

    // 3. 解析 API Key 从 Authorization header
    let auth_header = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim_start_matches("Bearer ").to_string());

    // 4. 解析请求头中的 x-plan-id
    let plan_id_from_header = headers
        .get("x-plan-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    // 5. 解析请求头中的 x-api-key
    let api_key_from_header = headers
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    // 5. 查找匹配的 UserPlan
    let user_plan = match resolve_user_plan(
        &state,
        plan_id_from_header.as_deref(),
        auth_header.as_deref(),
        api_key_from_header.as_deref(),
    ).await {
        Some(plan) => plan,
        None => {
            tracing::warn!("No matching plan found for request");
            return create_error_response(401, "No matching plan found. Please configure a plan with API key.");
        }
    };

    tracing::info!("Using plan: {} ({}) for Anthropic request", user_plan.name, user_plan.id);

    // 6. 检查配额
    let quota_allowed = state.quota_tracker.check_and_consume(&user_plan.id).await;
    if !quota_allowed {
        tracing::warn!("Quota exceeded for plan: {}", user_plan.id);
        return create_error_response(429, "Quota exceeded. Please upgrade your plan.");
    }

    // 7. 获取 Provider 模板
    let provider = match state.provider_engine.get_provider(&user_plan.provider_id).await {
        Some(p) => p,
        None => {
            tracing::error!("Provider not found: {}", user_plan.provider_id);
            return create_error_response(500, "Provider not configured");
        }
    };

    // 8. 获取模型 ID
    let model_id = request_json.get("model")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    // 9. 构建目标 URL
    let target_url = build_target_url(&provider, &user_plan, "/v1/messages");
    tracing::debug!("Forwarding to: {}", target_url);

    // 10. 转发请求
    let forwarder = crate::core::Forwarder::new();

    // 构建转发的请求
    let mut req_builder = forwarder.client
        .request(hyper::Method::POST, &target_url);

    // 添加必要的 headers
    req_builder = req_builder
        .header("Content-Type", "application/json")
        .header("x-api-key", &user_plan.api_key)
        .header("anthropic-version", "2023-06-01");

    // 复制其他必要 headers
    for (name, value) in request.headers() {
        let name_str = name.as_str();
        if !["host", "content-length", "authorization"].contains(&name_str) {
            if let Ok(v) = value.to_str() {
                req_builder = req_builder.header(name_str, v);
            }
        }
    }

    // 添加请求体
    req_builder = req_builder.body(body_bytes.to_vec());

    // 11. 发送请求并处理响应
    match req_builder.send().await {
        Ok(response) => {
            let status = response.status();
            let elapsed = start_time.elapsed().as_millis();

            tracing::info!("Request completed in {}ms with status {}", elapsed, status);

            // 记录请求日志
            let request_id = uuid::Uuid::new_v4().to_string();
            if let Some(ref sqlite) = state.sqlite_store {
                let params = RequestLogParams {
                    request_id: request_id.clone(),
                    plan_id: user_plan.id.clone(),
                    agent_id: None,
                    model_id: model_id.clone(),
                    input_tokens: None,
                    output_tokens: None,
                    status_code: Some(status.as_u16() as i32),
                    error_message: None,
                    latency_ms: Some(elapsed as i64),
                };
                let _ = sqlite.log_request(&params).await;
            }

            // Fallback 事件追踪
            if let Some(ref sqlite) = state.sqlite_store {
                let status_u16 = status.as_u16();
                if status_u16 == 429 {
                    let _ = sqlite.log_fallback_event(FallbackEventParams {
                        request_id: request_id.clone(),
                        trigger_code: Some(status_u16 as i32),
                        trigger_type: "rate_limit".to_string(),
                        source_plan_id: user_plan.id.clone(),
                        source_provider_id: Some(user_plan.provider_id.clone()),
                        target_plan_id: None,
                        target_provider_id: None,
                        attempt_index: 0,
                        protocol_converted: false,
                        error_message: Some("Rate limit exceeded".to_string()),
                        latency_ms: Some(elapsed as i64),
                    }).await;
                } else if status_u16 >= 500 {
                    let _ = sqlite.log_fallback_event(FallbackEventParams {
                        request_id: request_id.clone(),
                        trigger_code: Some(status_u16 as i32),
                        trigger_type: "server_error".to_string(),
                        source_plan_id: user_plan.id.clone(),
                        source_provider_id: Some(user_plan.provider_id.clone()),
                        target_plan_id: None,
                        target_provider_id: None,
                        attempt_index: 0,
                        protocol_converted: false,
                        error_message: Some(format!("Server error: {}", status_u16)),
                        latency_ms: Some(elapsed as i64),
                    }).await;
                } else if status_u16 < 400 {
                    // 请求成功，尝试将该 plan 的未解决 fallback 事件标记为恢复
                    let _ = sqlite.resolve_fallback_events_by_plan(user_plan.id.clone()).await;
                }
            }

            // 构建响应 - 先复制 headers，再获取 body (bytes() 会消费 response)
            let response_headers = response.headers().clone();
            let body = response.bytes().await.unwrap_or_default();
            let mut builder = Response::builder()
                .status(status.as_u16())
                .header("Content-Type", "application/json");

            // 复制必要的 headers
            for (name, value) in response_headers.iter() {
                let name_str = name.as_str();
                if !["content-encoding", "transfer-encoding", "connection"].contains(&name_str) {
                    if let Ok(v) = value.to_str() {
                        builder = builder.header(name_str, v);
                    }
                }
            }

            builder.body(Body::from(body)).unwrap_or_else(|_| create_error_response(500, "Failed to build response"))
        }
        Err(e) => {
            tracing::error!("Forward request failed: {}", e);
            let request_id = uuid::Uuid::new_v4().to_string();
            if let Some(ref sqlite) = state.sqlite_store {
                let params = RequestLogParams {
                    request_id: request_id.clone(),
                    plan_id: user_plan.id.clone(),
                    agent_id: None,
                    model_id: model_id.clone(),
                    input_tokens: None,
                    output_tokens: None,
                    status_code: Some(0),
                    error_message: Some(e.to_string()),
                    latency_ms: Some(start_time.elapsed().as_millis() as i64),
                };
                let _ = sqlite.log_request(&params).await;

                // 记录 fallback 事件：连接失败
                let trigger_type = if e.is_timeout() {
                    "timeout"
                } else {
                    "connection_failure"
                };
                let _ = sqlite.log_fallback_event(FallbackEventParams {
                    request_id,
                    trigger_code: None,
                    trigger_type: trigger_type.to_string(),
                    source_plan_id: user_plan.id.clone(),
                    source_provider_id: Some(user_plan.provider_id.clone()),
                    target_plan_id: None,
                    target_provider_id: None,
                    attempt_index: 0,
                    protocol_converted: false,
                    error_message: Some(e.to_string()),
                    latency_ms: Some(start_time.elapsed().as_millis() as i64),
                }).await;
            }
            create_error_response(502, "Failed to forward request to provider")
        }
    }
}

/// 解析请求并确定要使用的 UserPlan
async fn resolve_user_plan(
    state: &Arc<GatewayState>,
    plan_id: Option<&str>,
    auth_key: Option<&str>,
    api_key: Option<&str>,
) -> Option<UserPlan> {
    // 1. 优先使用显式指定的 plan_id
    if let Some(id) = plan_id {
        if let Some(plan) = state.plan_manager.get(id).await {
            return Some(plan);
        }
    }

    // 2. 尝试通过 API Key 匹配
    if let Some(key) = auth_key.or(api_key) {
        let plans = state.plan_manager.load_all().await.ok()?;
        for plan in plans {
            if plan.api_key == key {
                return Some(plan);
            }
        }
    }

    // 3. 使用默认套餐
    state.plan_manager.get_default().await
}

/// 构建转发目标 URL
fn build_target_url(
    provider: &crate::model::ProviderTemplate,
    user_plan: &UserPlan,
    endpoint: &str,
) -> String {
    // 优先使用固定的 base_url
    if let Some(ref base_url) = provider.base_url {
        return format!("{}{}", base_url.trim_end_matches('/'), endpoint);
    }

    // 使用 base_url_template
    if let Some(ref template) = provider.base_url_template {
        let url = template
            .replace("{plan_id}", &user_plan.plan_id)
            .replace("{provider_id}", &user_plan.provider_id);
        return format!("{}{}", url.trim_end_matches('/'), endpoint);
    }

    // 默认使用 provider 的 homepage
    format!("{}{}", provider.homepage.trim_end_matches('/'), endpoint)
}

/// 创建错误响应
fn create_error_response(status: u16, message: &str) -> Response {
    let body = serde_json::json!({
        "error": {
            "type": "error",
            "message": message
        }
    });

    Response::builder()
        .status(status)
        .header("Content-Type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}