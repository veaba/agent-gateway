//! fallback 命令

use std::sync::Arc;

use anyhow::Result;
use clap::{Parser, Subcommand};

use agw_core::model::FallbackConfig;
use agw_core::storage::ConfigStore;

/// Fallback 控制命令
#[derive(Parser, Debug)]
pub struct FallbackCommand {
    #[command(subcommand)]
    pub command: FallbackSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum FallbackSubcommand {
    /// 启用自动 Fallback
    On,
    /// 禁用自动 Fallback
    Off,
    /// 设置 Fallback 优先级顺序
    Set {
        /// Plan ID 列表（逗号分隔）
        plans: String,
        /// 最大重试次数
        #[arg(long, default_value = "3")]
        max_attempts: u32,
    },
    /// 显示当前 Fallback 配置
    Status,
    /// 添加 Plan 到优先级列表末尾
    Add {
        /// Plan ID
        plan_id: String,
    },
    /// 从优先级列表移除 Plan
    Remove {
        /// Plan ID
        plan_id: String,
    },
}

impl FallbackCommand {
    pub async fn run(&self) -> Result<()> {
        let config_store = Arc::new(ConfigStore::new()?);

        match &self.command {
            FallbackSubcommand::On => {
                self.handle_on(&config_store).await?;
            }
            FallbackSubcommand::Off => {
                self.handle_off(&config_store).await?;
            }
            FallbackSubcommand::Set { plans, max_attempts } => {
                self.handle_set(&config_store, plans, *max_attempts).await?;
            }
            FallbackSubcommand::Status => {
                self.handle_status(&config_store).await?;
            }
            FallbackSubcommand::Add { plan_id } => {
                self.handle_add(&config_store, plan_id).await?;
            }
            FallbackSubcommand::Remove { plan_id } => {
                self.handle_remove(&config_store, plan_id).await?;
            }
        }
        Ok(())
    }

    /// 启用 Fallback
    async fn handle_on(&self, config_store: &Arc<ConfigStore>) -> Result<()> {
        let mut config = config_store.load_fallback_config().await?;
        config.enabled = true;
        config_store.save_fallback_config(&config).await?;

        println!("✅ Automatic fallback enabled");
        self.show_status(&config);
        Ok(())
    }

    /// 禁用 Fallback
    async fn handle_off(&self, config_store: &Arc<ConfigStore>) -> Result<()> {
        let mut config = config_store.load_fallback_config().await?;
        config.enabled = false;
        config_store.save_fallback_config(&config).await?;

        println!("❌ Automatic fallback disabled");
        self.show_status(&config);
        Ok(())
    }

    /// 设置优先级顺序
    async fn handle_set(
        &self,
        config_store: &Arc<ConfigStore>,
        plans: &str,
        max_attempts: u32,
    ) -> Result<()> {
        let plan_ids: Vec<String> = plans.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if plan_ids.is_empty() {
            anyhow::bail!("No plan IDs provided. Use comma-separated list.");
        }

        // 验证 plan 存在性
        let user_plans = config_store.load_user_plans().await?;
        for plan_id in &plan_ids {
            if !user_plans.user_plans.iter().any(|p| &p.id == plan_id) {
                anyhow::bail!("Plan '{}' not found", plan_id);
            }
        }

        let mut config = config_store.load_fallback_config().await?;
        config.priority_order = plan_ids.clone();
        config.max_attempts = max_attempts;
        config_store.save_fallback_config(&config).await?;

        println!("✅ Fallback priority set: {}", plans);
        println!("   Max attempts: {}", max_attempts);
        self.show_status(&config);
        Ok(())
    }

    /// 显示状态
    async fn handle_status(&self, config_store: &Arc<ConfigStore>) -> Result<()> {
        let config = config_store.load_fallback_config().await?;
        self.show_status(&config);
        Ok(())
    }

    /// 添加 Plan 到优先级列表
    async fn handle_add(&self, config_store: &Arc<ConfigStore>, plan_id: &str) -> Result<()> {
        // 验证 plan 存在性
        let user_plans = config_store.load_user_plans().await?;
        if !user_plans.user_plans.iter().any(|p| &p.id == plan_id) {
            anyhow::bail!("Plan '{}' not found", plan_id);
        }

        let mut config = config_store.load_fallback_config().await?;

        // 检查是否已存在
        if config.priority_order.iter().any(|p| p == plan_id) {
            println!("⚠️  Plan '{}' is already in fallback list", plan_id);
            return Ok(());
        }

        config.priority_order.push(plan_id.to_string());
        config_store.save_fallback_config(&config).await?;

        println!("✅ Plan '{}' added to fallback list", plan_id);
        self.show_status(&config);
        Ok(())
    }

    /// 从优先级列表移除 Plan
    async fn handle_remove(&self, config_store: &Arc<ConfigStore>, plan_id: &str) -> Result<()> {
        let mut config = config_store.load_fallback_config().await?;

        let pos = config.priority_order.iter().position(|p| p == plan_id);
        match pos {
            Some(idx) => {
                config.priority_order.remove(idx);
                config_store.save_fallback_config(&config).await?;
                println!("✅ Plan '{}' removed from fallback list", plan_id);
                self.show_status(&config);
            }
            None => {
                println!("⚠️  Plan '{}' is not in fallback list", plan_id);
            }
        }

        Ok(())
    }

    /// 显示 Fallback 配置状态
    fn show_status(&self, config: &FallbackConfig) {
        println!();
        println!("Fallback Configuration:");
        println!("  Enabled: {}", if config.enabled { "✓ Yes" } else { "✗ No" });
        println!("  Max Attempts: {}", config.max_attempts);

        if config.priority_order.is_empty() {
            println!("  Priority Order: (empty)");
            println!("    Use 'agw fallback set <plan1,plan2>' to set priority order.");
        } else {
            println!("  Priority Order:");
            for (i, plan_id) in config.priority_order.iter().enumerate() {
                let marker = if i == 0 { "→ PRIMARY" } else { "" };
                println!("    {}. {} {}", i + 1, plan_id, marker);
            }
        }
    }
}