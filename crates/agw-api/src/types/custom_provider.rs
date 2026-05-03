//! 自定义 Provider DTO 类型

use serde::{Deserialize, Serialize};

use agw_core::model::{CustomProvider, CustomModel, ApiFormat};

/// 创建自定义 Provider 请求
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCustomProviderRequest {
    /// Provider ID/标识符
    pub provider_id: String,
    /// 名称
    pub name: String,
    /// API 格式
    pub api_format: String,
    /// Base URL
    pub base_url: String,
    /// 是否需要 API Key
    #[serde(default = "default_true")]
    pub requires_api_key: bool,
    /// 描述
    #[serde(default)]
    pub description: Option<String>,
    /// Logo URL
    #[serde(default)]
    pub logo_url: Option<String>,
    /// 官网
    #[serde(default)]
    pub homepage: Option<String>,
    /// 文档 URL
    #[serde(default)]
    pub docs_url: Option<String>,
    /// 获取 API Key URL
    #[serde(default)]
    pub get_api_key_url: Option<String>,
    /// 模型列表
    #[serde(default)]
    pub models: Vec<CustomModelRequest>,
}

fn default_true() -> bool { true }

/// 创建自定义模型请求
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomModelRequest {
    /// 模型 ID
    pub model_id: String,
    /// 名称
    pub name: String,
    /// 描述
    #[serde(default)]
    pub description: Option<String>,
    /// 上下文长度
    #[serde(default)]
    pub context_length: Option<u64>,
    /// 能力标签
    #[serde(default)]
    pub capabilities: Vec<String>,
}

impl From<CustomModelRequest> for CustomModel {
    fn from(m: CustomModelRequest) -> Self {
        Self {
            model_id: m.model_id,
            name: m.name,
            description: m.description,
            context_length: m.context_length,
            capabilities: m.capabilities,
        }
    }
}

/// 更新自定义 Provider 请求
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCustomProviderRequest {
    /// 名称
    #[serde(default)]
    pub name: Option<String>,
    /// 描述
    #[serde(default)]
    pub description: Option<String>,
    /// Logo URL
    #[serde(default)]
    pub logo_url: Option<String>,
    /// 官网
    #[serde(default)]
    pub homepage: Option<String>,
    /// 文档 URL
    #[serde(default)]
    pub docs_url: Option<String>,
    /// 获取 API Key URL
    #[serde(default)]
    pub get_api_key_url: Option<String>,
    /// Base URL
    #[serde(default)]
    pub base_url: Option<String>,
    /// API 格式
    #[serde(default)]
    pub api_format: Option<String>,
    /// 是否需要 API Key
    #[serde(default)]
    pub requires_api_key: Option<bool>,
    /// 模型列表
    #[serde(default)]
    pub models: Option<Vec<CustomModelRequest>>,
}

/// 自定义 Provider 响应
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomProviderResponse {
    pub id: String,
    pub provider_id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logo_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docs_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub get_api_key_url: Option<String>,
    pub api_format: String,
    pub base_url: String,
    pub requires_api_key: bool,
    pub models: Vec<CustomModelResponse>,
    pub created_at: String,
    pub updated_at: String,
    /// 是否为自定义 Provider
    pub is_custom: bool,
}

/// 自定义模型响应
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomModelResponse {
    pub model_id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_length: Option<u64>,
    pub capabilities: Vec<String>,
}

impl From<CustomModel> for CustomModelResponse {
    fn from(m: CustomModel) -> Self {
        Self {
            model_id: m.model_id,
            name: m.name,
            description: m.description,
            context_length: m.context_length,
            capabilities: m.capabilities,
        }
    }
}

/// 自定义 Provider 列表响应
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomProviderListResponse {
    pub custom_providers: Vec<CustomProviderResponse>,
}

impl From<CustomProvider> for CustomProviderResponse {
    fn from(p: CustomProvider) -> Self {
        Self {
            id: p.id,
            provider_id: p.provider_id,
            name: p.name,
            description: p.description,
            logo_url: p.logo_url,
            homepage: p.homepage,
            docs_url: p.docs_url,
            get_api_key_url: p.get_api_key_url,
            api_format: p.api_format.to_string(),
            base_url: p.base_url,
            requires_api_key: p.requires_api_key,
            models: p.models.into_iter().map(CustomModelResponse::from).collect(),
            created_at: p.created_at.to_rfc3339(),
            updated_at: p.updated_at.to_rfc3339(),
            is_custom: true,
        }
    }
}

/// 将字符串转换为 ApiFormat
pub fn parse_api_format(s: &str) -> Result<ApiFormat, String> {
    match s.to_lowercase().as_str() {
        "anthropic" => Ok(ApiFormat::Anthropic),
        "openai" => Ok(ApiFormat::OpenAi),
        "custom" => Ok(ApiFormat::Custom),
        other => Err(format!("Unknown API format: {}", other)),
    }
}