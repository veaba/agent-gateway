//! Shared test utilities and fixtures for integration tests.
//!
//! This module provides helper functions to create common test data instances
//! such as `ProviderTemplate`, `UserPlan`, and `RequestContext`, along with
//! sample payloads for protocol conversion tests.

use crate::model::*;
use crate::model_types::*;
use chrono::Utc;

// ============================================================================
// ProviderTemplate fixtures
// ============================================================================

/// Create a minimal `ProviderTemplate` suitable for unit/integration tests.
pub fn create_test_provider_template() -> ProviderTemplate {
    ProviderTemplate {
        provider_id: "test-provider".to_string(),
        name: "Test Provider".to_string(),
        description: "A provider used in tests.".to_string(),
        logo_url: None,
        homepage: "https://example.com".to_string(),
        docs_url: "https://docs.example.com".to_string(),
        get_api_key_url: Some("https://example.com/key".to_string()),
        setup_guide_url: None,
        api_format: ApiFormat::OpenAi,
        base_url: Some("https://api.example.com/v1".to_string()),
        base_url_template: None,
        requires_api_key: true,
        onboarding: ProviderOnboarding {
            description: "Sign up at example.com".to_string(),
            signup_url: "https://example.com/signup".to_string(),
            plans_comparison_url: None,
            get_key_url: None,
            setup_guide_url: None,
            faq_url: None,
            agent_setup_guides: vec![],
        },
        coding_plans: vec![create_test_coding_plan_template()],
        models: vec![create_test_model_template()],
        supported_agents: vec![AgentToolRef {
            agent_id: "claude-code".to_string(),
            name: "Claude Code".to_string(),
        }],
        version: "1.0.0".to_string(),
    }
}

/// Create an Anthropic-format provider for protocol conversion tests.
pub fn create_anthropic_provider_template() -> ProviderTemplate {
    ProviderTemplate {
        provider_id: "anthropic-test".to_string(),
        name: "Anthropic Test".to_string(),
        description: "Anthropic provider for tests.".to_string(),
        logo_url: None,
        homepage: "https://anthropic.com".to_string(),
        docs_url: "https://docs.anthropic.com".to_string(),
        get_api_key_url: Some("https://anthropic.com/key".to_string()),
        setup_guide_url: None,
        api_format: ApiFormat::Anthropic,
        base_url: Some("https://api.anthropic.com".to_string()),
        base_url_template: None,
        requires_api_key: true,
        onboarding: ProviderOnboarding {
            description: "Sign up at anthropic.com".to_string(),
            signup_url: "https://anthropic.com/signup".to_string(),
            plans_comparison_url: None,
            get_key_url: None,
            setup_guide_url: None,
            faq_url: None,
            agent_setup_guides: vec![],
        },
        coding_plans: vec![create_test_coding_plan_template()],
        models: vec![create_test_model_template()],
        supported_agents: vec![AgentToolRef {
            agent_id: "claude-code".to_string(),
            name: "Claude Code".to_string(),
        }],
        version: "1.0.0".to_string(),
    }
}

// ============================================================================
// CodingPlanTemplate fixture
// ============================================================================

/// Create a minimal `CodingPlanTemplate` for test providers.
pub fn create_test_coding_plan_template() -> CodingPlanTemplate {
    CodingPlanTemplate {
        plan_id: "test-plan".to_string(),
        name: "Test Plan".to_string(),
        description: "A plan used in tests.".to_string(),
        tier: PlanTier::Pro,
        supported_model_ids: vec!["test-model".to_string()],
        supported_agent_ids: vec!["claude-code".to_string()],
        default_model_id: "test-model".to_string(),
        default_agent_id: "claude-code".to_string(),
        quota_daily: Some(1000),
        quota_monthly: Some(10000),
        rpm_limit: Some(60),
        price: Some("$10/month".to_string()),
        features: vec!["code".to_string(), "reasoning".to_string()],
    }
}

// ============================================================================
// ModelTemplate fixture
// ============================================================================

/// Create a minimal `ModelTemplate` for test providers.
pub fn create_test_model_template() -> ModelTemplate {
    ModelTemplate {
        model_id: "test-model".to_string(),
        name: "Test Model".to_string(),
        description: Some("A model used in tests.".to_string()),
        context_length: Some(128_000),
        capabilities: vec![ModelCapability::Code, ModelCapability::Reasoning],
        provider_id: "test-provider".to_string(),
    }
}

// ============================================================================
// UserPlan fixtures
// ============================================================================

/// Create a minimal `UserPlan` suitable for unit/integration tests.
pub fn create_test_user_plan() -> UserPlan {
    UserPlan::new(
        "plan-001".to_string(),
        "test-provider".to_string(),
        "test-plan".to_string(),
        "My Test Plan".to_string(),
        "sk-test-1234567890".to_string(),
        "test-model".to_string(),
    )
}

