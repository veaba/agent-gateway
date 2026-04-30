//! plan 命令

use anyhow::Result;
use clap::{Parser, Subcommand};

/// 套餐管理命令
#[derive(Parser, Debug)]
pub struct PlanCommand {
    #[command(subcommand)]
    pub command: PlanSubcommand,
}

#[derive(Subcommand)]
pub enum PlanSubcommand {
    /// 添加套餐（向导式）
    Add {
        /// 交互式向导
        #[arg(long)]
        pub wizard: bool,
        /// Provider ID
        #[arg(long)]
        pub provider: Option<String>,
        /// Plan ID
        #[arg(long)]
        pub plan: Option<String>,
        /// 模型 ID
        #[arg(long)]
        pub model: Option<String>,
        /// Agent 工具（逗号分隔）
        #[arg(long)]
        pub agents: Option<String>,
        /// API Key
        #[arg(long)]
        pub api_key: Option<String>,
    },
    /// 列出所有套餐
    List,
    /// 使用套餐
    Use {
        /// Plan ID
        pub plan_id: String,
    },
    /// 测试套餐连接
    Test {
        /// Plan ID
        pub plan_id: String,
    },
    /// 删除套餐
    Delete {
        /// Plan ID
        pub plan_id: String,
    },
    /// 编辑套餐
    Edit {
        /// Plan ID
        pub plan_id: String,
    },
}

impl PlanCommand {
    pub async fn run(&self) -> Result<()> {
        match &self.command {
            PlanSubcommand::Add { .. } => {
                // TODO: 实现添加套餐逻辑
                tracing::info!("Adding new plan...");
            }
            PlanSubcommand::List => {
                // TODO: 实现列出套餐逻辑
                tracing::info!("Listing all plans...");
            }
            PlanSubcommand::Use { plan_id } => {
                tracing::info!("Setting default plan to: {}", plan_id);
            }
            PlanSubcommand::Test { plan_id } => {
                tracing::info!("Testing plan: {}", plan_id);
            }
            PlanSubcommand::Delete { plan_id } => {
                tracing::info!("Deleting plan: {}", plan_id);
            }
            PlanSubcommand::Edit { plan_id } => {
                tracing::info!("Editing plan: {}", plan_id);
            }
        }
        Ok(())
    }
}