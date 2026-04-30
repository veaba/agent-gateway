//! provider 命令

use anyhow::Result;
use clap::{Parser, Subcommand};

/// Provider 管理命令
#[derive(Parser, Debug)]
pub struct ProviderCommand {
    #[command(subcommand)]
    pub command: ProviderSubcommand,
}

#[derive(Subcommand)]
pub enum ProviderSubcommand {
    /// 列出所有 Provider
    List,
    /// 查看 Provider 详情
    Info {
        /// Provider ID
        pub provider_id: String,
    },
    /// 更新 Provider 配置
    Update,
    /// 添加自定义 Provider
    Add {
        /// Provider 配置文件路径
        pub config_path: String,
    },
}

impl ProviderCommand {
    pub async fn run(&self) -> Result<()> {
        match &self.command {
            ProviderSubcommand::List => {
                tracing::info!("Listing all providers...");
            }
            ProviderSubcommand::Info { provider_id } => {
                tracing::info!("Getting info for provider: {}", provider_id);
            }
            ProviderSubcommand::Update => {
                tracing::info!("Updating providers from remote...");
            }
            ProviderSubcommand::Add { config_path } => {
                tracing::info!("Adding custom provider from: {}", config_path);
            }
        }
        Ok(())
    }
}