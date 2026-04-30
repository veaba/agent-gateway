//! fallback 命令

use anyhow::Result;
use clap::{Parser, Subcommand};

/// Fallback 控制命令
#[derive(Parser, Debug)]
pub struct FallbackCommand {
    #[command(subcommand)]
    pub command: FallbackSubcommand,
}

#[derive(Subcommand)]
pub enum FallbackSubcommand {
    /// 启用自动 Fallback
    On,
    /// 禁用自动 Fallback
    Off,
    /// 设置 Fallback 优先级顺序
    Set {
        /// Plan ID 列表（逗号分隔）
        pub plans: String,
    },
    /// 显示当前 Fallback 配置
    Status,
}

impl FallbackCommand {
    pub async fn run(&self) -> Result<()> {
        match &self.command {
            FallbackSubcommand::On => {
                tracing::info!("Enabling automatic fallback");
            }
            FallbackSubcommand::Off => {
                tracing::info!("Disabling automatic fallback");
            }
            FallbackSubcommand::Set { plans } => {
                tracing::info!("Setting fallback priority: {}", plans);
            }
            FallbackSubcommand::Status => {
                tracing::info!("Showing fallback status");
            }
        }
        Ok(())
    }
}