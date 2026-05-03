//! 自定义 Agent DTO 类型

use serde::{Deserialize, Serialize};

use agw_core::model::CustomAgent;

/// 创建自定义 Agent 请求
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCustomAgentRequest {
    /// 工具代码/标识符
    pub agent_id: String,
    /// 工具名称
    pub name: String,
    /// 版本号
    pub version: String,
    /// 图标 URL
    #[serde(default)]
    pub logo_url: Option<String>,
    /// 描述
    #[serde(default)]
    pub description: Option<String>,
}

/// 更新自定义 Agent 请求
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCustomAgentRequest {
    /// 工具名称
    #[serde(default)]
    pub name: Option<String>,
    /// 版本号
    #[serde(default)]
    pub version: Option<String>,
    /// 图标 URL
    #[serde(default)]
    pub logo_url: Option<String>,
    /// 描述
    #[serde(default)]
    pub description: Option<String>,
}

/// 自定义 Agent 响应
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomAgentResponse {
    pub id: String,
    pub agent_id: String,
    pub name: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logo_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    /// 是否为自定义 Agent
    pub is_custom: bool,
}

/// 自定义 Agent 列表响应
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomAgentListResponse {
    pub custom_agents: Vec<CustomAgentResponse>,
}

impl From<CustomAgent> for CustomAgentResponse {
    fn from(agent: CustomAgent) -> Self {
        Self {
            id: agent.id,
            agent_id: agent.agent_id,
            name: agent.name,
            version: agent.version,
            logo_url: agent.logo_url,
            description: agent.description,
            created_at: agent.created_at.to_rfc3339(),
            updated_at: agent.updated_at.to_rfc3339(),
            is_custom: true,
        }
    }
}