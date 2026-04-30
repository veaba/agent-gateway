//! agent 命令

use std::sync::Arc;

use anyhow::Result;
use clap::{Parser, Subcommand};

use agw_core::business::{PlanManager, ProviderEngine};
use agw_core::model::AgentBinding;
use agw_core::model_types::AgentConfigStatus;
use agw_core::storage::ConfigStore;

/// Agent 工具管理命令
#[derive(Parser, Debug)]
pub struct AgentCommand {
    #[command(subcommand)]
    pub command: AgentSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum AgentSubcommand {
    /// 列出所有支持的 Agent 工具
    List,
    /// 绑定 Agent 工具到套餐
    Bind {
        /// Plan ID
        plan_id: String,
        /// Agent ID
        agent_id: String,
    },
    /// 解绑 Agent 工具
    Unbind {
        /// Plan ID
        plan_id: String,
        /// Agent ID
        agent_id: String,
    },
    /// 自动配置 Agent 工具
    AutoConfig {
        /// Plan ID
        plan_id: String,
        /// Agent ID
        agent_id: String,
    },
    /// 查看 Agent 配置方法
    Config {
        /// Agent ID
        agent_id: String,
    },
}

impl AgentCommand {
    pub async fn run(&self) -> Result<()> {
        match &self.command {
            AgentSubcommand::List => {
                self.handle_list().await?;
            }
            AgentSubcommand::Bind { plan_id, agent_id } => {
                self.handle_bind(plan_id, agent_id).await?;
            }
            AgentSubcommand::Unbind { plan_id, agent_id } => {
                self.handle_unbind(plan_id, agent_id).await?;
            }
            AgentSubcommand::AutoConfig { plan_id, agent_id } => {
                self.handle_autoconfig(plan_id, agent_id).await?;
            }
            AgentSubcommand::Config { agent_id } => {
                self.handle_config(agent_id).await?;
            }
        }
        Ok(())
    }

    /// 列出所有支持的 Agent 工具
    async fn handle_list(&self) -> Result<()> {
        let engine = ProviderEngine::new();
        let providers = engine.list_providers().await;

        let mut agents: Vec<_> = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for provider in providers {
            for agent in &provider.supported_agents {
                if seen.insert(agent.agent_id.clone()) {
                    agents.push((agent.agent_id.clone(), agent.name.clone(), provider.provider_id.clone()));
                }
            }
        }

        if agents.is_empty() {
            println!("No supported agents found.");
            return Ok(());
        }

        println!("Supported Agents ({})\n", agents.len());
        for (id, name, provider_id) in agents {
            println!("  {} ({}) - Provider: {}", name, id, provider_id);
        }

        Ok(())
    }

    /// 绑定 Agent 到套餐
    async fn handle_bind(&self, plan_id: &str, agent_id: &str) -> Result<()> {
        let config_store = Arc::new(ConfigStore::new()?);
        let manager = PlanManager::new(config_store);

        let mut plan = manager.get(plan_id).await
            .ok_or_else(|| anyhow::anyhow!("Plan '{}' not found", plan_id))?;

        // 检查是否已绑定
        if plan.bound_agents.iter().any(|a| a.agent_id == agent_id) {
            println!("Agent '{}' is already bound to plan '{}'", agent_id, plan_id);
            return Ok(());
        }

        plan.bound_agents.push(AgentBinding {
            agent_id: agent_id.to_string(),
            configured: false,
            config_status: AgentConfigStatus::NotConfigured,
            last_connected: None,
            error_message: None,
        });

        manager.update(plan).await?;
        println!("✅ Agent '{}' bound to plan '{}'", agent_id, plan_id);
        Ok(())
    }

    /// 解绑 Agent
    async fn handle_unbind(&self, plan_id: &str, agent_id: &str) -> Result<()> {
        let config_store = Arc::new(ConfigStore::new()?);
        let manager = PlanManager::new(config_store);

        let mut plan = manager.get(plan_id).await
            .ok_or_else(|| anyhow::anyhow!("Plan '{}' not found", plan_id))?;

        let pos = plan.bound_agents.iter().position(|a| a.agent_id == agent_id);
        match pos {
            Some(idx) => {
                plan.bound_agents.remove(idx);
                manager.update(plan).await?;
                println!("✅ Agent '{}' unbound from plan '{}'", agent_id, plan_id);
            }
            None => {
                println!("Agent '{}' is not bound to plan '{}'", agent_id, plan_id);
            }
        }

        Ok(())
    }

