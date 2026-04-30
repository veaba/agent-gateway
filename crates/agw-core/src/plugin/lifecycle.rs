//! 插件生命周期管理

use anyhow::Result;

use super::{PluginEngine, PluginRegistry, PluginInfo};

/// 插件生命周期管理器
#[allow(dead_code)]
pub struct PluginLifecycle {
    engine: PluginEngine,
    registry: PluginRegistry,
}

impl PluginLifecycle {
    /// 创建新的生命周期管理器
    pub fn new() -> Self {
        Self {
            engine: PluginEngine::new().expect("Failed to create plugin engine"),
            registry: PluginRegistry::new(),
        }
    }

    /// 安装插件
    pub async fn install(&self, source: &str) -> Result<PluginInfo> {
        // TODO: 实现插件安装逻辑
        // 1. 解析源（本地文件、GitHub、远程 URL）
        // 2. 下载 WASM 模块
        // 3. 验证签名
        // 4. 加载模块
        // 5. 注册到注册表
        tracing::info!("Installing plugin from: {}", source);
        Ok(PluginInfo {
            id: "temp".to_string(),
            name: "Temp Plugin".to_string(),
            version: "0.1.0".to_string(),
            plugin_type: crate::model_types::PluginType::Provider,
            status: crate::model_types::PluginStatus::Installed,
            description: "".to_string(),
            author: "".to_string(),
            entry_point: "".to_string(),
        })
    }

    /// 卸载插件
    pub async fn uninstall(&self, id: &str) -> Result<()> {
        self.registry.unregister(id);
        tracing::info!("Uninstalled plugin: {}", id);
        Ok(())
    }

    /// 启用插件
    pub fn enable(&self, id: &str) -> Result<()> {
        if !self.registry.enable(id) {
            anyhow::bail!("Plugin not found: {}", id);
        }
        Ok(())
    }

    /// 禁用插件
    pub fn disable(&self, id: &str) -> Result<()> {
        if !self.registry.disable(id) {
            anyhow::bail!("Plugin not found: {}", id);
        }
        Ok(())
    }

    /// 获取注册表
    pub fn registry(&self) -> &PluginRegistry {
        &self.registry
    }
}

impl Default for PluginLifecycle {
    fn default() -> Self {
        Self::new()
    }
}