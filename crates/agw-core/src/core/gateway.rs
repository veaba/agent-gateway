//! HTTP 网关

use std::sync::Arc;
use tokio::sync::RwLock;
use axum::{
    Router,
    routing::{get, post},
    extract::{State, Request},
    body::Body,
    response::Response,
};
use axum::http;
use tower_http::trace::TraceLayer;
use tower_http::cors::{Any, CorsLayer};
use dashmap::DashMap;
use http_body_util::StreamBody;
use tokio_stream::StreamExt;
use serde_json::Value;

use crate::business::{ProviderEngine, QuotaTracker, FallbackEngine, PlanManager, FallbackReason};
use crate::storage::{ConfigStore, SqliteStore};
use crate::security::EncryptionService;
use crate::core::Forwarder;
use crate::model::UserPlan;
use crate::model_types::ApiFormat;
use crate::paths;

/// 网关状态
pub struct GatewayState {
    pub provider_engine: Arc<ProviderEngine>,
    pub plan_manager: Arc<PlanManager>,
    pub quota_tracker: Arc<QuotaTracker>,
    pub fallback_engine: Arc<RwLock<FallbackEngine>>,
    pub config_store: Arc<ConfigStore>,
    pub sqlite_store: Arc<SqliteStore>,
    pub encryption: Arc<EncryptionService>,
    /// Agent -> Plan 绑定
    pub agent_bindings: Arc<DashMap<String, String>>,
    pub default_plan_id: Arc<RwLock<Option<String>>>,
}

// Safety: GatewayState 管理的内容都是线程安全的
unsafe impl Send for GatewayState {}
unsafe impl Sync for GatewayState {}

impl GatewayState {
    /// 创建新的网关状态
    pub async fn new() -> anyhow::Result<Self> {
        // 初始化应用目录（包含迁移）
        paths::init_app_dirs().await?;

        let config_store = Arc::new(ConfigStore::new()?);
        config_store.init_data_dir().await?;

        let fallback_config = config_store.load_fallback_config().await?;

        // 创建 PlanManager
        let plan_manager = Arc::new(PlanManager::new(config_store.clone()));

        // 使用统一路径
        let sqlite_path = paths::db_path();
        let sqlite_store = Arc::new(SqliteStore::new(sqlite_path)?);

        // 创建带 SQLite 持久化的 QuotaTracker
        let quota_tracker = Arc::new(QuotaTracker::with_sqlite(sqlite_store.clone()));

        // 加载现有 Plans 到缓存
        let plans = plan_manager.load_all().await?;

        // 从 SQLite 加载现有配额数据
        let plan_ids: Vec<String> = plans.iter().map(|p| p.id.clone()).collect();
        quota_tracker.load_from_sqlite(&plan_ids).await?;

        // 同步 Plan 配额限制到 QuotaTracker
        for plan in &plans {
            let limits = crate::business::quota::QuotaLimit {
                daily: plan.custom_quota_daily,
                monthly: plan.custom_quota_monthly,
                rpm: plan.custom_rpm_limit,
            };
            quota_tracker.set_limits(&plan.id, limits).await;
        }

        Ok(Self {
            provider_engine: Arc::new(ProviderEngine::new()),
            plan_manager: plan_manager.clone(),
            quota_tracker: quota_tracker.clone(),
            fallback_engine: Arc::new(RwLock::new(FallbackEngine::with_dependencies(
                fallback_config,
                plan_manager,
                quota_tracker,
            ))),
            config_store: config_store.clone(),
            sqlite_store,
            encryption: Arc::new(EncryptionService::from_key_file(
                paths::encryption_key_path()
            )?),
            agent_bindings: Arc::new(DashMap::new()),
            default_plan_id: Arc::new(RwLock::new(None)),
        })
    }
}

/// 创建网关应用
pub async fn create_app(state: Arc<GatewayState>) -> Router {
    Router::new()
        .route("/health", get(health_handler))
        .route("/v1/messages", post(anthropic_handler))
        .route("/v1/chat/completions", post(openai_handler))
        .with_state(state.clone())
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any))
}

/// 健康检查
async fn health_handler() -> Response {
    Response::builder()
        .body(Body::from("{\"status\":\"ok\"}"))
        .unwrap()
}