    /// 自动配置 Agent
    async fn handle_autoconfig(&self, plan_id: &str, agent_id: &str) -> Result<()> {
        let config_store = Arc::new(ConfigStore::new()?);
        let manager = PlanManager::new(config_store.clone());

        let plan = manager.get(plan_id).await
            .ok_or_else(|| anyhow::anyhow!("Plan '{}' not found", plan_id))?;

        // 验证 agent 已绑定
        if !plan.bound_agents.iter().any(|a| a.agent_id == agent_id) {
            anyhow::bail!("Agent '{}' is not bound to plan '{}'. Run 'agw agent bind {} {}' first.", agent_id, plan_id, plan_id, agent_id);
        }

        // 查找 provider 中的 setup guide
        let engine = ProviderEngine::new();
        let provider = engine.get_provider(&plan.provider_id).await
            .ok_or_else(|| anyhow::anyhow!("Provider '{}' not found", plan.provider_id))?;

        let guide = provider.onboarding.agent_setup_guides.iter()
            .find(|g| g.agent_id == agent_id);

        match guide {
            Some(guide) => {
                println!("Auto-configuring agent '{}' for plan '{}'", agent_id, plan_id);
                println!();

                if !guide.auto_config_supported {
                    println!("⚠️  Auto-config is not supported for this agent.");
                    println!("   Use 'agw agent config {}' to see manual setup steps.", agent_id);
                    return Ok(());
                }

                // 输出环境变量配置
                if !guide.env_vars.is_empty() {
                    println!("Environment Variables:");
                    for env in &guide.env_vars {
                        // Replace placeholders in value
                        let value = env.value
                            .replace("{{api_key}}", &plan.api_key)
                            .replace("{{base_url}}", provider.base_url.as_deref().unwrap_or(""))
                            .replace("{{model}}", &plan.selected_model_id);
                        println!("  export {}={}", env.name, value);
                    }
                    println!();
                }

                // 输出配置文件路径
                println!("Config File Paths:");
                #[cfg(target_os = "windows")]
                if let Some(ref path) = guide.config_file_paths.windows {
                    println!("  {}", path);
                }
                #[cfg(target_os = "macos")]
                if let Some(ref path) = guide.config_file_paths.macos {
                    println!("  {}", path);
                }
                #[cfg(target_os = "linux")]
                if let Some(ref path) = guide.config_file_paths.linux {
                    println!("  {}", path);
                }

                // 更新绑定状态
                let mut plan = plan;
                if let Some(agent) = plan.bound_agents.iter_mut().find(|a| a.agent_id == agent_id) {
                    agent.configured = true;
                    agent.config_status = AgentConfigStatus::AutoConfigured;
                }
                manager.update(plan).await?;

                println!();
                println!("✅ Agent auto-configuration steps generated.");
                println!("   Please apply the settings above to your agent.");
            }
            None => {
                anyhow::bail!("No setup guide found for agent '{}' in provider '{}'", agent_id, plan.provider_id);
            }
        }

        Ok(())
    }

    /// 查看 Agent 配置方法
    async fn handle_config(&self, agent_id: &str) -> Result<()> {
        let engine = ProviderEngine::new();
        let providers = engine.list_providers().await;

        let mut found = false;
        for provider in providers {
            if let Some(guide) = provider.onboarding.agent_setup_guides.iter().find(|g| g.agent_id == agent_id) {
                found = true;
                println!("📝 {} Setup Guide", guide.agent_name);
                println!("   Provider: {} ({})", provider.name, provider.provider_id);
                println!("   Auto Config Supported: {}", if guide.auto_config_supported { "Yes" } else { "No" });
                println!();

                if !guide.manual_steps.is_empty() {
                    println!("Manual Steps:");
                    for step in &guide.manual_steps {
                        println!("  {}. {}", step.step_number, step.description);
                        if let Some(ref cmd) = step.command {
                            println!("     Command: {}", cmd);
                        }
                        if let Some(ref text) = step.copyable_text {
                            println!("     Copy: {}", text);
                        }
                    }
                    println!();
                }

                if !guide.env_vars.is_empty() {
                    println!("Environment Variables:");
                    for env in &guide.env_vars {
                        println!("  {} - {}", env.name, env.description);
                        println!("     Value: {}", env.value);
                    }
                    println!();
                }

                println!("Config File Paths:");
                #[cfg(target_os = "windows")]
                {
                    println!("  Windows:");
                    if let Some(ref path) = guide.config_file_paths.windows {
                        println!("    {}", path);
                    }
                }
                #[cfg(target_os = "macos")]
                {
                    println!("  macOS:");
                    if let Some(ref path) = guide.config_file_paths.macos {
                        println!("    {}", path);
                    }
                }
                #[cfg(target_os = "linux")]
                {
                    println!("  Linux:");
                    if let Some(ref path) = guide.config_file_paths.linux {
                        println!("    {}", path);
                    }
                }

                break;
            }
        }

        if !found {
            anyhow::bail!("Agent '{}' not found in any provider", agent_id);
        }

        Ok(())
    }
}
