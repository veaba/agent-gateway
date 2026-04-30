//! agent-gateway API Server

use std::net::SocketAddr;

use anyhow::Result;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use agw_core::business::start_health_monitor;

mod state;
mod error;
mod types;
mod handlers;
mod middleware;

pub use state::AppState;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化 tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    tracing::info!("Starting agent-gateway API server...");

    // 初始化 AppState
    let state = AppState::init().await?;
    tracing::info!("AppState initialized");

    // 启动后台健康监控任务
    // 每 5 分钟检查一次健康的 plan
    // 每 1 分钟检查一次处于 Error 状态的 plan（用于快速检测恢复）
    start_health_monitor(
        state.health_checker.clone(),
        300,  // 5 分钟正常检查间隔
        60,   // 1 分钟恢复检测间隔
    ).await;

    // CORS 配置
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // 速率限制器: 每分钟 100 请求
    let rate_limiter = middleware::rate_limit::RateLimiter::new(100, 60);

    // 创建路由并应用中间件
    let app = middleware::apply(handlers::create_router(state), rate_limiter)
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([127, 0, 0, 1], 8081));
    tracing::info!("API server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