/// Create a `UserPlan` with Anthropic-style API key.
pub fn create_anthropic_test_user_plan() -> UserPlan {
    UserPlan::new(
        "plan-002".to_string(),
        "anthropic-test".to_string(),
        "test-plan".to_string(),
        "Anthropic Test Plan".to_string(),
        "sk-ant-test-1234567890".to_string(),
        "claude-sonnet-4-5".to_string(),
    )
}

// ============================================================================
// RequestContext fixtures
// ============================================================================

/// Create a `RequestContext` for OpenAI-target tests.
pub fn create_test_request_context() -> RequestContext {
    RequestContext {
        user_plan: create_test_user_plan(),
        agent_tool: Some("claude-code".to_string()),
        endpoint_format: ApiFormat::OpenAi,
        needs_conversion: false,
        target_format: ApiFormat::OpenAi,
    }
}

/// Create a `RequestContext` that requires Anthropic -> OpenAI conversion.
pub fn create_conversion_request_context() -> RequestContext {
    RequestContext {
        user_plan: create_anthropic_test_user_plan(),
        agent_tool: Some("claude-code".to_string()),
        endpoint_format: ApiFormat::Anthropic,
        needs_conversion: true,
        target_format: ApiFormat::OpenAi,
    }
}

// ============================================================================
// Common test data for protocol conversion
// ============================================================================

/// Sample OpenAI Chat Completions request body (JSON string).
pub const SAMPLE_OPENAI_REQUEST: &str = r#"{
    "model": "test-model",
    "messages": [
        {"role": "system", "content": "You are a helpful assistant."},
        {"role": "user", "content": "Hello!"}
    ],
    "temperature": 0.7,
    "max_tokens": 256
}"#;

/// Sample Anthropic Messages request body (JSON string).
pub const SAMPLE_ANTHROPIC_REQUEST: &str = r#"{
    "model": "claude-sonnet-4-5",
    "max_tokens": 256,
    "messages": [
        {"role": "user", "content": "Hello!"}
    ],
    "system": "You are a helpful assistant."
}"#;

/// Sample OpenAI Chat Completions streaming response chunk.
pub const SAMPLE_OPENAI_STREAM_CHUNK: &str = r#"{
    "id": "chatcmpl-test",
    "object": "chat.completion.chunk",
    "created": 1714521600,
    "model": "test-model",
    "choices": [
        {"index": 0, "delta": {"role": "assistant", "content": "Hi"}, "finish_reason": null}
    ]
}"#;

/// Sample Anthropic Messages streaming response chunk.
pub const SAMPLE_ANTHROPIC_STREAM_CHUNK: &str = r#"{
    "type": "content_block_delta",
    "index": 0,
    "delta": {"type": "text_delta", "text": "Hi"}
}"#;

/// Sample OpenAI Chat Completions non-streaming response.
pub const SAMPLE_OPENAI_RESPONSE: &str = r#"{
    "id": "chatcmpl-test",
    "object": "chat.completion",
    "created": 1714521600,
    "model": "test-model",
    "choices": [
        {
            "index": 0,
            "message": {"role": "assistant", "content": "Hello! How can I help you today?"},
            "finish_reason": "stop"
        }
    ],
    "usage": {"prompt_tokens": 10, "completion_tokens": 20, "total_tokens": 30}
}"#;

/// Sample Anthropic Messages non-streaming response.
pub const SAMPLE_ANTHROPIC_RESPONSE: &str = r#"{
    "id": "msg_01TestAnthropic",
    "type": "message",
    "role": "assistant",
    "model": "claude-sonnet-4-5",
    "content": [
        {"type": "text", "text": "Hello! How can I help you today?"}
    ],
    "stop_reason": "end_turn",
    "usage": {"input_tokens": 10, "output_tokens": 20}
}"#;

/// Helper to parse a JSON string into a `serde_json::Value` for assertions.
pub fn parse_json(json_str: &str) -> serde_json::Value {
    serde_json::from_str(json_str).expect("valid JSON")
}

/// Helper to build a minimal `FallbackConfig` for tests.
pub fn create_test_fallback_config() -> FallbackConfig {
    FallbackConfig {
        enabled: true,
        max_attempts: 3,
        priority_order: vec!["plan-001".to_string(), "plan-002".to_string()],
    }
}

/// Helper to build a minimal `AgentBinding` for tests.
pub fn create_test_agent_binding(agent_id: &str) -> AgentBinding {
    AgentBinding {
        agent_id: agent_id.to_string(),
        configured: true,
        config_status: AgentConfigStatus::AutoConfigured,
        last_connected: Some(Utc::now()),
        error_message: None,
    }
}
