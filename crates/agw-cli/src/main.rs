//! agent-gateway CLI 工具
//!
//! 命令行界面用于管理多AI编码工具网关

mod commands;

use anyhow::Result;
use clap::{Parser, Subcommand};

use commands::{
    serve::ServeCommand,
    plan::PlanCommand,
    provider::ProviderCommand,
    agent::AgentCommand,
    fallback::FallbackCommand,
    quota::QuotaCommand,
    config::ConfigCommand,
    log::LogCommand,
    completion::CompletionCommand,
};

/// CLI 应用程序
#[derive(Parser)]
#[command(
    name = "agw",
    about = "agent-gateway: 多AI编码工具统一网关",
    long_about = None,
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 启动网关服务
    Serve(ServeCommand),
    /// 停止网关服务
    Stop,
    /// 套餐管理
    Plan(PlanCommand),
    /// Provider 管理
    Provider(ProviderCommand),
    /// Agent 工具管理
    Agent(AgentCommand),
    /// Fallback 控制
    Fallback(FallbackCommand),
    /// 配额管理
    Quota(QuotaCommand),
    /// 配置管理
    Config(ConfigCommand),
    /// 日志管理
    Log(LogCommand),
    /// Shell 补全
    Completion(CompletionCommand),
    /// API Key 助手
    Key {
        #[command(subcommand)]
        command: KeyCommands,
    },
}

#[derive(Subcommand)]
enum KeyCommands {
    /// 打开 Provider 的 API Key 获取页面
    OpenPage {
        /// Provider ID
        provider: String,
    },
    /// 测试 API Key 是否有效
    Test {
        /// Plan ID
        plan: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Serve(cmd) => cmd.run().await,
        Commands::Stop => {
            // TODO: 实现停止服务
            tracing::info!("Stopping gateway service...");
            Ok(())
        }
        Commands::Plan(cmd) => cmd.run().await,
        Commands::Provider(cmd) => cmd.run().await,
        Commands::Agent(cmd) => cmd.run().await,
        Commands::Fallback(cmd) => cmd.run().await,
        Commands::Quota(cmd) => cmd.run().await,
        Commands::Config(cmd) => cmd.run().await,
        Commands::Log(cmd) => cmd.run().await,
        Commands::Completion(cmd) => cmd.run().await,
        Commands::Key { command } => {
            match command {
                KeyCommands::OpenPage { provider } => {
                    // TODO: 实现打开 API Key 页面
                    tracing::info!("Opening API key page for provider: {}", provider);
                    Ok(())
                }
                KeyCommands::Test { plan } => {
                    // TODO: 实现测试 API Key
                    tracing::info!("Testing API key for plan: {}", plan);
                    Ok(())
                }
            }
        }
    }
}