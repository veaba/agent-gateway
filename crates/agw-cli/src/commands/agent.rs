//! agent 命令

use anyhow::Result;
use clap::{Parser, Subcommand};

/// Agent 工具管理命令
#[derive(Parser, Debug)]
pub struct AgentCommand {
    #[command(subcommand)]
    pub command: AgentSubcommand,
}

#[derive(Subcommand)]
pub enum AgentSubcommand {
    /// 列出所有支持的 Agent 工具
    List,
    /// 绑定 Agent 工具到套餐
    Bind {
        /// Plan ID
        pub plan_id: String,
        /// Agent ID
        pub agent_id: String,
    },
    /// 解绑 Agent 工具
    Unbind {
        /// Plan ID
        pub plan_id: String,
        /// Agent ID
        pub agent_id: String,
    },
    /// 自动配置 Agent 工具
    AutoConfig {
        /// Plan ID
        pub plan_id: String,
        /// Agent ID
        pub agent_id: String,
    },
    /// 查看 Agent 配置方法
    Config {
        /// Agent ID
        pub agent_id: String,
    },
}

impl AgentCommand {
    pub async fn run(&self) -> Result<()> {
        match &self.command {
            AgentSubcommand::List => {
                tracing::info!("Listing all supported agents...");
            }
            AgentSubcommand::Bind { plan_id, agent_id } => {
                tracing::info!("Binding agent {} to plan {}", agent_id, plan_id);
            }
            AgentSubcommand::Unbind { plan_id, agent_id } => {
                tracing::info!("Unbinding agent {} from plan {}", agent_id, plan_id);
            }
            AgentSubcommand::AutoConfig { plan_id, agent_id } => {
                tracing::info!("Auto-configuring agent {} for plan {}", agent_id, plan_id);
            }
            AgentSubcommand::Config { agent_id } => {
                tracing::info!("Showing config for agent: {}", agent_id);
            }
        }
        Ok(())
    }
}