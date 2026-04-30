//! plan 命令

use std::sync::Arc;

use anyhow::Result;
use clap::{Parser, Subcommand};

use agw_core::model::{UserPlan, AgentBinding};
use agw_core::model_types::{HealthStatus, AgentConfigStatus};
use agw_core::business::PlanManager;
use agw_core::storage::ConfigStore;

/// 套餐管理命令
#[derive(Parser, Debug)]
pub struct PlanCommand {
    #[command(subcommand)]
    pub command: PlanSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum PlanSubcommand {
    /// 添加套餐（向导式）
    Add {
        /// 交互式向导
        #[arg(long)]
        wizard: bool,
        /// Provider ID
        #[arg(long)]
        provider: Option<String>,
        /// Plan ID
        #[arg(long)]
        plan: Option<String>,
        /// 模型 ID
        #[arg(long)]
        model: Option<String>,
        /// Agent 工具（逗号分隔）
        #[arg(long)]
        agents: Option<String>,
        /// API Key
        #[arg(long)]
        api_key: Option<String>,
        /// 套餐名称
        #[arg(long)]
        name: Option<String>,
    },
    /// 列出所有套餐
    List {
        /// 显示详细信息
        #[arg(long)]
        verbose: bool,
    },
    /// 使用套餐（设置为默认）
    Use {
        /// Plan ID
        plan_id: String,
    },
    /// 测试套餐连接
    Test {
        /// Plan ID
        plan_id: String,
    },
    /// 删除套餐
    Delete {
        /// Plan ID
        plan_id: String,
        /// 强制删除
        #[arg(long)]
        force: bool,
    },
    /// 编辑套餐
    Edit {
        /// Plan ID
        plan_id: String,
        /// 新的 API Key
        #[arg(long)]
        api_key: Option<String>,
        /// 新的模型 ID
        #[arg(long)]
        model: Option<String>,
        /// 启用/禁用
        #[arg(long)]
        enable: Option<bool>,
        /// 新的名称
        #[arg(long)]
        name: Option<String>,
    },
}

impl PlanCommand {
    pub async fn run(&self) -> Result<()> {
        let config_store = Arc::new(ConfigStore::new()?);
        let manager = PlanManager::new(config_store);

        match &self.command {
            PlanSubcommand::Add {
                wizard,
                provider,
                plan,
                model,
                agents,
                api_key,
                name,
            } => {
                self.handle_add(
                    &manager,
                    *wizard,
                    provider,
                    plan,
                    model,
                    agents,
                    api_key,
                    name,
                ).await?;
            }
            PlanSubcommand::List { verbose } => {
                self.handle_list(&manager, *verbose).await?;
            }
            PlanSubcommand::Use { plan_id } => {
                self.handle_use(&manager, plan_id).await?;
            }
            PlanSubcommand::Test { plan_id } => {
                self.handle_test(&manager, plan_id).await?;
            }
            PlanSubcommand::Delete { plan_id, force } => {
                self.handle_delete(&manager, plan_id, *force).await?;
            }
            PlanSubcommand::Edit {
                plan_id,
                api_key,
                model,
                enable,
                name,
            } => {
                self.handle_edit(&manager, plan_id, api_key, model, enable, name).await?;
            }
        }
        Ok(())
    }

    /// 添加套餐
    async fn handle_add(
        &self,
        manager: &PlanManager,
        wizard: bool,
        provider: &Option<String>,
        plan: &Option<String>,
        model: &Option<String>,
        agents: &Option<String>,
        api_key: &Option<String>,
        name: &Option<String>,
    ) -> Result<()> {
        if wizard {
            println!("Starting interactive plan add wizard...");
            println!("(Interactive wizard not yet implemented, please use command line arguments)");
            return Ok(());
        }

        // 验证必要参数
        let provider_id = provider.as_ref()
            .ok_or_else(|| anyhow::anyhow!("--provider is required"))?;
        let plan_id = plan.as_ref()
            .ok_or_else(|| anyhow::anyhow!("--plan is required"))?;
        let model_id = model.as_ref()
            .ok_or_else(|| anyhow::anyhow!("--model is required"))?;
        let key = api_key.as_ref()
            .ok_or_else(|| anyhow::anyhow!("--api-key is required"))?;

        // 生成唯一 ID
        let id = format!("{}-{}-{}", provider_id, plan_id, uuid::Uuid::new_v4().simple());
        let plan_name = name.clone()
            .unwrap_or_else(|| format!("{} {}", provider_id, plan_id));

        // 解析 agents
        let bound_agents = if let Some(agents_str) = agents {
            agents_str.split(',')
                .map(|a| AgentBinding {
                    agent_id: a.trim().to_string(),
                    configured: false,
                    config_status: AgentConfigStatus::NotConfigured,
                    last_connected: None,
                    error_message: None,
                })
                .collect()
        } else {
            Vec::new()
        };

        // 创建套餐
        let user_plan = UserPlan::new(
            id.clone(),
            provider_id.clone(),
            plan_id.clone(),
            plan_name.clone(),
            key.clone(),
            model_id.clone(),
        );

        // 设置绑定 agents
        let mut user_plan = user_plan;
        user_plan.bound_agents = bound_agents;

        // 添加套餐
        manager.add(user_plan).await?;

        println!("✅ Plan added successfully!");
        println!("   ID: {}", id);
        println!("   Name: {}", plan_name);
        println!("   Provider: {}", provider_id);
        println!("   Model: {}", model_id);

        if agents.is_some() {
            println!("   Bound Agents: {}", agents.as_ref().unwrap());
        }

        Ok(())
    }

