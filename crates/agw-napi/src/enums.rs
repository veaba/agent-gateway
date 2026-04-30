use napi_derive::napi;

#[napi(string_enum)]
#[allow(dead_code)]
pub enum ApiFormat {
    Anthropic,
    OpenAi,
    Custom,
}

#[napi(string_enum)]
pub enum PlanTier {
    Free,
    Pro,
    Enterprise,
    Custom,
}

#[napi(string_enum)]
pub enum ModelCapability {
    Code,
    Reasoning,
    LongContext,
    ChineseOptimized,
    Math,
    Multimodal,
}

#[napi(string_enum)]
pub enum AgentConfigStatus {
    NotConfigured,
    AutoConfigured,
    ManuallyConfigured,
    ConfigError,
    NeedsUpdate,
}

#[napi(string_enum)]
pub enum HealthStatus {
    Unknown,
    Healthy,
    Warning,
    Error,
    Disabled,
}

#[napi(string_enum)]
#[allow(dead_code)]
pub enum FallbackTrigger {
    RateLimit,
    ServerError,
    ConnectionFailure,
    Timeout,
    QuotaExceeded,
}

#[napi(string_enum)]
#[allow(dead_code)]
pub enum PluginStatus {
    Installed,
    Enabled,
    Disabled,
    Error,
}

#[napi(string_enum)]
#[allow(dead_code)]
pub enum PluginType {
    Provider,
    Transform,
    Tool,
}

#[napi(string_enum)]
#[allow(dead_code)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl From<agw_core::ApiFormat> for ApiFormat {
    fn from(v: agw_core::ApiFormat) -> Self {
        match v {
            agw_core::ApiFormat::Anthropic => ApiFormat::Anthropic,
            agw_core::ApiFormat::OpenAi => ApiFormat::OpenAi,
            agw_core::ApiFormat::Custom => ApiFormat::Custom,
        }
    }
}

impl From<ApiFormat> for agw_core::ApiFormat {
    fn from(v: ApiFormat) -> Self {
        match v {
            ApiFormat::Anthropic => agw_core::ApiFormat::Anthropic,
            ApiFormat::OpenAi => agw_core::ApiFormat::OpenAi,
            ApiFormat::Custom => agw_core::ApiFormat::Custom,
        }
    }
}

impl From<agw_core::PlanTier> for PlanTier {
    fn from(v: agw_core::PlanTier) -> Self {
        match v {
            agw_core::PlanTier::Free => PlanTier::Free,
            agw_core::PlanTier::Pro => PlanTier::Pro,
            agw_core::PlanTier::Enterprise => PlanTier::Enterprise,
            agw_core::PlanTier::Custom => PlanTier::Custom,
        }
    }
}

impl From<agw_core::ModelCapability> for ModelCapability {
    fn from(v: agw_core::ModelCapability) -> Self {
        match v {
            agw_core::ModelCapability::Code => ModelCapability::Code,
            agw_core::ModelCapability::Reasoning => ModelCapability::Reasoning,
            agw_core::ModelCapability::LongContext => ModelCapability::LongContext,
            agw_core::ModelCapability::ChineseOptimized => ModelCapability::ChineseOptimized,
            agw_core::ModelCapability::Math => ModelCapability::Math,
            agw_core::ModelCapability::Multimodal => ModelCapability::Multimodal,
        }
    }
}

impl From<agw_core::AgentConfigStatus> for AgentConfigStatus {
    fn from(v: agw_core::AgentConfigStatus) -> Self {
        match v {
            agw_core::AgentConfigStatus::NotConfigured => AgentConfigStatus::NotConfigured,
            agw_core::AgentConfigStatus::AutoConfigured => AgentConfigStatus::AutoConfigured,
            agw_core::AgentConfigStatus::ManuallyConfigured => AgentConfigStatus::ManuallyConfigured,
            agw_core::AgentConfigStatus::ConfigError => AgentConfigStatus::ConfigError,
            agw_core::AgentConfigStatus::NeedsUpdate => AgentConfigStatus::NeedsUpdate,
        }
    }
}

impl From<agw_core::HealthStatus> for HealthStatus {
    fn from(v: agw_core::HealthStatus) -> Self {
        match v {
            agw_core::HealthStatus::Unknown => HealthStatus::Unknown,
            agw_core::HealthStatus::Healthy => HealthStatus::Healthy,
            agw_core::HealthStatus::Warning => HealthStatus::Warning,
            agw_core::HealthStatus::Error => HealthStatus::Error,
            agw_core::HealthStatus::Disabled => HealthStatus::Disabled,
        }
    }
}

impl From<agw_core::PluginStatus> for PluginStatus {
    fn from(v: agw_core::PluginStatus) -> Self {
        match v {
            agw_core::PluginStatus::Installed => PluginStatus::Installed,
            agw_core::PluginStatus::Enabled => PluginStatus::Enabled,
            agw_core::PluginStatus::Disabled => PluginStatus::Disabled,
            agw_core::PluginStatus::Error => PluginStatus::Error,
        }
    }
}

impl From<agw_core::PluginType> for PluginType {
    fn from(v: agw_core::PluginType) -> Self {
        match v {
            agw_core::PluginType::Provider => PluginType::Provider,
            agw_core::PluginType::Transform => PluginType::Transform,
            agw_core::PluginType::Tool => PluginType::Tool,
        }
    }
}