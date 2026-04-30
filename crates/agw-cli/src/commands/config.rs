//! config 命令

use anyhow::Result;
use clap::Parser;

use std::process::Command;

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
        if self.edit {
            // 使用默认编辑器打开配置
            let config_path = dirs::config_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .join("agent-gateway");

            tracing::info!("Opening config dir: {:?}", config_path);

            #[cfg(target_os = "windows")]
            Command::new("explorer").arg(&config_path).spawn()?;
            #[cfg(target_os = "macos")]
            Command::new("open").arg(&config_path).spawn()?;
            #[cfg(target_os = "linux")]
            Command::new("xdg-open").arg(&config_path).spawn()?;
        } else if self.show {
            tracing::info!("Showing current configuration");
        }
        Ok(())
    }
}