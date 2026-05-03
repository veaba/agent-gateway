//! 核心数据模型定义

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub use super::model_types::*;

// ============================================================================
// Provider 模板（内置，可远程更新）
// ============================================================================

/// Provider 模板
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderTemplate {
    pub provider_id: String,
    pub name: String,
    pub description: String,
    pub logo_url: Option<String>,
    pub homepage: String,
    pub docs_url: String,
    pub get_api_key_url: Option<String>,
    pub setup_guide_url: Option<String>,
    pub api_format: ApiFormat,
    pub base_url: Option<String>,
    pub base_url_template: Option<String>,
    pub requires_api_key: bool,
    pub onboarding: ProviderOnboarding,
    pub coding_plans: Vec<CodingPlanTemplate>,
    pub models: Vec<ModelTemplate>,
    pub supported_agents: Vec<AgentToolRef>,
    pub version: String,
}

/// Provider 新用户引导信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderOnboarding {
    pub description: String,
    pub signup_url: String,
    pub plans_comparison_url: Option<String>,
    pub get_key_url: Option<String>,
    pub setup_guide_url: Option<String>,
    pub faq_url: Option<String>,
    pub agent_setup_guides: Vec<AgentSetupGuide>,
}

/// Agent 工具配置指南
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSetupGuide {
    pub agent_id: String,
    pub agent_name: String,
    pub auto_config_supported: bool,
    pub auto_config_script: Option<String>,
    pub manual_steps: Vec<SetupStep>,
    pub config_file_paths: PlatformPaths,
    pub env_vars: Vec<EnvVarConfig>,
}

/// 配置步骤
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupStep {
    pub step_number: u32,
    pub description: String,
    pub command: Option<String>,
    pub copyable_text: Option<String>,
    pub note: Option<String>,
}

/// 环境变量配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvVarConfig {
    pub name: String,
    pub value: String,
    pub description: String,
}

/// 跨平台路径
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformPaths {
    pub macos: Option<String>,
    pub linux: Option<String>,
    pub windows: Option<String>,
}

// ============================================================================
// Coding Plan 模板
// ============================================================================

/// Coding Plan 模板
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodingPlanTemplate {
    pub plan_id: String,
    pub name: String,
    pub description: String,
    pub tier: PlanTier,
    pub supported_model_ids: Vec<String>,
    pub supported_agent_ids: Vec<String>,
    pub default_model_id: String,
    pub default_agent_id: String,
    pub quota_daily: Option<u64>,
    pub quota_monthly: Option<u64>,
    pub rpm_limit: Option<u32>,
    pub price: Option<String>,
    pub features: Vec<String>,
}

// ============================================================================
// 模型模板
// ============================================================================

/// 模型模板
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelTemplate {
    pub model_id: String,
    pub name: String,
    pub description: Option<String>,
    pub context_length: Option<u64>,
    pub capabilities: Vec<ModelCapability>,
    pub provider_id: String,
}

/// Agent 工具引用
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentToolRef {
    pub agent_id: String,
    pub name: String,
}

// ============================================================================
// Agent 工具定义
// ============================================================================

/// Agent 工具定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTool {
    pub agent_id: String,
    pub name: String,
    pub description: String,
    pub logo_url: Option<String>,
    pub homepage: String,
    pub install_url: String,
    pub supported_formats: Vec<ApiFormat>,
    pub config_methods: Vec<AgentConfigMethod>,
}

/// Agent 配置方式
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum AgentConfigMethod {
    #[serde(rename = "env_var")]
    EnvVar { name: String, value_template: String },
    #[serde(rename = "config_file")]
    ConfigFile { path_template: String, content_template: String },
    #[serde(rename = "cli_flag")]
    CliFlag { flag: String },
}

// ============================================================================
// 用户套餐实例（运行时配置单元）
// ============================================================================

/// 用户套餐实例
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPlan {
    pub id: String,
    pub provider_id: String,
    pub plan_id: String,
    pub name: String,
    pub api_key: String,
    pub selected_model_id: String,
    pub bound_agents: Vec<AgentBinding>,
    pub enabled: bool,
    pub priority: u32,
    pub custom_quota_daily: Option<u64>,
    pub custom_quota_monthly: Option<u64>,
    pub custom_rpm_limit: Option<u32>,
    pub alert_threshold: Option<f32>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_health_check: Option<DateTime<Utc>>,
    pub health_status: HealthStatus,
}

