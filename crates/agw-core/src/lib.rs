//! agent-gateway 核心库
//!
//! 统一网关：管理多个AI编码工具（Claude Code, Kimi Code, OpenCode, Kilo CLI）
//! 支持 Provider-Plan-Model-Agent 四层体系、自动降级、配额控制、协议转换

pub mod model;
pub mod model_types;

pub mod business;
pub mod core;
pub mod storage;
pub mod security;
pub mod plugin;

pub use model::*;
pub use model_types::*;

/// 全局初始化
pub async fn init() -> anyhow::Result<()> {
    // 初始化 tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    tracing::info!("agent-gateway core initialized");
    Ok(())
}