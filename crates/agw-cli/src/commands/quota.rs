//! quota 命令

use anyhow::Result;
use clap::{Parser, Subcommand};

/// 配额管理命令
#[derive(Parser, Debug)]
pub struct QuotaCommand {
    #[command(subcommand)]
    pub command: QuotaSubcommand,
}

#[derive(Subcommand)]
pub enum QuotaSubcommand {
    /// 显示配额使用状态
    Status {
        /// Plan ID（可选，不指定则显示所有）
        pub plan_id: Option<String>,
    },
    /// 设置配额
    Set {
        /// Plan ID
        pub plan_id: String,
        /// 日配额
        pub daily: Option<u64>,
        /// 月配额
        pub monthly: Option<u64>,
        /// RPM 限制
        pub rpm: Option<u32>,
    },
    /// 重置配额计数器
    Reset {
        /// Plan ID
        pub plan_id: String,
    },
}

impl QuotaCommand {
    pub async fn run(&self) -> Result<()> {
        match &self.command {
            QuotaSubcommand::Status { plan_id } => {
                if let Some(id) = plan_id {
                    tracing::info!("Showing quota status for plan: {}", id);
                } else {
                    tracing::info!("Showing quota status for all plans");
                }
            }
            QuotaSubcommand::Set { plan_id, daily, monthly, rpm } => {
                tracing::info!(
                    "Setting quota for plan {}: daily={:?}, monthly={:?}, rpm={:?}",
                    plan_id, daily, monthly, rpm
                );
            }
            QuotaSubcommand::Reset { plan_id } => {
                tracing::info!("Resetting quota counters for plan: {}", plan_id);
            }
        }
        Ok(())
    }
}