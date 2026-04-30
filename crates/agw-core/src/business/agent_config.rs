//! Agent 工具自动配置
//!
//! 自动配置 AI 编码工具（Claude Code、Kimi CLI 等）连接到 agent-gateway

use std::path::PathBuf;
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// 配置报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigReport {
    /// Agent 名称
    pub agent: String,
    /// 配置方式描述
    pub method: String,
    /// 修改的文件路径
    pub paths: Vec<String>,
    /// 是否需要重新加载 shell
    pub requires_reload: bool,
    /// 重新加载命令
    pub reload_command: Option<String>,
}

/// Agent 自动配置器
pub struct AgentAutoConfig;

impl AgentAutoConfig {
    /// 自动配置 Agent 工具连接到网关
    pub async fn configure(
        agent_id: &str,
        gateway_addr: &str,
    ) -> Result<ConfigReport> {
        match agent_id {
            "claude-code" => Self::configure_claude_code(gateway_addr).await,
            "kimi-cli" => Self::configure_kimi_cli(gateway_addr).await,
            "opencode" => Self::configure_opencode(gateway_addr).await,
            "kilo-cli" => Self::configure_kilo_cli(gateway_addr).await,
            _ => anyhow::bail!("Unknown agent: {}", agent_id),
        }
    }

    /// 配置 Claude Code
    async fn configure_claude_code(gateway_addr: &str) -> Result<ConfigReport> {
        let shell = Self::detect_shell();
        let rc_file = Self::get_rc_file(&shell);
        let rc_path = shellexpand::tilde(&rc_file).to_string();
        let rc_path_buf = PathBuf::from(&rc_path);

        let base_url = format!("http://{}", gateway_addr);
        let env_block = format!(
            "\n# Added by agent-gateway\n\
             export ANTHROPIC_BASE_URL={}\n\
             export ANTHROPIC_API_KEY=\"dummy\"\n",
            base_url
        );

        let mut appended = false;
        if rc_path_buf.exists() {
            let content = tokio::fs::read_to_string(&rc_path_buf).await?;
            if !content.contains("ANTHROPIC_BASE_URL") {
                tokio::fs::OpenOptions::new()
                    .append(true)
                    .open(&rc_path_buf)
                    .await?
                    .write_all(env_block.as_bytes())
                    .await?;
                appended = true;
            }
        } else {
            if let Some(parent) = rc_path_buf.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }
            tokio::fs::write(&rc_path_buf, env_block).await?;
            appended = true;
        }

