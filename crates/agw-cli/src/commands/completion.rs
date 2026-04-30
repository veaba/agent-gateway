//! completion 命令

use anyhow::Result;
use clap::{Parser, ValueEnum};

/// Shell 补全命令
#[derive(Parser, Debug)]
pub struct CompletionCommand {
    /// Shell 类型
    #[arg(value_enum, default_value = "bash")]
    pub shell: Shell,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
    PowerShell,
    Elvish,
}

impl CompletionCommand {
    pub async fn run(&self) -> Result<()> {
        // TODO: 生成补全脚本
        match self.shell {
            Shell::Bash => {
                tracing::info!("Generating bash completion script");
            }
            Shell::Zsh => {
                tracing::info!("Generating zsh completion script");
            }
            Shell::Fish => {
                tracing::info!("Generating fish completion script");
            }
            Shell::PowerShell => {
                tracing::info!("Generating PowerShell completion script");
            }
            Shell::Elvish => {
                tracing::info!("Generating elvish completion script");
            }
        }
        Ok(())
    }
}