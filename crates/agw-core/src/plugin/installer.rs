//! 插件安装器

use anyhow::Result;
use std::path::PathBuf;

/// 插件安装器
pub struct PluginInstaller;

impl PluginInstaller {
    /// 从本地文件安装
    pub async fn install_from_file(path: &PathBuf) -> Result<Vec<u8>> {
        let bytes = tokio::fs::read(path).await?;
        Self::validate_wasm(&bytes)?;
        Ok(bytes)
    }

    /// 从 GitHub 安装
    pub async fn install_from_github(owner: &str, repo: &str, version: &str) -> Result<Vec<u8>> {
        let url = format!(
            "https://github.com/{}/{}/releases/download/{}/plugin.wasm",
            owner, repo, version
        );

        let response = reqwest::get(&url).await?;
        let bytes = response.bytes().await?.to_vec();

        Self::validate_wasm(&bytes)?;
        Ok(bytes)
    }

    /// 从远程 URL 安装
    pub async fn install_from_url(url: &str) -> Result<Vec<u8>> {
        let response = reqwest::get(url).await?;
        let bytes = response.bytes().await?.to_vec();

        Self::validate_wasm(&bytes)?;
        Ok(bytes)
    }

    /// 验证 WASM 模块
    fn validate_wasm(bytes: &[u8]) -> Result<()> {
        // 检查 WASM magic number
        if bytes.len() < 8 {
            anyhow::bail!("Invalid WASM file: too short");
        }

        if &bytes[0..4] != b"\0asm" {
            anyhow::bail!("Invalid WASM file: bad magic number");
        }

        Ok(())
    }

    /// 获取插件安装目录
    pub fn plugin_dir() -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("agent-gateway")
            .join("plugins")
    }
}