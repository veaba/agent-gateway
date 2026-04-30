//! Plugin 命令

use anyhow::Result;
use clap::{Parser, Subcommand};

use agw_core::plugin::PluginLifecycle;
use agw_core::model_types::PluginType;

/// Plugin 命令
#[derive(Parser)]
pub struct PluginCommand {
    #[command(subcommand)]
    pub command: PluginSubcommand,
}

#[derive(Subcommand)]
pub enum PluginSubcommand {
    /// 列出所有插件
    List {
        /// 按类型过滤
        #[arg(short, long)]
        type_filter: Option<String>,
    },

    /// 安装插件
    Install {
        /// 安装源 (本地文件、GitHub、远程 URL)
        source: String,
    },

    /// 卸载插件
    Uninstall {
        /// 插件 ID
        id: String,
    },

    /// 更新插件
    Update {
        /// 插件 ID
        id: String,
        /// 更新源 (本地文件、GitHub、远程 URL，可选)
        source: Option<String>,
    },

    /// 启用插件
    Enable {
        /// 插件 ID
        id: String,
    },

    /// 禁用插件
    Disable {
        /// 插件 ID
        id: String,
    },

    /// 显示插件详情
    Info {
        /// 插件 ID
        id: String,
    },
}

impl PluginCommand {
    pub async fn run(&self) -> Result<()> {
        let lifecycle = PluginLifecycle::new();

        match &self.command {
            PluginSubcommand::List { type_filter } => {
                let plugins = lifecycle.registry().list();

                let filtered_plugins = if let Some(filter) = type_filter {
                    let plugin_type = parse_plugin_type(filter)?;
                    plugins.into_iter()
                        .filter(|p| p.plugin_type == plugin_type)
                        .collect()
                } else {
                    plugins
                };

                if filtered_plugins.is_empty() {
                    println!("No plugins installed.");
                } else {
                    println!("Installed plugins:");
                    println!();
                    println!("{:<20} {:<15} {:<10} {:<10} {:<30}", "ID", "NAME", "VERSION", "STATUS", "TYPE");
                    println!("{}", "-".repeat(85));

                    for plugin in filtered_plugins {
                        println!(
                            "{:<20} {:<15} {:<10} {:<10} {:<30}",
                            plugin.id,
                            truncate(&plugin.name, 15),
                            plugin.version,
                            plugin.status,
                            plugin.plugin_type
                        );
                    }
                }
            }

            PluginSubcommand::Install { source } => {
                println!("Installing plugin from: {}", source);
                let plugin = lifecycle.install(source).await?;

                println!();
                println!("Plugin installed successfully:");
                println!("  ID:          {}", plugin.id);
                println!("  Name:        {}", plugin.name);
                println!("  Version:     {}", plugin.version);
                println!("  Type:        {}", plugin.plugin_type);
                println!("  Description: {}", truncate(&plugin.description, 50));
                println!("  Author:      {}", plugin.author);
                println!();
                println!("Use 'agw plugin enable {}' to enable the plugin.", plugin.id);
            }

            PluginSubcommand::Uninstall { id } => {
                println!("Uninstalling plugin: {}", id);
                lifecycle.uninstall(id).await?;
                println!("Plugin {} uninstalled successfully.", id);
            }

            PluginSubcommand::Update { id, source } => {
                println!("Updating plugin: {}", id);
                let source_ref = source.as_deref();
                let plugin = lifecycle.update(id, source_ref).await?;

                println!();
                println!("Plugin updated successfully:");
                println!("  ID:          {}", plugin.id);
                println!("  Name:        {}", plugin.name);
                println!("  Version:     {}", plugin.version);
                println!("  Type:        {}", plugin.plugin_type);
                println!("  Description: {}", truncate(&plugin.description, 50));
                println!("  Author:      {}", plugin.author);
            }

            PluginSubcommand::Enable { id } => {
                lifecycle.enable(id)?;
                println!("Plugin {} enabled successfully.", id);
            }

            PluginSubcommand::Disable { id } => {
                lifecycle.disable(id)?;
                println!("Plugin {} disabled successfully.", id);
            }

            PluginSubcommand::Info { id } => {
                let plugin = lifecycle.get(id)
                    .ok_or_else(|| anyhow::anyhow!("Plugin not found: {}", id))?;

                println!("Plugin: {}", plugin.name);
                println!("{}", "-".repeat(40));
                println!("  ID:          {}", plugin.id);
                println!("  Version:     {}", plugin.version);
                println!("  Type:        {}", plugin.plugin_type);
                println!("  Status:      {}", plugin.status);
                println!("  Author:      {}", plugin.author);
                println!("  Description: {}", plugin.description);
                println!("  Entry Point: {}", plugin.entry_point);
                println!("  WASM Path:   {}", plugin.wasm_path.display());
            }
        }

        Ok(())
    }
}

/// 解析插件类型字符串
fn parse_plugin_type(s: &str) -> Result<PluginType> {
    match s.to_lowercase().as_str() {
        "provider" => Ok(PluginType::Provider),
        "transform" => Ok(PluginType::Transform),
        "tool" => Ok(PluginType::Tool),
        _ => anyhow::bail!("Invalid plugin type: {}. Valid types: provider, transform, tool", s),
    }
}

/// 截断字符串
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}...", &s[..max_len - 3])
    } else {
        s.to_string()
    }
}