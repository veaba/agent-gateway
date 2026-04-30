//! quota 命令

use std::sync::Arc;

use anyhow::Result;
use clap::{Parser, Subcommand};

use agw_core::business::PlanManager;
use agw_core::storage::{ConfigStore, SqliteStore};

/// 配额管理命令
#[derive(Parser, Debug)]
pub struct QuotaCommand {
    #[command(subcommand)]
    pub command: QuotaSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum QuotaSubcommand {
    /// 显示配额使用状态
    Status {
        /// Plan ID（可选，不指定则显示所有）
        plan_id: Option<String>,
    },
    /// 设置配额
    Set {
        /// Plan ID
        plan_id: String,
        /// 日配额
        #[arg(long)]
        daily: Option<u64>,
        /// 月配额
        #[arg(long)]
        monthly: Option<u64>,
        /// RPM 限制
        #[arg(long)]
        rpm: Option<u32>,
    },
    /// 重置配额计数器
    Reset {
        /// Plan ID
        plan_id: String,
    },
}

impl QuotaCommand {
    pub async fn run(&self) -> Result<()> {
        let config_store = Arc::new(ConfigStore::new()?);
        let manager = PlanManager::new(config_store.clone());

        match &self.command {
            QuotaSubcommand::Status { plan_id } => {
                self.handle_status(&manager, plan_id.as_deref()).await?;
            }
            QuotaSubcommand::Set {
                plan_id,
                daily,
                monthly,
                rpm,
            } => {
                self.handle_set(&manager, plan_id, *daily, *monthly, *rpm).await?;
            }
            QuotaSubcommand::Reset { plan_id } => {
                self.handle_reset(&config_store, plan_id).await?;
            }
        }
        Ok(())
    }

    /// 显示配额状态
    async fn handle_status(&self, manager: &PlanManager, plan_id: Option<&str>) -> Result<()> {
        let plans = manager.load_all().await?;

        if plans.is_empty() {
            println!("No plans configured.");
            return Ok(());
        }

        let filtered: Vec<_> = match plan_id {
            Some(id) => {
                let plan = manager.get(id).await
                    .ok_or_else(|| anyhow::anyhow!("Plan '{}' not found", id))?;
                vec![plan]
            }
            None => plans,
        };

        println!("Quota Status\n");

        for plan in filtered {
            println!("📋 {} ({})", plan.name, plan.id);

            let daily = plan.custom_quota_daily.or_else(|| {
                // 尝试从 provider 的 coding plan 获取默认配额
                None
            });
            let monthly = plan.custom_quota_monthly;
            let rpm = plan.custom_rpm_limit;

            match daily {
                Some(v) => println!("  Daily Limit:   {} requests", v),
                None => println!("  Daily Limit:   (unlimited)"),
            }
            match monthly {
                Some(v) => println!("  Monthly Limit: {} requests", v),
                None => println!("  Monthly Limit: (unlimited)"),
            }
            match rpm {
                Some(v) => println!("  RPM Limit:     {} requests/min", v),
                None => println!("  RPM Limit:     (unlimited)"),
            }

            println!("  Usage:         (tracked at runtime by gateway)");
            println!();
        }

        Ok(())
    }

    /// 设置配额
    async fn handle_set(
        &self,
        manager: &PlanManager,
        plan_id: &str,
        daily: Option<u64>,
        monthly: Option<u64>,
        rpm: Option<u32>,
    ) -> Result<()> {
        let mut plan = manager.get(plan_id).await
            .ok_or_else(|| anyhow::anyhow!("Plan '{}' not found", plan_id))?;

        if daily.is_none() && monthly.is_none() && rpm.is_none() {
            anyhow::bail!("At least one of --daily, --monthly, or --rpm must be specified");
        }

        if let Some(d) = daily {
            plan.custom_quota_daily = Some(d);
            println!("✅ Daily quota set to: {} requests", d);
        }
        if let Some(m) = monthly {
            plan.custom_quota_monthly = Some(m);
            println!("✅ Monthly quota set to: {} requests", m);
        }
        if let Some(r) = rpm {
            plan.custom_rpm_limit = Some(r);
            println!("✅ RPM limit set to: {} requests/min", r);
        }

        manager.update(plan).await?;
        Ok(())
    }

    /// 重置配额计数器
    async fn handle_reset(&self, config_store: &ConfigStore, plan_id: &str) -> Result<()> {
        let sqlite_path = config_store.data_dir().join("gateway.db");
        let sqlite_store = SqliteStore::new(sqlite_path)?;

        // 清除 SQLite 中的日配额和月配额记录
        sqlite_store.reset_quota_usage(plan_id.to_string(), "daily".to_string()).await?;
        sqlite_store.reset_quota_usage(plan_id.to_string(), "monthly".to_string()).await?;

        println!("🔄 Resetting quota counters for plan: {}", plan_id);
        println!("   SQLite persisted quota records have been cleared.");
        println!("   Note: If the gateway is currently running, its in-memory counters");
        println!("   will remain until the next request or a gateway restart.");
        println!("✅ Quota reset completed.");
        Ok(())
    }
}
