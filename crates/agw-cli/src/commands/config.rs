//! config 命令

use anyhow::Result;
use clap::Parser;

use std::process::Command;

use agw_core::storage::ConfigStore;

/// 配置管理命令
#[derive(Parser, Debug)]
pub struct ConfigCommand {
    /// 打开编辑器编辑配置
    #[arg(long)]
    pub edit: bool,
    /// 显示配置内容
    #[arg(long)]
    pub show: bool,
}

impl ConfigCommand {
    pub async fn run(&self) -> Result<()> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Cannot find config directory"))?
            .join("agent-gateway");

        if self.edit {
            tracing::info!("Opening config dir: {:?}", config_dir);

            #[cfg(target_os = "windows")]
            Command::new("explorer").arg(&config_dir).spawn()?;
            #[cfg(target_os = "macos")]
            Command::new("open").arg(&config_dir).spawn()?;
            #[cfg(target_os = "linux")]
            Command::new("xdg-open").arg(&config_dir).spawn()?;
        } else if self.show {
            self.handle_show(&config_dir).await?;
        } else {
            // 默认行为：显示帮助
            println!("Configuration directory: {}", config_dir.display());
            println!();
            println!("Use --edit to open config editor, or --show to display config");
        }
        Ok(())
    }

    /// 显示配置内容
    async fn handle_show(&self, config_dir: &std::path::Path) -> Result<()> {
        let config_store = ConfigStore::with_path(config_dir.to_path_buf())?;

        // 显示用户套餐配置
        let user_plans_path = config_dir.join("user_plans.yaml");
        println!("📋 User Plans Config");
        println!("   Path: {}", user_plans_path.display());
        if user_plans_path.exists() {
            match tokio::fs::read_to_string(&user_plans_path).await {
                Ok(content) => {
                    let plans = config_store.load_user_plans().await?;
                    println!("   Plans: {}", plans.user_plans.len());
                    if let Some(ref default_id) = plans.default_user_plan_id {
                        println!("   Default Plan: {}", default_id);
                    }
                    println!();
                    println!("{}", content);
                }
                Err(e) => println!("   Error reading file: {}", e),
            }
        } else {
            println!("   (not created yet)");
        }

        println!();

        // 显示 Fallback 配置
        let fallback_path = config_dir.join("fallback.yaml");
        println!("🔄 Fallback Config");
        println!("   Path: {}", fallback_path.display());
        if fallback_path.exists() {
            match tokio::fs::read_to_string(&fallback_path).await {
                Ok(content) => {
                    let fallback = config_store.load_fallback_config().await?;
                    println!("   Enabled: {}", fallback.enabled);
                    println!("   Max Attempts: {}", fallback.max_attempts);
                    println!("   Priority Order: {:?}", fallback.priority_order);
                    println!();
                    println!("{}", content);
                }
                Err(e) => println!("   Error reading file: {}", e),
            }
        } else {
            println!("   (not created yet)");
        }

        println!();

        // 显示内置 Provider 配置
        let providers_path = config_dir.join("providers_builtin.yaml");
        println!("📦 Providers Config");
        println!("   Path: {}", providers_path.display());
        if providers_path.exists() {
            match tokio::fs::read_to_string(&providers_path).await {
                Ok(content) => {
                    let providers = config_store.load_providers().await?;
                    println!("   Providers: {}", providers.providers.len());
                    println!();
                    println!("{}", content);
                }
                Err(e) => println!("   Error reading file: {}", e),
            }
        } else {
            println!("   (not created yet)");
        }

        Ok(())
    }
}
