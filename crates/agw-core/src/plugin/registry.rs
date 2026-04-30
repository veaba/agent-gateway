//! 插件注册表

use std::path::PathBuf;
use std::sync::Arc;
use dashmap::DashMap;

use crate::model_types::{PluginStatus, PluginType};

/// 插件信息
#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub plugin_type: PluginType,
    pub status: PluginStatus,
    pub description: String,
    pub author: String,
    pub entry_point: String,
    /// WASM 文件路径
    pub wasm_path: PathBuf,
}

/// 插件注册表
#[derive(Clone)]
pub struct PluginRegistry {
    plugins: Arc<DashMap<String, PluginInfo>>,
}

impl PluginRegistry {
    /// 创建新的注册表
    pub fn new() -> Self {
        Self {
            plugins: Arc::new(DashMap::new()),
        }
    }

    /// 注册插件
    pub fn register(&self, info: PluginInfo) {
        self.plugins.insert(info.id.clone(), info);
    }

    /// 注销插件
    pub fn unregister(&self, id: &str) -> bool {
        self.plugins.remove(id).is_some()
    }

    /// 获取插件
    pub fn get(&self, id: &str) -> Option<PluginInfo> {
        self.plugins.get(id).map(|r| r.clone())
    }

    /// 列出所有插件
    pub fn list(&self) -> Vec<PluginInfo> {
        self.plugins.iter().map(|r| r.value().clone()).collect()
    }

    /// 按类型列出插件
    pub fn list_by_type(&self, plugin_type: PluginType) -> Vec<PluginInfo> {
        self.plugins
            .iter()
            .filter(|r| r.value().plugin_type == plugin_type)
            .map(|r| r.value().clone())
            .collect()
    }

    /// 启用插件
    pub fn enable(&self, id: &str) -> bool {
        if let Some(mut info) = self.plugins.get_mut(id) {
            info.status = PluginStatus::Enabled;
            true
        } else {
            false
        }
    }

    /// 禁用插件
    pub fn disable(&self, id: &str) -> bool {
        if let Some(mut info) = self.plugins.get_mut(id) {
            info.status = PluginStatus::Disabled;
            true
        } else {
            false
        }
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_plugin(id: &str) -> PluginInfo {
        PluginInfo {
            id: id.to_string(),
            name: format!("Test Plugin {}", id),
            version: "1.0.0".to_string(),
            plugin_type: PluginType::Tool,
            status: PluginStatus::Installed,
            description: "Test plugin".to_string(),
            author: "Test Author".to_string(),
            entry_point: "main".to_string(),
            wasm_path: PathBuf::from("/tmp/test.wasm"),
        }
    }

    #[test]
    fn test_register_and_get() {
        let registry = PluginRegistry::new();
        let plugin = create_test_plugin("test-1");

        registry.register(plugin);
        let retrieved = registry.get("test-1");

        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.id, "test-1");
        assert_eq!(retrieved.name, "Test Plugin test-1");
    }

    #[test]
    fn test_unregister() {
        let registry = PluginRegistry::new();
        let plugin = create_test_plugin("test-2");

        registry.register(plugin);
        assert!(registry.get("test-2").is_some());

        let result = registry.unregister("test-2");
        assert!(result);
        assert!(registry.get("test-2").is_none());

        // Unregister non-existent plugin
        let result = registry.unregister("nonexistent");
        assert!(!result);
    }

    #[test]
    fn test_list() {
        let registry = PluginRegistry::new();

        registry.register(create_test_plugin("plugin-1"));
        registry.register(create_test_plugin("plugin-2"));
        registry.register(create_test_plugin("plugin-3"));

        let list = registry.list();
        assert_eq!(list.len(), 3);
    }

    #[test]
    fn test_list_by_type() {
        let registry = PluginRegistry::new();

        let mut tool_plugin = create_test_plugin("tool-1");
        tool_plugin.plugin_type = PluginType::Tool;

        let mut provider_plugin = create_test_plugin("provider-1");
        provider_plugin.plugin_type = PluginType::Provider;

        let mut transform_plugin = create_test_plugin("transform-1");
        transform_plugin.plugin_type = PluginType::Transform;

        registry.register(tool_plugin);
        registry.register(provider_plugin);
        registry.register(transform_plugin);

        let tool_plugins = registry.list_by_type(PluginType::Tool);
        assert_eq!(tool_plugins.len(), 1);
        assert_eq!(tool_plugins[0].id, "tool-1");

        let provider_plugins = registry.list_by_type(PluginType::Provider);
        assert_eq!(provider_plugins.len(), 1);
        assert_eq!(provider_plugins[0].id, "provider-1");
    }

    #[test]
    fn test_enable_disable() {
        let registry = PluginRegistry::new();
        registry.register(create_test_plugin("test-3"));

        // Enable
        let result = registry.enable("test-3");
        assert!(result);
        let plugin = registry.get("test-3").unwrap();
        assert_eq!(plugin.status, PluginStatus::Enabled);

        // Disable
        let result = registry.disable("test-3");
        assert!(result);
        let plugin = registry.get("test-3").unwrap();
        assert_eq!(plugin.status, PluginStatus::Disabled);

        // Enable non-existent
        let result = registry.enable("nonexistent");
        assert!(!result);
    }
}