/// Anthropic Messages API 处理器
#[axum::debug_handler]
async fn anthropic_handler(
    State(state): State<Arc<GatewayState>>,
    request: Request,
) -> Response {
    let start_time = std::time::Instant::now();

    let headers = request.headers().clone();

    let body_bytes = match axum::body::to_bytes(request.into_body(), 1024 * 1024 * 10).await {
        Ok(bytes) => bytes,
        Err(e) => {
            tracing::error!("Failed to read request body: {}", e);
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

    let stream = request_json.get("stream")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let auth_header = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim_start_matches("Bearer ").to_string());

    let plan_id_from_header = headers
        .get("x-plan-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let api_key_from_header = headers
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

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

    let quota_allowed = state.quota_tracker.check_and_consume(&user_plan.id).await;
    if !quota_allowed {
        tracing::warn!("Quota exceeded for plan: {}", user_plan.id);
        return create_error_response(429, "Quota exceeded. Please upgrade your plan.");
    }

    if let Some(threshold) = user_plan.alert_threshold {
        if let Some(alert) = state.quota_tracker.check_alert(&user_plan.id, threshold).await {
            tracing::warn!(
                "Quota alert for plan {}: {} ({}%)",
                user_plan.id, alert.message, alert.usage_percent * 100.0
            );
        }
    }

    let provider = match state.provider_engine.get_provider(&user_plan.provider_id).await {
        Some(p) => p,
        None => {
            tracing::error!("Provider not found: {}", user_plan.provider_id);
            return create_error_response(500, "Provider not configured");
        }
    };

    let target_url = build_target_url(&provider, &user_plan, "/v1/messages");
    tracing::debug!("Forwarding to: {}", target_url);

    let fallback_engine = state.fallback_engine.read().await;
    let max_attempts = fallback_engine.max_attempts();
    let fallback_enabled = fallback_engine.get_config().enabled;
    drop(fallback_engine);

    let mut current_plan = user_plan;
    let mut current_plan_id = current_plan.id.clone();
    let mut attempt = 0;
    let mut last_error: Option<(u16, String)> = None;

    loop {
        attempt += 1;
        tracing::info!(
            plan_id = %current_plan_id,
            attempt = attempt,
            "Sending Anthropic request"
        );

        if stream {
            match send_anthropic_stream_request(&state, &current_plan, &headers, &body_bytes, &target_url).await {
                Ok((status, response)) => {
                    let elapsed = start_time.elapsed().as_millis();
                    tracing::info!("Request completed in {}ms with status {}", elapsed, status);
                    return handle_streaming_response(response).await;
                }
                Err((status, error_msg)) => {
                    tracing::warn!(
                        plan_id = %current_plan_id,
                        status = status,
                        attempt = attempt,
                        "Request failed, checking fallback"
                    );
                    last_error = Some((status, error_msg));

                    if !should_try_fallback(status, &last_error, fallback_enabled, attempt, max_attempts, &current_plan_id, &state).await {
                        break;
                    }
                }
            }
        } else {
            match send_anthropic_request(&state, &current_plan, &headers, &body_bytes, &target_url).await {
                Ok(response) => {
                    let elapsed = start_time.elapsed().as_millis();
                    tracing::info!("Request completed in {}ms", elapsed);
                    return response;
                }
                Err((status, error_msg)) => {
                    tracing::warn!(
                        plan_id = %current_plan_id,
                        status = status,
                        attempt = attempt,
                        "Request failed, checking fallback"
                    );
                    last_error = Some((status, error_msg));

                    if !should_try_fallback(status, &last_error, fallback_enabled, attempt, max_attempts, &current_plan_id, &state).await {
                        break;
                    }
                }
            }
        }

        if let Some(fallback_plan_id) = find_fallback_plan(&state, &current_plan_id).await {
            if let Some(new_plan) = state.plan_manager.get(&fallback_plan_id).await {
                tracing::info!(
                    current_plan = %current_plan_id,
                    fallback_plan = %fallback_plan_id,
                    "Switching to fallback plan"
                );

                let _ = state.quota_tracker.check_and_consume(&fallback_plan_id).await;

                current_plan = new_plan;
                current_plan_id = fallback_plan_id.clone();

                if let Some(new_provider) = state.provider_engine.get_provider(&current_plan.provider_id).await {
                    let new_target_url = build_target_url(&new_provider, &current_plan, "/v1/messages");
                    tracing::debug!("Switching to fallback URL: {}", new_target_url);
                } else {
                    tracing::error!("Fallback provider not found: {}", current_plan.provider_id);
                    break;
                }
                continue;
            }
        }

        break;
    }

    if let Some((status, error_msg)) = last_error {
        if status == 429 {
            create_error_response(429, "Rate limit exceeded and no fallback available")
        } else if status >= 500 {
            create_error_response(status as u16, "Provider server error and no fallback available")
        } else {
            create_error_response(status as u16, &error_msg)
        }
    } else {
        create_error_response(502, "Failed to forward request to provider")
    }
}

/// 检查是否应该尝试 fallback
async fn should_try_fallback(
    status: u16,
    last_error: &Option<(u16, String)>,
    fallback_enabled: bool,
    attempt: u32,
    max_attempts: u32,
    current_plan_id: &str,
    state: &Arc<GatewayState>,
) -> bool {
    if !fallback_enabled {
        return false;
    }

    if attempt >= max_attempts {
        tracing::warn!(attempt = attempt, max_attempts = max_attempts, "Max fallback attempts reached");
        return false;
    }

    let fallback_engine = state.fallback_engine.read().await;
    if let Some(reason) = FallbackReason::from_status(status, last_error.as_ref().map(|e| e.1.clone())) {
        let should = fallback_engine.should_fallback(&reason);
        if should && fallback_engine.find_alternative(current_plan_id).await.is_some() {
            return true;
        }
    }
    false
}

/// 查找 fallback plan
async fn find_fallback_plan(
    state: &Arc<GatewayState>,
    current_plan_id: &str,
) -> Option<String> {
    let fallback_engine = state.fallback_engine.read().await;
    fallback_engine.find_alternative(current_plan_id).await
}

/// OpenAI Chat Completions API 处理器
#[axum::debug_handler]
async fn openai_handler(
    State(state): State<Arc<GatewayState>>,
    request: Request,
) -> Response {
    let start_time = std::time::Instant::now();

    let headers = request.headers().clone();

    let body_bytes = match axum::body::to_bytes(request.into_body(), 1024 * 1024 * 10).await {
        Ok(bytes) => bytes,
        Err(e) => {
            tracing::error!("Failed to read request body: {}", e);
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

    let stream = request_json.get("stream")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let auth_header = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim_start_matches("Bearer ").to_string());

    let plan_id_from_header = headers
        .get("x-plan-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let api_key_from_header = headers
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let user_plan = match resolve_user_plan(
        &state,
        plan_id_from_header.as_deref(),
        auth_header.as_deref(),
        api_key_from_header.as_deref(),
    ).await {
        Some(plan) => plan,
        None => {
            tracing::warn!("No matching plan found for OpenAI request");
            return create_error_response(401, "No matching plan found. Please configure a plan with API key.");
        }
    };

    tracing::info!("OpenAI request using plan: {} ({})", user_plan.name, user_plan.id);

    let quota_allowed = state.quota_tracker.check_and_consume(&user_plan.id).await;
    if !quota_allowed {
        tracing::warn!("Quota exceeded for plan: {}", user_plan.id);
        return create_error_response(429, "Quota exceeded. Please upgrade your plan.");
    }

    if let Some(threshold) = user_plan.alert_threshold {
        if let Some(alert) = state.quota_tracker.check_alert(&user_plan.id, threshold).await {
            tracing::warn!(
                "Quota alert for plan {}: {} ({}%)",
                user_plan.id, alert.message, alert.usage_percent * 100.0
            );
        }
    }

    let provider = match state.provider_engine.get_provider(&user_plan.provider_id).await {
        Some(p) => p,
        None => {
            tracing::error!("Provider not found: {}", user_plan.provider_id);
            return create_error_response(500, "Provider not configured");
        }
    };

    let needs_conversion = provider.api_format != ApiFormat::OpenAi;
    let fallback_engine = state.fallback_engine.read().await;
    let max_attempts = fallback_engine.max_attempts();
    let fallback_enabled = fallback_engine.get_config().enabled;
    drop(fallback_engine);

    let mut current_plan = user_plan;
    let mut current_plan_id = current_plan.id.clone();
    let mut attempt = 0;
    let mut last_error: Option<(u16, String)> = None;
    let mut endpoint = if needs_conversion { "/v1/messages" } else { "/v1/chat/completions" };

    loop {
        attempt += 1;
        tracing::info!(
            plan_id = %current_plan_id,
            attempt = attempt,
            needs_conversion = needs_conversion,
            "Sending OpenAI request"
        );

        if needs_conversion {
            let target_url = build_target_url(&provider, &current_plan, "/v1/messages");
            tracing::debug!("Converting and forwarding to: {}", target_url);

            let anthropic_request = match crate::core::converter::openai_request_to_anthropic(&request_json) {
                Ok(req) => req,
                Err(e) => {
                    tracing::error!("Failed to convert request: {}", e);
                    return create_error_response(500, "Failed to convert request format");
                }
            };

            let anthropic_body = serde_json::to_vec(&anthropic_request)
                .unwrap_or_else(|_| body_bytes.to_vec());

            let result = send_openai_converted_request(&state, &current_plan, &headers, &anthropic_body, &target_url).await;
            match result {
                Ok((status, response)) => {
                    let elapsed = start_time.elapsed().as_millis();
                    tracing::info!("Converted request completed in {}ms with status {}", elapsed, status);

                    if stream {
                        return handle_converted_streaming_response(response, "anthropic_to_openai").await;
                    } else {
                        let body_bytes = response.bytes().await.unwrap_or_default();
                        let anthropic_response: Value = serde_json::from_slice(&body_bytes)
                            .unwrap_or_else(|_| serde_json::json!({}));

                        let openai_response = crate::core::converter::anthropic_response_to_openai(&anthropic_response)
                            .unwrap_or_else(|_| serde_json::json!({}));

                        return Response::builder()
                            .status(status)
                            .header("Content-Type", "application/json")
                            .body(Body::from(openai_response.to_string()))
                            .unwrap_or_else(|_| create_error_response(500, "Failed to build response"));
                    }
                }
                Err((status, error_msg)) => {
                    tracing::warn!(
                        plan_id = %current_plan_id,
                        status = status,
                        attempt = attempt,
                        "Request failed, checking fallback"
                    );
                    last_error = Some((status, error_msg));

                    if !should_try_fallback(status, &last_error, fallback_enabled, attempt, max_attempts, &current_plan_id, &state).await {
                        break;
                    }
                }
            }
        } else {
            let target_url = build_target_url(&provider, &current_plan, "/v1/chat/completions");
            tracing::debug!("Forwarding directly to: {}", target_url);

            let result = send_openai_request(&state, &current_plan, &headers, &body_bytes, &target_url).await;
            match result {
                Ok((status, response)) => {
                    let elapsed = start_time.elapsed().as_millis();
                    tracing::info!("OpenAI request completed in {}ms with status {}", elapsed, status);

                    if stream {
                        return handle_streaming_response(response).await;
                    } else {
                        let resp_headers = response.headers().clone();
                        let body = response.bytes().await.unwrap_or_default();
                        let mut builder = Response::builder()
                            .status(status)
                            .header("Content-Type", "application/json");

                        for (name, value) in resp_headers.iter() {
                            let name_str = name.as_str();
                            if !["content-encoding", "transfer-encoding", "connection"].contains(&name_str) {
                                if let Ok(v) = value.to_str() {
                                    builder = builder.header(name_str, v);
                                }
                            }
                        }

                        return builder.body(Body::from(body))
                            .unwrap_or_else(|_| create_error_response(500, "Failed to build response"));
                    }
                }
                Err((status, error_msg)) => {
                    tracing::warn!(
                        plan_id = %current_plan_id,
                        status = status,
                        attempt = attempt,
                        "Request failed, checking fallback"
                    );
                    last_error = Some((status, error_msg));

                    if !should_try_fallback(status, &last_error, fallback_enabled, attempt, max_attempts, &current_plan_id, &state).await {
                        break;
                    }
                }
            }
        }

        if let Some(fallback_plan_id) = find_fallback_plan(&state, &current_plan_id).await {
            if let Some(new_plan) = state.plan_manager.get(&fallback_plan_id).await {
                tracing::info!(
                    current_plan = %current_plan_id,
                    fallback_plan = %fallback_plan_id,
                    "Switching to fallback plan"
                );

                let _ = state.quota_tracker.check_and_consume(&fallback_plan_id).await;

                current_plan = new_plan;
                current_plan_id = fallback_plan_id.clone();

                if let Some(new_provider) = state.provider_engine.get_provider(&current_plan.provider_id).await {
                    let new_conversion = new_provider.api_format != ApiFormat::OpenAi;
                    if new_conversion != needs_conversion {
                        endpoint = if new_conversion { "/v1/messages" } else { "/v1/chat/completions" };
                    }
                    tracing::debug!(
                        "Switching to fallback provider: {}",
                        new_provider.name
                    );
                } else {
                    tracing::error!("Fallback provider not found: {}", current_plan.provider_id);
                    break;
                }
                continue;
            }
        }

        break;
    }

    if let Some((status, error_msg)) = last_error {
        if status == 429 {
            create_error_response(429, "Rate limit exceeded and no fallback available")
        } else if status >= 500 {
            create_error_response(status as u16, "Provider server error and no fallback available")
        } else {
            create_error_response(status as u16, &error_msg)
        }
    } else {
        create_error_response(502, "Failed to forward request to provider")
    }
}

/// 发送 OpenAI 请求（直接转发）
async fn send_openai_request(
    state: &Arc<GatewayState>,
    user_plan: &UserPlan,
    headers: &http::HeaderMap,
    body_bytes: &[u8],
    target_url: &str,
) -> Result<(u16, reqwest::Response), (u16, String)> {
    let _ = state;
    let forwarder = Forwarder::new();
    let mut req_builder = forwarder.client
        .request(hyper::Method::POST, target_url);

    req_builder = req_builder
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", user_plan.api_key));

    for (name, value) in headers.iter() {
        let name_str = name.as_str();
        if !["host", "content-length", "authorization"].contains(&name_str) {
            if let Ok(v) = value.to_str() {
                req_builder = req_builder.header(name_str, v);
            }
        }
    }

    req_builder = req_builder.body(body_bytes.to_vec());

    match req_builder.send().await {
        Ok(response) => {
            let status = response.status();
            if status.is_success() || status.is_redirection() {
                Ok((status.as_u16(), response))
            } else {
                Err((status.as_u16(), String::new()))
            }
        }
        Err(e) => {
            let error_str = e.to_string();
            let status = if error_str.contains("timeout") {
                504
            } else if error_str.contains("connection") || error_str.contains("refused") {
                502
            } else {
                502
            };
            Err((status, error_str))
        }
    }
}

/// 发送 OpenAI 请求（转换为 Anthropic 格式）
async fn send_openai_converted_request(
    state: &Arc<GatewayState>,
    user_plan: &UserPlan,
    headers: &http::HeaderMap,
    anthropic_body: &[u8],
    target_url: &str,
) -> Result<(u16, reqwest::Response), (u16, String)> {
    let _ = headers;
    let _ = state;
    let forwarder = Forwarder::new();
    let mut req_builder = forwarder.client
        .request(hyper::Method::POST, target_url);

    req_builder = req_builder
        .header("Content-Type", "application/json")
        .header("x-api-key", &user_plan.api_key)
        .header("anthropic-version", "2023-06-01");

    req_builder = req_builder.body(anthropic_body.to_vec());

    match req_builder.send().await {
        Ok(response) => {
            let status = response.status();
            if status.is_success() || status.is_redirection() {
                Ok((status.as_u16(), response))
            } else {
                Err((status.as_u16(), String::new()))
            }
        }
        Err(e) => {
            let error_str = e.to_string();
            let status = if error_str.contains("timeout") {
                504
            } else if error_str.contains("connection") || error_str.contains("refused") {
                502
            } else {
                502
            };
            Err((status, error_str))
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

/// 发送 Anthropic 请求并返回响应
async fn send_anthropic_request(
    state: &Arc<GatewayState>,
    user_plan: &UserPlan,
    headers: &http::HeaderMap,
    body_bytes: &[u8],
    target_url: &str,
) -> Result<Response, (u16, String)> {
    let forwarder = Forwarder::new();
    let mut req_builder = forwarder.client
        .request(hyper::Method::POST, target_url);

    req_builder = req_builder
        .header("Content-Type", "application/json")
        .header("x-api-key", &user_plan.api_key)
        .header("anthropic-version", "2023-06-01");

    for (name, value) in headers.iter() {
        let name_str = name.as_str();
        if !["host", "content-length", "authorization"].contains(&name_str) {
            if let Ok(v) = value.to_str() {
                req_builder = req_builder.header(name_str, v);
            }
        }
    }

    req_builder = req_builder.body(body_bytes.to_vec());

    match req_builder.send().await {
        Ok(response) => {
            let status = response.status();
            if status.is_success() || status.is_redirection() {
                let resp_headers = response.headers().clone();
                let body = response.bytes().await.unwrap_or_default();
                let mut builder = Response::builder()
                    .status(status)
                    .header("Content-Type", "application/json");

                for (name, value) in resp_headers.iter() {
                    let name_str = name.as_str();
                    if !["content-encoding", "transfer-encoding", "connection"].contains(&name_str) {
                        if let Ok(v) = value.to_str() {
                            builder = builder.header(name_str, v);
                        }
                    }
                }

                Ok(builder.body(Body::from(body)).unwrap_or_else(|_| create_error_response(500, "Failed to build response")))
            } else {
                Err((status.as_u16(), String::new()))
            }
        }
        Err(e) => {
            let error_str = e.to_string();
            let status = if error_str.contains("timeout") {
                504
            } else if error_str.contains("connection") || error_str.contains("refused") {
                502
            } else {
                502
            };
            Err((status, error_str))
        }
    }
}

/// 发送流式 Anthropic 请求并返回流
async fn send_anthropic_stream_request(
    state: &Arc<GatewayState>,
    user_plan: &UserPlan,
    headers: &http::HeaderMap,
    body_bytes: &[u8],
    target_url: &str,
) -> Result<(u16, reqwest::Response), (u16, String)> {
    let forwarder = Forwarder::new();
    let mut req_builder = forwarder.client
        .request(hyper::Method::POST, target_url);

    req_builder = req_builder
        .header("Content-Type", "application/json")
        .header("x-api-key", &user_plan.api_key)
        .header("anthropic-version", "2023-06-01");

    for (name, value) in headers.iter() {
        let name_str = name.as_str();
        if !["host", "content-length", "authorization"].contains(&name_str) {
            if let Ok(v) = value.to_str() {
                req_builder = req_builder.header(name_str, v);
            }
        }
    }

    req_builder = req_builder.body(body_bytes.to_vec());

    match req_builder.send().await {
        Ok(response) => {
            let status = response.status();
            if status.is_success() || status.is_redirection() {
                Ok((status.as_u16(), response))
            } else {
                Err((status.as_u16(), String::new()))
            }
        }
        Err(e) => {
            let error_str = e.to_string();
            let status = if error_str.contains("timeout") {
                504
            } else if error_str.contains("connection") || error_str.contains("refused") {
                502
            } else {
                502
            };
            Err((status, error_str))
        }
    }
}

/// 构建转发目标 URL
fn build_target_url(
    provider: &crate::model::ProviderTemplate,
    user_plan: &UserPlan,
    endpoint: &str,
) -> String {
    if let Some(ref base_url) = provider.base_url {
        return format!("{}{}", base_url.trim_end_matches('/'), endpoint);
    }

    if let Some(ref template) = provider.base_url_template {
        let url = template
            .replace("{plan_id}", &user_plan.plan_id)
            .replace("{provider_id}", &user_plan.provider_id);
        return format!("{}{}", url.trim_end_matches('/'), endpoint);
    }

    format!("{}{}", provider.homepage.trim_end_matches('/'), endpoint)
}

/// 处理流式响应（直接透传）
async fn handle_streaming_response(response: reqwest::Response) -> Response {
    let status = response.status().as_u16();
    let byte_stream = response.bytes_stream();

    let stream = byte_stream.map(|chunk| match chunk {
        Ok(bytes) => Ok(bytes),
        Err(e) => Err(e)
    });

    let body = StreamBody::new(stream);

    Response::builder()
        .status(status)
        .header("Content-Type", "text/event-stream")
        .header("Cache-Control", "no-cache")
        .header("Connection", "keep-alive")
        .body(Body::from_stream(body))
        .unwrap_or_else(|_| create_error_response(500, "Failed to build streaming response"))
}

/// 处理转换后的流式响应 (Anthropic SSE -> OpenAI SSE)
async fn handle_converted_streaming_response(
    response: reqwest::Response,
    conversion_type: &str,
) -> Response {
    let status = response.status().as_u16();
    let byte_stream = response.bytes_stream();

    // 使用 Forwarder 的 SSE 转换流
    let stream = Forwarder::convert_sse_stream(byte_stream, conversion_type);

    let body = StreamBody::new(stream);

    Response::builder()
        .status(status)
        .header("Content-Type", "text/event-stream")
        .header("Cache-Control", "no-cache")
        .header("Connection", "keep-alive")
        .body(Body::from_stream(body))
        .unwrap_or_else(|_| create_error_response(500, "Failed to build streaming response"))
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

/// 启动网关
pub async fn serve(listen: &str) -> anyhow::Result<()> {
    let state = Arc::new(GatewayState::new().await?);
    let app = create_app(state).await;

    let addr: std::net::SocketAddr = listen.parse()
        .map_err(|_| anyhow::anyhow!("Invalid address: {}", listen))?;

    tracing::info!("Starting gateway on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}