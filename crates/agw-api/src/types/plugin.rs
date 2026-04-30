//! Plugin 插件 DTO 类型

use serde::{Deserialize, Serialize};

use agw_core::plugin::registry::PluginInfo;

/// 插件列表响应
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginListResponse {
    pub plugins: Vec<PluginResponse>,
}

/// 插件响应
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginResponse {
    pub id: String,
    pub name: String,
    pub version: String,
    pub plugin_type: String,
    pub status: String,
    pub description: String,
    pub author: String,
}

/// 插件安装请求
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallPluginRequest {
    pub source: String,
}

/// 插件更新请求
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePluginRequest {
    pub source: Option<String>,
}

impl From<PluginInfo> for PluginResponse {
    fn from(info: PluginInfo) -> Self {
        Self {
            id: info.id,
            name: info.name,
            version: info.version,
            plugin_type: info.plugin_type.to_string(),
            status: info.status.to_string(),
            description: info.description,
            author: info.author,
        }
    }
}
