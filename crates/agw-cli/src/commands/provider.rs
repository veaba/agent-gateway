//! provider 命令

use anyhow::Result;
use clap::{Parser, Subcommand};

use agw_core::business::ProviderEngine;
use agw_core::storage::ConfigStore;

/// Provider 管理命令
#[derive(Parser, Debug)]
pub struct ProviderCommand {
    #[command(subcommand)]
    pub command: ProviderSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum ProviderSubcommand {
    /// 列出所有 Provider
    List {
        /// 显示详细信息
        #[arg(long)]
        verbose: bool,
        /// 只显示内置 Provider
        #[arg(long)]
        builtin: bool,
        /// 只显示自定义 Provider
        #[arg(long)]
        custom: bool,
    },
    /// 查看 Provider 详情
    Info {
        /// Provider ID
        provider_id: String,
        /// 显示完整配置
        #[arg(long)]
        full: bool,
    },
    /// 更新 Provider 配置（从远程）
    Update {
        /// 强制更新
        #[arg(long)]
        force: bool,
    },
    /// 添加自定义 Provider
    Add {
        /// Provider 配置文件路径（YAML）
        config_path: String,
    },
}

impl ProviderCommand {
    pub async fn run(&self) -> Result<()> {
        let engine = ProviderEngine::new();

        match &self.command {
            ProviderSubcommand::List { verbose, builtin, custom } => {
                self.handle_list(&engine, *verbose, *builtin, *custom).await?;
            }
            ProviderSubcommand::Info { provider_id, full } => {
                self.handle_info(&engine, provider_id, *full).await?;
            }
            ProviderSubcommand::Update { force } => {
                self.handle_update(&engine, *force).await?;
            }
            ProviderSubcommand::Add { config_path } => {
                self.handle_add(config_path).await?;
            }
        }
        Ok(())
    }

    /// 列出 Provider
    async fn handle_list(
        &self,
        engine: &ProviderEngine,
        verbose: bool,
        builtin: bool,
        custom: bool,
    ) -> Result<()> {
        let providers = engine.list_providers().await;

        // 过滤
        let filtered: Vec<_> = providers.into_iter().filter(|p| {
            // 如果同时指定 builtin 和 custom，显示所有
            if builtin && custom {
                return true;
            }
            // 只显示 builtin（内置 provider ID 在 engine 初始化时定义）
            if builtin && !custom {
                return p.provider_id == "alaya" || p.provider_id == "anthropic";
            }
            // 只显示 custom
            if custom && !builtin {
                return p.provider_id != "alaya" && p.provider_id != "anthropic";
            }
            // 默认显示所有
            true
        }).collect();

        if filtered.is_empty() {
            println!("No providers found.");
            if !builtin && !custom {
                println!("Use 'agw provider add <config_path>' to add a custom provider.");
            }
            return Ok(());
        }

        println!("Providers ({})", filtered.len());
        println!();

        for provider in filtered {
            if verbose {
                println!("📦 {}", provider.name);
                println!("  ID: {}", provider.provider_id);
                println!("  Description: {}", provider.description);
                println!("  API Format: {}", provider.api_format);
                println!("  Homepage: {}", provider.homepage);
                if let Some(ref url) = provider.get_api_key_url {
                    println!("  Get API Key: {}", url);
                }
                println!("  Coding Plans: {}", provider.coding_plans.len());
                for plan in &provider.coding_plans {
                    println!("    - {} ({})", plan.name, plan.tier);
                    println!("      Models: {}", plan.supported_model_ids.join(", "));
                    if let Some(ref price) = plan.price {
                        println!("      Price: {}", price);
                    }
                }
                println!("  Models: {}", provider.models.len());
                for model in &provider.models {
                    println!("    - {} (context: {})", model.name,
                        model.context_length.map(|c| c.to_string()).unwrap_or_else(|| "N/A".to_string()));
                }
                println!("  Supported Agents: {}", provider.supported_agents.len());
                for agent in &provider.supported_agents {
                    println!("    - {} ({})", agent.name, agent.agent_id);
                }
                println!("  Version: {}", provider.version);
                println!();
            } else {
                let plans_count = provider.coding_plans.len();
                let models_count = provider.models.len();
                println!("📦 {} ({}) - {} plans, {} models",
                    provider.name,
                    provider.provider_id,
                    plans_count,
                    models_count
                );
            }
        }

        Ok(())
    }