impl UserPlan {
    /// 创建新套餐
    pub fn new(
        id: String,
        provider_id: String,
        plan_id: String,
        name: String,
        api_key: String,
        selected_model_id: String,
    ) -> Self {
        Self {
            id,
            provider_id,
            plan_id,
            name,
            api_key,
            selected_model_id,
            bound_agents: Vec::new(),
            enabled: true,
            priority: 1,
            custom_quota_daily: None,
            custom_quota_monthly: None,
            custom_rpm_limit: None,
            alert_threshold: Some(0.8),
            notes: None,
            created_at: Utc::now(),
            last_health_check: None,
            health_status: HealthStatus::Unknown,
        }
    }
}

/// Agent 工具绑定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentBinding {
    pub agent_id: String,
    pub configured: bool,
    pub config_status: AgentConfigStatus,
    pub last_connected: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
}

// ============================================================================
// 运行时请求路由上下文
// ============================================================================

/// 请求上下文
#[derive(Debug, Clone)]
pub struct RequestContext {
    pub user_plan: UserPlan,
    pub agent_tool: Option<String>,
    pub endpoint_format: ApiFormat,
    pub needs_conversion: bool,
    pub target_format: ApiFormat,
}

// ============================================================================
// 配置文件结构
// ============================================================================

/// 用户套餐配置文件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPlansConfig {
    pub version: String,
    pub default_user_plan_id: Option<String>,
    pub user_plans: Vec<UserPlan>,
}

impl Default for UserPlansConfig {
    fn default() -> Self {
        Self {
            version: "2.0".to_string(),
            default_user_plan_id: None,
            user_plans: Vec::new(),
        }
    }
}

/// 内置 Provider 配置文件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvidersConfig {
    pub version: String,
    pub providers: Vec<ProviderTemplate>,
}

// ============================================================================
// Fallback 配置
// ============================================================================

/// Fallback 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackConfig {
    pub enabled: bool,
    pub max_attempts: u32,
    pub priority_order: Vec<String>,
}

impl Default for FallbackConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_attempts: 3,
            priority_order: Vec::new(),
        }
    }
}

// ============================================================================
// 自定义 Agent 工具
// ============================================================================

/// 自定义 Agent 工具
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomAgent {
    /// UUID
    pub id: String,
    /// 用户填写的 code/标识符
    pub agent_id: String,
    /// 工具名称
    pub name: String,
    /// 版本号
    pub version: String,
    /// 图标 URL
    pub logo_url: Option<String>,
    /// 描述
    pub description: Option<String>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

impl CustomAgent {
    /// 创建新的自定义 Agent
    pub fn new(
        id: String,
        agent_id: String,
        name: String,
        version: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            agent_id,
            name,
            version,
            logo_url: None,
            description: None,
            created_at: now,
            updated_at: now,
        }
    }
}

/// 自定义 Agent 配置文件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomAgentsConfig {
    pub version: String,
    pub custom_agents: Vec<CustomAgent>,
}

impl Default for CustomAgentsConfig {
    fn default() -> Self {
        Self {
            version: "1.0".to_string(),
            custom_agents: Vec::new(),
        }
    }
}

// ============================================================================
// 自定义 Provider
// ============================================================================

/// 自定义 Provider（用户添加的服务商）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomProvider {
    /// UUID
    pub id: String,
    /// 用户填写的 provider_id/标识符
    pub provider_id: String,
    /// 服务商名称
    pub name: String,
    /// 描述
    pub description: Option<String>,
    /// 图标 URL
    pub logo_url: Option<String>,
    /// 官网
    pub homepage: Option<String>,
    /// 文档 URL
    pub docs_url: Option<String>,
    /// 获取 API Key 的 URL
    pub get_api_key_url: Option<String>,
    /// API 格式
    pub api_format: ApiFormat,
    /// Base URL（API endpoint）
    pub base_url: String,
    /// 是否需要 API Key
    pub requires_api_key: bool,
    /// 模型列表
    pub models: Vec<CustomModel>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

impl CustomProvider {
    /// 创建新的自定义 Provider
    pub fn new(
        id: String,
        provider_id: String,
        name: String,
        api_format: ApiFormat,
        base_url: String,
        requires_api_key: bool,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            provider_id,
            name,
            description: None,
            logo_url: None,
            homepage: None,
            docs_url: None,
            get_api_key_url: None,
            api_format,
            base_url,
            requires_api_key,
            models: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }
}

/// 自定义模型（用于 CustomProvider）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomModel {
    /// 模型 ID
    pub model_id: String,
    /// 模型名称
    pub name: String,
    /// 描述
    pub description: Option<String>,
    /// 上下文长度
    pub context_length: Option<u64>,
    /// 能力标签
    pub capabilities: Vec<String>,
}

/// 自定义 Provider 配置文件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomProvidersConfig {
    pub version: String,
    pub custom_providers: Vec<CustomProvider>,
}

impl Default for CustomProvidersConfig {
    fn default() -> Self {
        Self {
            version: "1.0".to_string(),
            custom_providers: Vec::new(),
        }
    }
}