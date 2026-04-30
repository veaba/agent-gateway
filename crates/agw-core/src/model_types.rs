//! 枚举类型定义

use serde::{Deserialize, Serialize};

// ============================================================================
// API 格式
// ============================================================================

/// API 格式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApiFormat {
    /// Anthropic Messages API
    Anthropic,
    /// OpenAI Chat Completions API
    OpenAi,
    /// 自定义格式
    Custom,
}

impl Default for ApiFormat {
    fn default() -> Self {
        Self::Anthropic
    }
}

impl std::fmt::Display for ApiFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiFormat::Anthropic => write!(f, "anthropic"),
            ApiFormat::OpenAi => write!(f, "openai"),
            ApiFormat::Custom => write!(f, "custom"),
        }
    }
}

// ============================================================================
// 套餐等级
// ============================================================================

/// 套餐等级
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PlanTier {
    Free,
    Pro,
    Enterprise,
    Custom,
}

impl Default for PlanTier {
    fn default() -> Self {
        Self::Free
    }
}

// ============================================================================
// 模型能力
// ============================================================================

/// 模型能力
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelCapability {
    /// 代码生成
    Code,
    /// 推理
    Reasoning,
    /// 长上下文
    LongContext,
    /// 中文优化
    ChineseOptimized,
    /// 数学
    Math,
    /// 多模态
    Multimodal,
}

// ============================================================================
// Agent 配置状态
// ============================================================================

/// Agent 配置状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentConfigStatus {
    /// 未配置
    NotConfigured,
    /// 已自动配置
    AutoConfigured,
    /// 已手动配置
    ManuallyConfigured,
    /// 配置出错
    ConfigError,
    /// 需要更新
    NeedsUpdate,
}

impl Default for AgentConfigStatus {
    fn default() -> Self {
        Self::NotConfigured
    }
}

// ============================================================================
// 健康状态
// ============================================================================

/// 健康状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthStatus {
    Unknown,
    Healthy,
    Warning,
    Error,
    Disabled,
}

impl Default for HealthStatus {
    fn default() -> Self {
        Self::Unknown
    }
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthStatus::Unknown => write!(f, "unknown"),
            HealthStatus::Healthy => write!(f, "healthy"),
            HealthStatus::Warning => write!(f, "warning"),
            HealthStatus::Error => write!(f, "error"),
            HealthStatus::Disabled => write!(f, "disabled"),
        }
    }
}

// ============================================================================
// Fallback 触发条件
// ============================================================================

/// Fallback 触发条件
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FallbackTrigger {
    /// 429 Rate Limit
    RateLimit,
    /// 5xx 服务端错误
    ServerError,
    /// 连接失败
    ConnectionFailure,
    /// 超时
    Timeout,
    /// 配额用尽
    QuotaExceeded,
}

impl std::fmt::Display for FallbackTrigger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FallbackTrigger::RateLimit => write!(f, "rate_limit"),
            FallbackTrigger::ServerError => write!(f, "server_error"),
            FallbackTrigger::ConnectionFailure => write!(f, "connection_failure"),
            FallbackTrigger::Timeout => write!(f, "timeout"),
            FallbackTrigger::QuotaExceeded => write!(f, "quota_exceeded"),
        }
    }
}

// ============================================================================
// 插件状态
// ============================================================================

/// 插件状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginStatus {
    Installed,
    Enabled,
    Disabled,
    Error,
}

impl Default for PluginStatus {
    fn default() -> Self {
        Self::Installed
    }
}

// ============================================================================
// 插件类型
// ============================================================================

/// 插件类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginType {
    /// Provider 插件（扩展新的 AI 服务商）
    Provider,
    /// 转换插件（自定义协议转换）
    Transform,
    /// 工具插件（扩展功能）
    Tool,
}

impl Default for PluginType {
    fn default() -> Self {
        Self::Provider
    }
}