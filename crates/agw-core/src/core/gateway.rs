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
use tower_http::trace::TraceLayer;
use tower_http::cors::{Any, CorsLayer};
use dashmap::DashMap;

use crate::business::{ProviderEngine, QuotaTracker, FallbackEngine};
use crate::storage::{ConfigStore, SqliteStore};
use crate::security::EncryptionService;

/// 网关状态
pub struct GatewayState {
    pub provider_engine: Arc<ProviderEngine>,
    pub quota_tracker: Arc<QuotaTracker>,
    pub fallback_engine: Arc<RwLock<FallbackEngine>>,
    pub config_store: Arc<ConfigStore>,
    pub sqlite_store: Option<Arc<SqliteStore>>,
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
        let config_store = Arc::new(ConfigStore::new()?);
        config_store.init_data_dir().await?;

        let fallback_config = config_store.load_fallback_config().await?;

        Ok(Self {
            provider_engine: Arc::new(ProviderEngine::new()),
            quota_tracker: Arc::new(QuotaTracker::new()),
            fallback_engine: Arc::new(RwLock::new(FallbackEngine::with_config(fallback_config))),
            config_store: config_store.clone(),
            sqlite_store: None,
            encryption: Arc::new(EncryptionService::from_key_file(
                config_store.config_dir().join("encryption.key")
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
async fn anthropic_handler(
    State(_state): State<Arc<GatewayState>>,
    _request: Request<Body>,
) -> Response {
    tracing::info!("Received Anthropic API request");

    // TODO: 实现完整的请求处理逻辑
    // 1. 解析请求
    // 2. 识别 Agent
    // 3. 加载 UserPlan
    // 4. 检查配额
    // 5. 转发请求
    // 6. 记录日志

    Response::builder()
        .status(200)
        .body(Body::from("{}"))
        .unwrap()
}

/// OpenAI Chat Completions API 处理器
async fn openai_handler(
    State(_state): State<Arc<GatewayState>>,
    _request: Request<Body>,
) -> Response {
    tracing::info!("Received OpenAI API request");

    // TODO: 实现完整的请求处理逻辑
    // 1. 解析请求
    // 2. 识别 Agent
    // 3. 加载 UserPlan
    // 4. 检查配额
    // 5. 协议转换
    // 6. 转发请求
    // 7. 记录日志

    Response::builder()
        .status(200)
        .body(Body::from("{}"))
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