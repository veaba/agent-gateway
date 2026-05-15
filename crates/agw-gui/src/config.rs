//! Server 配置模块
//!
//! 嵌入式/外部服务器模式配置

use serde::{Serialize, Deserialize};
use std::path::PathBuf;

/// 服务器模式
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub enum ServerMode {
    /// 嵌入式模式：内置服务器，同时提供 proxy 和管理 API
    #[default]
    Embedded,
    /// 外部模式：连接到外部 API 服务器
    External,
}

/// Server 配置
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ServerConfig {
    /// 服务器模式
    pub mode: ServerMode,
    /// 嵌入式服务器监听地址
    pub embedded_listen: String,
    /// 外部服务器 endpoint (仅 External 模式)
    pub external_endpoint: Option<String>,
    /// 是否自动启动服务器
    pub auto_start: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            mode: ServerMode::Embedded,
            embedded_listen: "127.0.0.1:8080".to_string(),
            external_endpoint: None,
            auto_start: true,
        }
    }
}

impl ServerConfig {
    /// 加载配置
    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(config) = serde_yaml::from_str::<Self>(&content) {
                    tracing::info!("Loaded server config from {}", path.display());
                    return config;
                }
            }
        }
        Self::default()
    }

    /// 保存配置
    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::config_path();
        // 确保目录存在
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_yaml::to_string(self)?;
        std::fs::write(&path, content)?;
        tracing::info!("Saved server config to {}", path.display());
        Ok(())
    }

    /// 配置文件路径
    pub fn config_path() -> PathBuf {
        agw_core::paths::gui_server_config_path()
    }

    /// 获取实际 endpoint
    pub fn get_endpoint(&self) -> String {
        match self.mode {
            ServerMode::Embedded => format!("http://{}", self.embedded_listen),
            ServerMode::External => self.external_endpoint.clone()
                .unwrap_or_else(|| "http://127.0.0.1:8081".to_string()),
        }
    }
}