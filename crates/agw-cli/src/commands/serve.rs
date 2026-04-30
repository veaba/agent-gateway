//! serve 命令

use anyhow::Result;
use clap::Parser;

use super::super::Cli;

/// 启动网关服务
#[derive(Parser, Debug)]
pub struct ServeCommand {
    /// 后台运行
    #[arg(long)]
    pub daemon: bool,
    /// 监听地址
    #[arg(long, default_value = "127.0.0.1:8080")]
    pub listen: String,
}

impl ServeCommand {
    pub async fn run(&self) -> Result<()> {
        tracing::info!("Starting agent-gateway on {}", self.listen);

        if self.daemon {
            tracing::info!("Running in daemon mode");
        }

        // TODO: 实现网关启动逻辑
        tracing::info!("Gateway service started");
        Ok(())
    }
}