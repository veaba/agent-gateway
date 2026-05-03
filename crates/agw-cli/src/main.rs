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
    plugin::PluginCommand,
};
use agw_core::paths;

/// CLI 应用程序
#[derive(Parser)]
#[command(
    name = "agw",
    about = "agent-gateway: 多AI编码工具统一网关",
    long_about = None,
    version
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
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
    /// 插件管理
    Plugin(PluginCommand),
    /// API Key 助手
    Key {
        #[command(subcommand)]
        command: KeyCommands,
    },
}

#[derive(Subcommand)]
pub enum KeyCommands {
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
    // 初始化 tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Serve(cmd) => cmd.run().await,
        Commands::Stop => {
            handle_stop().await
        }
        Commands::Plan(cmd) => cmd.run().await,
        Commands::Provider(cmd) => cmd.run().await,
        Commands::Agent(cmd) => cmd.run().await,
        Commands::Fallback(cmd) => cmd.run().await,
        Commands::Quota(cmd) => cmd.run().await,
        Commands::Config(cmd) => cmd.run().await,
        Commands::Log(cmd) => cmd.run().await,
        Commands::Completion(cmd) => cmd.run().await,
        Commands::Plugin(cmd) => cmd.run().await,
        Commands::Key { command } => {
            match command {
                KeyCommands::OpenPage { provider } => {
                    handle_key_open_page(&provider).await
                }
                KeyCommands::Test { plan } => {
                    handle_key_test(&plan).await
                }
            }
        }
    }
}

/// 停止网关服务
async fn handle_stop() -> Result<()> {
    let pid_file = paths::pid_path();

    if pid_file.exists() {
        let pid_str = tokio::fs::read_to_string(&pid_file).await?;
        let pid: u32 = pid_str.trim().parse()
            .map_err(|e| anyhow::anyhow!("Invalid PID in file: {}", e))?;

        println!("Stopping gateway service (PID: {})...", pid);

        #[cfg(target_os = "windows")]
        {
            std::process::Command::new("taskkill")
                .args(["/PID", &pid.to_string(), "/F"])
                .output()?;
        }
        #[cfg(not(target_os = "windows"))]
        {
            std::process::Command::new("kill")
                .arg(pid.to_string())
                .output()?;
        }

        // 删除 PID 文件
        tokio::fs::remove_file(&pid_file).await.ok();

        println!("✅ Gateway service stopped.");
    } else {
        println!("⚠️  PID file not found. Gateway may not be running.");
        println!("   If the gateway is still running, stop it manually.");
    }

    Ok(())
}

/// 打开 Provider 的 API Key 页面
async fn handle_key_open_page(provider_id: &str) -> Result<()> {
    use agw_core::business::ProviderEngine;

    let engine = ProviderEngine::new();
    let provider = engine.get_provider(provider_id).await
        .ok_or_else(|| anyhow::anyhow!("Provider '{}' not found", provider_id))?;

    let url = provider.get_api_key_url
        .or(provider.onboarding.get_key_url)
        .ok_or_else(|| anyhow::anyhow!("Provider '{}' does not have an API key URL", provider_id))?;

    println!("Opening API key page for {}...", provider.name);

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", &url])
            .spawn()?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&url)
            .spawn()?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&url)
            .spawn()?;
    }

    println!("✅ Opened: {}", url);
    Ok(())
}

/// 测试 Plan 的 API Key
async fn handle_key_test(plan_id: &str) -> Result<()> {
    use std::sync::Arc;
    use agw_core::business::PlanManager;
    use agw_core::storage::ConfigStore;

    let config_store = Arc::new(ConfigStore::new()?);
    let manager = PlanManager::new(config_store);

    println!("Testing API key for plan: {}...", plan_id);

    let result = manager.test_connection(plan_id).await?;

    if result {
        println!("✅ API key is valid!");
    } else {
        println!("❌ API key test failed. Please check your key and plan configuration.");
    }

    Ok(())
}
