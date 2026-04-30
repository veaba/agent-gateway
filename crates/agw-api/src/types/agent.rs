//! Agent DTO 类型

use serde::{Deserialize, Serialize};

use agw_core::model::{AgentTool, AgentToolRef};

/// Agent 列表响应
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentListResponse {
    pub agents: Vec<AgentResponse>,
}

/// Agent 响应
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentResponse {
    pub agent_id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logo_url: Option<String>,
    pub homepage: String,
    pub install_url: String,
    pub supported_formats: Vec<String>,
    pub config_methods: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub setup_guide: Option<AgentSetupGuideResponse>,
}

/// Agent 设置指南响应
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentSetupGuideResponse {
    pub agent_id: String,
    pub agent_name: String,
    pub auto_config_supported: bool,
    pub manual_steps: Vec<SetupStepResponse>,
    pub config_file_paths: PlatformPathsResponse,
    pub env_vars: Vec<EnvVarResponse>,
}

/// 配置步骤响应
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetupStepResponse {
    pub step_number: u32,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub copyable_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// 跨平台路径响应
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformPathsResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub macos: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linux: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub windows: Option<String>,
}

/// 环境变量配置响应
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvVarResponse {
    pub name: String,
    pub value: String,
    pub description: String,
}

/// Agent 绑定请求（空体）
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BindAgentRequest {
    #[serde(default)]
    pub auto_config: bool,
}

/// Agent 绑定结果响应
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentBindResponse {
    pub plan_id: String,
    pub agent_id: String,
    pub bound: bool,
}

/// Agent 解绑结果响应
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentUnbindResponse {
    pub plan_id: String,
    pub agent_id: String,
    pub unbound: bool,
}

/// Agent 自动配置响应
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentAutoConfigResponse {
    pub plan_id: String,
    pub agent_id: String,
    pub success: bool,
    pub message: String,
}

impl From<AgentTool> for AgentResponse {
    fn from(agent: AgentTool) -> Self {
        Self {
            agent_id: agent.agent_id.clone(),
            name: agent.name,
            description: Some(agent.description),
            logo_url: agent.logo_url,
            homepage: agent.homepage,
            install_url: agent.install_url,
            supported_formats: agent.supported_formats.iter().map(|f| f.to_string()).collect(),
            config_methods: agent.config_methods.iter().map(|m| format!("{:?}", m)).collect(),
            setup_guide: None,
        }
    }
}

impl From<AgentToolRef> for AgentResponse {
    fn from(agent: AgentToolRef) -> Self {
        Self {
            agent_id: agent.agent_id.clone(),
            name: agent.name,
            description: None,
            logo_url: None,
            homepage: String::new(),
            install_url: String::new(),
            supported_formats: Vec::new(),
            config_methods: Vec::new(),
            setup_guide: None,
        }
    }
}
