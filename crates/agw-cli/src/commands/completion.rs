//! completion 命令

use anyhow::Result;
use clap::{CommandFactory, Parser, ValueEnum};
use clap_complete::{generate, Shell as ClapShell};

use crate::Cli;

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
        let shell = match self.shell {
            Shell::Bash => ClapShell::Bash,
            Shell::Zsh => ClapShell::Zsh,
            Shell::Fish => ClapShell::Fish,
            Shell::PowerShell => ClapShell::PowerShell,
            Shell::Elvish => ClapShell::Elvish,
        };

        generate(shell, &mut Cli::command(), "agw", &mut std::io::stdout());
        Ok(())
    }
}