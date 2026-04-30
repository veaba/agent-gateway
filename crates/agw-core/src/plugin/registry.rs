//! 插件注册表

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
}

/// 插件注册表
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