    /// 查看 Provider 详情
    async fn handle_info(
        &self,
        engine: &ProviderEngine,
        provider_id: &str,
        full: bool,
    ) -> Result<()> {
        let provider = engine.get_provider(provider_id).await
            .ok_or_else(|| anyhow::anyhow!("Provider '{}' not found", provider_id))?;

        println!("📦 {}", provider.name);
        println!("  ID: {}", provider.provider_id);
        println!("  Description: {}", provider.description);
        println!("  API Format: {}", provider.api_format);
        println!("  Homepage: {}", provider.homepage);
        println!("  Docs: {}", provider.docs_url);

        if let Some(ref url) = provider.get_api_key_url {
            println!("  Get API Key: {}", url);
        }
        if let Some(ref url) = provider.setup_guide_url {
            println!("  Setup Guide: {}", url);
        }

        println!();
        println!("📋 Coding Plans ({})", provider.coding_plans.len());
        for plan in &provider.coding_plans {
            println!();
            println!("  {} ({}) - {}", plan.name, plan.plan_id, plan.description);
            println!("    Tier: {}", plan.tier);
            println!("    Default Model: {}", plan.default_model_id);
            println!("    Supported Models: {}", plan.supported_model_ids.join(", "));
            println!("    Supported Agents: {}", plan.supported_agent_ids.join(", "));
            if let Some(ref daily) = plan.quota_daily {
                println!("    Daily Quota: {}", daily);
            }
            if let Some(ref monthly) = plan.quota_monthly {
                println!("    Monthly Quota: {}", monthly);
            }
            if let Some(ref rpm) = plan.rpm_limit {
                println!("    RPM Limit: {}", rpm);
            }
            if let Some(ref price) = plan.price {
                println!("    Price: {}", price);
            }
            println!("    Features: {}", plan.features.join(", "));
        }

        println!();
        println!("🤖 Models ({})", provider.models.len());
        for model in &provider.models {
            println!();
            println!("  {} ({})", model.name, model.model_id);
            if let Some(ref desc) = model.description {
                println!("    Description: {}", desc);
            }
            if let Some(ref ctx) = model.context_length {
                println!("    Context Length: {} tokens", ctx);
            }
            let caps: Vec<_> = model.capabilities.iter().map(|c| c.to_string()).collect();
            println!("    Capabilities: {}", caps.join(", "));
        }

        println!();
        println!("🔧 Supported Agents ({})", provider.supported_agents.len());
        for agent in &provider.supported_agents {
            println!("  {} ({})", agent.name, agent.agent_id);
        }

        if full {
            println!();
            println!("📖 Onboarding Info");
            println!("  {}", provider.onboarding.description);
            println!("  Signup: {}", provider.onboarding.signup_url);
            if let Some(ref url) = provider.onboarding.plans_comparison_url {
                println!("  Plans Comparison: {}", url);
            }
            if let Some(ref url) = provider.onboarding.get_key_url {
                println!("  Get Key: {}", url);
            }
            if let Some(ref url) = provider.onboarding.setup_guide_url {
                println!("  Setup Guide: {}", url);
            }
            for guide in &provider.onboarding.agent_setup_guides {
                println!();
                println!("  📝 {} Setup Guide", guide.agent_name);
                println!("    Auto Config Supported: {}", guide.auto_config_supported);
                for step in &guide.manual_steps {
                    println!("    {}. {}", step.step_number, step.description);
                    if let Some(ref cmd) = step.command {
                        println!("       Command: {}", cmd);
                    }
                    if let Some(ref text) = step.copyable_text {
                        println!("       Copy: {}", text);
                    }
                }
            }
        }

        Ok(())
    }

    /// 更新 Provider
    async fn handle_update(&self, engine: &ProviderEngine, force: bool) -> Result<()> {
        println!("Checking for provider updates...");

        let update_info = engine.check_update().await?;

        match update_info {
            Some(report) => {
                println!("New version available: {} (current: 0.1.0)", report.new_version);
                println!("Changes:");
                for change in &report.changes {
                    println!("  - {}", change);
                }
                println!("Updated providers: {}", report.updated_providers.join(", "));

                if force {
                    println!();
                    println!("Downloading and applying updates...");
                    let updated_ids = engine.apply_update().await?;
                    if updated_ids.is_empty() {
                        println!("No updates were applied.");
                    } else {
                        println!("✅ Successfully updated {} providers:", updated_ids.len());
                        for id in updated_ids {
                            println!("  - {}", id);
                        }
                    }
                } else {
                    println!();
                    println!("Use 'agw provider update --force' to apply update.");
                }
            }
            None => {
                println!("No updates available. Current version is up to date.");
            }
        }

        Ok(())
    }

    /// 添加自定义 Provider
    async fn handle_add(&self, config_path: &str) -> Result<()> {
        // 读取配置文件
        let content = std::fs::read_to_string(config_path)?;

        // 解析 YAML
        let provider: agw_core::model::ProviderTemplate = serde_yaml::from_str(&content)?;

        println!("Adding custom provider: {}", provider.name);
        println!("  ID: {}", provider.provider_id);
        println!("  API Format: {}", provider.api_format);

        // 保存到配置目录
        let config_store = ConfigStore::new()?;
        let custom_providers_dir = config_store.config_dir().join("custom_providers");
        std::fs::create_dir_all(&custom_providers_dir)?;

        let file_name = format!("{}.yaml", provider.provider_id);
        let file_path = custom_providers_dir.join(&file_name);

        std::fs::write(&file_path, &content)?;

        println!("✅ Custom provider '{}' saved to: {}",
            provider.provider_id,
            file_path.display()
        );
        println!("Restart gateway to apply the new provider.");

        Ok(())
    }
}