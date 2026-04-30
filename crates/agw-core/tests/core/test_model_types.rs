//! ModelTypes tests

use agw_core::model_types::*;

// ============================================================================
// ApiFormat Tests
// ============================================================================

#[test]
fn test_api_format_default() {
    let format = ApiFormat::default();
    assert_eq!(format, ApiFormat::Anthropic);
}

#[test]
fn test_api_format_display() {
    assert_eq!(ApiFormat::Anthropic.to_string(), "anthropic");
    assert_eq!(ApiFormat::OpenAi.to_string(), "openai");
    assert_eq!(ApiFormat::Custom.to_string(), "custom");
}

#[test]
fn test_api_format_serde() {
    // Serialize
    let json = serde_json::to_string(&ApiFormat::Anthropic).unwrap();
    assert_eq!(json, "\"anthropic\"");

    let json = serde_json::to_string(&ApiFormat::OpenAi).unwrap();
    // snake_case converts OpenAi to open_ai
    assert_eq!(json, "\"open_ai\"");

    // Deserialize
    let format: ApiFormat = serde_json::from_str("\"anthropic\"").unwrap();
    assert_eq!(format, ApiFormat::Anthropic);

    let format: ApiFormat = serde_json::from_str("\"open_ai\"").unwrap();
    assert_eq!(format, ApiFormat::OpenAi);

    let format: ApiFormat = serde_json::from_str("\"custom\"").unwrap();
    assert_eq!(format, ApiFormat::Custom);
}

#[test]
fn test_api_format_equality() {
    assert_eq!(ApiFormat::Anthropic, ApiFormat::Anthropic);
    assert_ne!(ApiFormat::Anthropic, ApiFormat::OpenAi);
    assert_ne!(ApiFormat::OpenAi, ApiFormat::Custom);
}

// ============================================================================
// PlanTier Tests
// ============================================================================

#[test]
fn test_plan_tier_default() {
    let tier = PlanTier::default();
    assert_eq!(tier, PlanTier::Free);
}

#[test]
fn test_plan_tier_display() {
    assert_eq!(PlanTier::Free.to_string(), "free");
    assert_eq!(PlanTier::Pro.to_string(), "pro");
    assert_eq!(PlanTier::Enterprise.to_string(), "enterprise");
    assert_eq!(PlanTier::Custom.to_string(), "custom");
}

#[test]
fn test_plan_tier_serde() {
    let json = serde_json::to_string(&PlanTier::Pro).unwrap();
    assert_eq!(json, "\"pro\"");

    let tier: PlanTier = serde_json::from_str("\"enterprise\"").unwrap();
    assert_eq!(tier, PlanTier::Enterprise);
}

// ============================================================================
// ModelCapability Tests
// ============================================================================

#[test]
fn test_model_capability_display() {
    assert_eq!(ModelCapability::Code.to_string(), "code");
    assert_eq!(ModelCapability::Reasoning.to_string(), "reasoning");
    assert_eq!(ModelCapability::LongContext.to_string(), "long_context");
    assert_eq!(ModelCapability::ChineseOptimized.to_string(), "chinese_optimized");
    assert_eq!(ModelCapability::Math.to_string(), "math");
    assert_eq!(ModelCapability::Multimodal.to_string(), "multimodal");
}

#[test]
fn test_model_capability_equality() {
    assert_eq!(ModelCapability::Code, ModelCapability::Code);
    assert_ne!(ModelCapability::Code, ModelCapability::Reasoning);
}

// ============================================================================
// AgentConfigStatus Tests
// ============================================================================

#[test]
fn test_agent_config_status_default() {
    let status = AgentConfigStatus::default();
    assert_eq!(status, AgentConfigStatus::NotConfigured);
}

#[test]
fn test_agent_config_status_display() {
    assert_eq!(AgentConfigStatus::NotConfigured.to_string(), "not_configured");
    assert_eq!(AgentConfigStatus::AutoConfigured.to_string(), "auto_configured");
    assert_eq!(AgentConfigStatus::ManuallyConfigured.to_string(), "manually_configured");
    assert_eq!(AgentConfigStatus::ConfigError.to_string(), "config_error");
    assert_eq!(AgentConfigStatus::NeedsUpdate.to_string(), "needs_update");
}