    /// 列出套餐
    async fn handle_list(&self, manager: &PlanManager, verbose: bool) -> Result<()> {
        let plans = manager.load_all().await?;

        if plans.is_empty() {
            println!("No plans configured.");
            println!("Use 'agw plan add' to add a new plan.");
            return Ok(());
        }

        // 获取默认套餐
        let default_plan = manager.get_default().await;

        println!("Plans ({})", plans.len());
        println!();

        for plan in plans {
            let is_default = default_plan.as_ref()
                .map(|d| d.id == plan.id)
                .unwrap_or(false);

            let default_marker = if is_default { " [DEFAULT]" } else { "" };
            let status_icon = match plan.health_status {
                HealthStatus::Healthy => "✓",
                HealthStatus::Warning => "⚠",
                HealthStatus::Error => "✗",
                HealthStatus::Disabled => "○",
                HealthStatus::Unknown => "?",
            };

            if verbose {
                println!("{} {}{}", status_icon, plan.name, default_marker);
                println!("  ID: {}", plan.id);
                println!("  Provider: {}", plan.provider_id);
                println!("  Plan: {}", plan.plan_id);
                println!("  Model: {}", plan.selected_model_id);
                println!("  Enabled: {}", plan.enabled);
                println!("  Priority: {}", plan.priority);
                println!("  Bound Agents: {}", plan.bound_agents.len());
                if !plan.bound_agents.is_empty() {
                    for agent in &plan.bound_agents {
                        let config_status = match agent.config_status {
                            AgentConfigStatus::NotConfigured => "not configured",
                            AgentConfigStatus::AutoConfigured => "auto configured",
                            AgentConfigStatus::ManuallyConfigured => "manual",
                            AgentConfigStatus::ConfigError => "error",
                            AgentConfigStatus::NeedsUpdate => "needs update",
                        };
                        println!("    - {} ({})", agent.agent_id, config_status);
                    }
                }
                println!("  Created: {}", plan.created_at.format("%Y-%m-%d %H:%M"));
                println!("  Health: {}", plan.health_status);
                println!();
            } else {
                println!("{} {}{} - {} ({})",
                    status_icon,
                    plan.name,
                    default_marker,
                    plan.selected_model_id,
                    plan.health_status
                );
            }
        }

        Ok(())
    }

    /// 设置默认套餐
    async fn handle_use(&self, manager: &PlanManager, plan_id: &str) -> Result<()> {
        // 验证套餐存在
        if manager.get(plan_id).await.is_none() {
            anyhow::bail!("Plan '{}' not found", plan_id);
        }

        manager.set_default(plan_id).await?;
        println!("✅ Default plan set to: {}", plan_id);
        Ok(())
    }

    /// 测试套餐连接
    async fn handle_test(&self, manager: &PlanManager, plan_id: &str) -> Result<()> {
        println!("Testing plan: {}...", plan_id);

        let result = manager.test_connection(plan_id).await?;

        if result {
            println!("✅ Connection successful!");
        } else {
            println!("❌ Connection failed!");
        }

        Ok(())
    }

    /// 删除套餐
    async fn handle_delete(&self, manager: &PlanManager, plan_id: &str, force: bool) -> Result<()> {
        // 检查套餐存在
        let plan = manager.get(plan_id).await
            .ok_or_else(|| anyhow::anyhow!("Plan '{}' not found", plan_id))?;

        // 检查是否是默认套餐
        let default_plan = manager.get_default().await;
        if let Some(ref default) = default_plan {
            if default.id == plan_id && !force {
                anyhow::bail!("Plan '{}' is the default plan. Use --force to delete.", plan_id);
            }
        }

        manager.delete(plan_id).await?;
        println!("✅ Plan '{}' deleted", plan.name);
        Ok(())
    }

    /// 编辑套餐
    async fn handle_edit(
        &self,
        manager: &PlanManager,
        plan_id: &str,
        api_key: &Option<String>,
        model: &Option<String>,
        enable: &Option<bool>,
        name: &Option<String>,
    ) -> Result<()> {
        let mut plan = manager.get(plan_id).await
            .ok_or_else(|| anyhow::anyhow!("Plan '{}' not found", plan_id))?;

        if let Some(key) = api_key {
            plan.api_key = key.clone();
            println!("Updated API Key");
        }

        if let Some(m) = model {
            plan.selected_model_id = m.clone();
            println!("Updated Model: {}", m);
        }

        if let Some(e) = enable {
            plan.enabled = *e;
            println!("Updated Enabled: {}", e);
        }

        if let Some(n) = name {
            plan.name = n.clone();
            println!("Updated Name: {}", n);
        }

        manager.update(plan).await?;
        println!("✅ Plan '{}' updated", plan_id);
        Ok(())
    }
}