        let reload_cmd = format!("source {}", rc_file);
        Ok(ConfigReport {
            agent: "Claude Code".to_string(),
            method: "Environment variables in shell rc file".to_string(),
            paths: vec![rc_path.clone()],
            requires_reload: appended,
            reload_command: if appended { Some(reload_cmd) } else { None },
        })
    }

    /// 配置 Kimi CLI
    async fn configure_kimi_cli(gateway_addr: &str) -> Result<ConfigReport> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Cannot find config directory"))?
            .join("kimi");

        tokio::fs::create_dir_all(&config_dir).await?;

        let config_path = config_dir.join("config.yaml");
        let base_url = format!("http://{}/v1", gateway_addr);
        let config_content = format!(
            "api: anthropic-messages\n\
             baseUrl: {}\n\
             apiKey: \"dummy\"\n",
            base_url
        );

        tokio::fs::write(&config_path, config_content).await?;

        Ok(ConfigReport {
            agent: "Kimi CLI".to_string(),
            method: "Config file".to_string(),
            paths: vec![config_path.to_string_lossy().to_string()],
            requires_reload: false,
            reload_command: None,
        })
    }

    /// 配置 OpenCode
    async fn configure_opencode(gateway_addr: &str) -> Result<ConfigReport> {
        let shell = Self::detect_shell();
        let rc_file = Self::get_rc_file(&shell);
        let rc_path = shellexpand::tilde(&rc_file).to_string();
        let rc_path_buf = PathBuf::from(&rc_path);

        let base_url = format!("http://{}", gateway_addr);
        let env_block = format!(
            "\n# Added by agent-gateway (OpenCode)\n\
             export OPENAI_BASE_URL={}/v1\n\
             export OPENAI_API_KEY=\"dummy\"\n",
            base_url
        );

        let mut appended = false;
        if rc_path_buf.exists() {
            let content = tokio::fs::read_to_string(&rc_path_buf).await?;
            if !content.contains("OPENAI_BASE_URL") {
                tokio::fs::OpenOptions::new()
                    .append(true)
                    .open(&rc_path_buf)
                    .await?
                    .write_all(env_block.as_bytes())
                    .await?;
                appended = true;
            }
        } else {
            if let Some(parent) = rc_path_buf.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }
            tokio::fs::write(&rc_path_buf, env_block).await?;
            appended = true;
        }

        let reload_cmd = format!("source {}", rc_file);
        Ok(ConfigReport {
            agent: "OpenCode".to_string(),
            method: "Environment variables in shell rc file".to_string(),
            paths: vec![rc_path.clone()],
            requires_reload: appended,
            reload_command: if appended { Some(reload_cmd) } else { None },
        })
    }

    /// 配置 Kilo CLI
    async fn configure_kilo_cli(gateway_addr: &str) -> Result<ConfigReport> {
        let shell = Self::detect_shell();
        let rc_file = Self::get_rc_file(&shell);
        let rc_path = shellexpand::tilde(&rc_file).to_string();
        let rc_path_buf = PathBuf::from(&rc_path);

        let base_url = format!("http://{}", gateway_addr);
        let env_block = format!(
            "\n# Added by agent-gateway (Kilo CLI)\n\
             export KILO_BASE_URL={}/v1\n\
             export KILO_API_KEY=\"dummy\"\n",
            base_url
        );

        let mut appended = false;
        if rc_path_buf.exists() {
            let content = tokio::fs::read_to_string(&rc_path_buf).await?;
            if !content.contains("KILO_BASE_URL") {
                tokio::fs::OpenOptions::new()
                    .append(true)
                    .open(&rc_path_buf)
                    .await?
                    .write_all(env_block.as_bytes())
                    .await?;
                appended = true;
            }
        } else {
            if let Some(parent) = rc_path_buf.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }
            tokio::fs::write(&rc_path_buf, env_block).await?;
            appended = true;
        }

        let reload_cmd = format!("source {}", rc_file);
        Ok(ConfigReport {
            agent: "Kilo CLI".to_string(),
            method: "Environment variables in shell rc file".to_string(),
            paths: vec![rc_path.clone()],
            requires_reload: appended,
            reload_command: if appended { Some(reload_cmd) } else { None },
        })
    }

    /// 检测当前 shell
    fn detect_shell() -> String {
        if cfg!(target_os = "windows") {
            "powershell".to_string()
        } else {
            std::env::var("SHELL")
                .map(|s| {
                    if s.contains("zsh") { "zsh".to_string() }
                    else if s.contains("bash") { "bash".to_string() }
                    else if s.contains("fish") { "fish".to_string() }
                    else { s.clone() }
                })
                .unwrap_or_else(|_| "bash".to_string())
        }
    }

    /// 获取 shell 配置文件路径
    fn get_rc_file(shell: &str) -> String {
        match shell {
            "zsh" => "~/.zshrc".to_string(),
            "fish" => "~/.config/fish/config.fish".to_string(),
            "powershell" => {
                let profile = dirs::document_dir()
                    .map(|d| d.join("PowerShell").join("Microsoft.VSCode_profile.ps1"))
                    .unwrap_or_else(|| PathBuf::from("$PROFILE"));
                profile.to_string_lossy().to_string()
            }
            _ => "~/.bashrc".to_string(),
        }
    }

    /// 获取 Agent 的配置指南（从 ProviderOnboarding 获取）
    pub fn get_setup_guide<'a>(
        provider_onboarding: &'a crate::model::ProviderOnboarding,
        agent_id: &str,
    ) -> Option<&'a crate::model::AgentSetupGuide> {
        provider_onboarding
            .agent_setup_guides
            .iter()
            .find(|g| g.agent_id == agent_id)
    }
}

use tokio::io::AsyncWriteExt;