#[test]
fn test_agent_config_status_serde() {
    let json = serde_json::to_string(&AgentConfigStatus::AutoConfigured).unwrap();
    assert_eq!(json, "\"auto_configured\"");

    let status: AgentConfigStatus = serde_json::from_str("\"config_error\"").unwrap();
    assert_eq!(status, AgentConfigStatus::ConfigError);
}

// ============================================================================
// HealthStatus Tests
// ============================================================================

#[test]
fn test_health_status_default() {
    let status = HealthStatus::default();
    assert_eq!(status, HealthStatus::Unknown);
}

#[test]
fn test_health_status_display() {
    assert_eq!(HealthStatus::Unknown.to_string(), "unknown");
    assert_eq!(HealthStatus::Healthy.to_string(), "healthy");
    assert_eq!(HealthStatus::Warning.to_string(), "warning");
    assert_eq!(HealthStatus::Error.to_string(), "error");
    assert_eq!(HealthStatus::Disabled.to_string(), "disabled");
}

#[test]
fn test_health_status_serde() {
    let json = serde_json::to_string(&HealthStatus::Healthy).unwrap();
    assert_eq!(json, "\"healthy\"");

    let status: HealthStatus = serde_json::from_str("\"error\"").unwrap();
    assert_eq!(status, HealthStatus::Error);

    let status: HealthStatus = serde_json::from_str("\"disabled\"").unwrap();
    assert_eq!(status, HealthStatus::Disabled);
}

#[test]
fn test_health_status_ordering() {
    // HealthStatus should be comparable
    assert!(HealthStatus::Healthy != HealthStatus::Error);
    assert!(HealthStatus::Warning != HealthStatus::Healthy);
}

// ============================================================================
// FallbackTrigger Tests
// ============================================================================

#[test]
fn test_fallback_trigger_display() {
    assert_eq!(FallbackTrigger::RateLimit.to_string(), "rate_limit");
    assert_eq!(FallbackTrigger::ServerError.to_string(), "server_error");
    assert_eq!(FallbackTrigger::ConnectionFailure.to_string(), "connection_failure");
    assert_eq!(FallbackTrigger::Timeout.to_string(), "timeout");
    assert_eq!(FallbackTrigger::QuotaExceeded.to_string(), "quota_exceeded");
}

#[test]
fn test_fallback_trigger_serde() {
    let json = serde_json::to_string(&FallbackTrigger::RateLimit).unwrap();
    assert_eq!(json, "\"rate_limit\"");

    let trigger: FallbackTrigger = serde_json::from_str("\"timeout\"").unwrap();
    assert_eq!(trigger, FallbackTrigger::Timeout);
}

// ============================================================================
// PluginStatus Tests
// ============================================================================

#[test]
fn test_plugin_status_default() {
    let status = PluginStatus::default();
    assert_eq!(status, PluginStatus::Installed);
}

#[test]
fn test_plugin_status_display() {
    assert_eq!(PluginStatus::Installed.to_string(), "installed");
    assert_eq!(PluginStatus::Enabled.to_string(), "enabled");
    assert_eq!(PluginStatus::Disabled.to_string(), "disabled");
    assert_eq!(PluginStatus::Error.to_string(), "error");
}

#[test]
fn test_plugin_status_serde() {
    let json = serde_json::to_string(&PluginStatus::Enabled).unwrap();
    assert_eq!(json, "\"enabled\"");

    let status: PluginStatus = serde_json::from_str("\"disabled\"").unwrap();
    assert_eq!(status, PluginStatus::Disabled);
}

// ============================================================================
// PluginType Tests
// ============================================================================

#[test]
fn test_plugin_type_default() {
    let plugin_type = PluginType::default();
    assert_eq!(plugin_type, PluginType::Provider);
}

#[test]
fn test_plugin_type_display() {
    assert_eq!(PluginType::Provider.to_string(), "provider");
    assert_eq!(PluginType::Transform.to_string(), "transform");
    assert_eq!(PluginType::Tool.to_string(), "tool");
}

#[test]
fn test_plugin_type_serde() {
    let json = serde_json::to_string(&PluginType::Transform).unwrap();
    assert_eq!(json, "\"transform\"");

    let plugin_type: PluginType = serde_json::from_str("\"tool\"").unwrap();
    assert_eq!(plugin_type, PluginType::Tool);
}