//! Provider 插件扩展
//!
//! 支持通过 WASM 插件扩展 Provider，包括自定义 API 格式转换和请求处理

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::lifecycle::PluginLifecycle;
use crate::model_types::PluginType;

/// Provider 插件转换接口
///
/// 插件可以实现此接口来自定义 Provider 的请求/响应处理
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderPluginTransform {
    /// 插件 ID
    pub plugin_id: String,
    /// 支持的 Provider ID（空表示所有 Provider）
    pub provider_id: Option<String>,
    /// 支持的 API 格式
    pub supported_formats: Vec<String>,
    /// 转换优先级（数字越小优先级越高）
    pub priority: u32,
}

/// 请求转换结果
#[derive(Debug, Clone)]
pub struct TransformResult {
    /// 转换后的请求头
    pub headers: HashMap<String, String>,
    /// 转换后的请求体
    pub body: Vec<u8>,
    /// 转换后的 URL（如果 URL 被修改）
    pub url: Option<String>,
    /// 转换后的 HTTP 方法
    pub method: Option<String>,
}

/// Provider 插件管理器
pub struct ProviderPluginManager {
    /// 插件生命周期管理器
    lifecycle: std::sync::Arc<PluginLifecycle>,
}

impl ProviderPluginManager {
    /// 创建新的 Provider 插件管理器
    pub fn new(lifecycle: std::sync::Arc<PluginLifecycle>) -> Self {
        Self { lifecycle }
    }

    /// 获取所有 Provider 类型的插件
    pub fn get_provider_plugins(&self) -> Vec<super::registry::PluginInfo> {
        self.lifecycle.registry().list_by_type(PluginType::Provider)
    }

    /// 获取所有 Transform 类型的插件
    pub fn get_transform_plugins(&self) -> Vec<super::registry::PluginInfo> {
        self.lifecycle.registry().list_by_type(PluginType::Transform)
    }

    /// 通过插件转换请求
    pub async fn transform_request(
        &self,
        plugin_id: &str,
        headers: &HashMap<String, String>,
        body: &[u8],
        url: &str,
        method: &str,
    ) -> Result<TransformResult> {
        let plugin = self.lifecycle.get(plugin_id)
            .ok_or_else(|| anyhow::anyhow!("Plugin not found: {}", plugin_id))?;

        if plugin.status != crate::model_types::PluginStatus::Enabled {
            anyhow::bail!("Plugin {} is not enabled", plugin_id);
        }

        let input = serde_json::json!({
            "type": "request",
            "method": method,
            "url": url,
            "headers": headers,
            "body": body,
        });

        let input_str = serde_json::to_string(&input)?;
        let output = self.lifecycle.execute(plugin_id, "transform_request", input_str.as_bytes()).await?;

        let result: serde_json::Value = serde_json::from_slice(&output)
            .map_err(|e| anyhow::anyhow!("Failed to parse plugin output: {}", e))?;

        let transformed_headers = result.get("headers")
            .and_then(|h| serde_json::from_value::<HashMap<String, String>>(h.clone()).ok())
            .unwrap_or_default();

        let transformed_body = result.get("body")
            .and_then(|b| b.as_str())
            .map(|s| s.as_bytes().to_vec())
            .unwrap_or_else(|| body.to_vec());

        let transformed_url = result.get("url")
            .and_then(|u| u.as_str())
            .map(|s| s.to_string());

        let transformed_method = result.get("method")
            .and_then(|m| m.as_str())
            .map(|s| s.to_string());

        Ok(TransformResult {
            headers: transformed_headers,
            body: transformed_body,
            url: transformed_url,
            method: transformed_method,
        })
    }

    /// 通过插件转换响应
    pub async fn transform_response(
        &self,
        plugin_id: &str,
        status_code: u16,
        headers: &HashMap<String, String>,
        body: &[u8],
    ) -> Result<TransformResult> {
        let plugin = self.lifecycle.get(plugin_id)
            .ok_or_else(|| anyhow::anyhow!("Plugin not found: {}", plugin_id))?;

        if plugin.status != crate::model_types::PluginStatus::Enabled {
            anyhow::bail!("Plugin {} is not enabled", plugin_id);
        }

        let input = serde_json::json!({
            "type": "response",
            "status_code": status_code,
            "headers": headers,
            "body": body,
        });

        let input_str = serde_json::to_string(&input)?;
        let output = self.lifecycle.execute(plugin_id, "transform_response", input_str.as_bytes()).await?;

        let result: serde_json::Value = serde_json::from_slice(&output)
            .map_err(|e| anyhow::anyhow!("Failed to parse plugin output: {}", e))?;

        let transformed_headers = result.get("headers")
            .and_then(|h| serde_json::from_value::<HashMap<String, String>>(h.clone()).ok())
            .unwrap_or_default();

        let transformed_body = result.get("body")
            .and_then(|b| b.as_str())
            .map(|s| s.as_bytes().to_vec())
            .unwrap_or_else(|| body.to_vec());

        Ok(TransformResult {
            headers: transformed_headers,
            body: transformed_body,
            url: None,
            method: None,
        })
    }
}

/// Transform 管道
///
/// 按优先级顺序依次通过多个 Transform 插件处理请求/响应
pub struct TransformPipeline {
    /// 管道步骤列表（plugin_id, priority）
    steps: Vec<(String, u32)>,
}

impl TransformPipeline {
    /// 创建空的管道
    pub fn new() -> Self {
        Self { steps: Vec::new() }
    }

    /// 添加步骤到管道
    pub fn add_step(&mut self, plugin_id: String, priority: u32) {
        self.steps.push((plugin_id, priority));
        self.steps.sort_by_key(|(_, p)| *p);
    }

    /// 获取管道步骤
    pub fn steps(&self) -> &[(String, u32)] {
        &self.steps
    }

    /// 检查管道是否为空
    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }
}

impl Default for TransformPipeline {
    fn default() -> Self {
        Self::new()
    }
}