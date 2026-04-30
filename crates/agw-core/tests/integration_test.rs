//! agw-core integration tests.
//!
//! These tests verify that the shared test utilities compile and basic
//! model construction works end-to-end.

use agw_core::model_types::*;
use agw_core::test_utils::*;

#[test]
fn test_provider_template_fixture_compiles() {
    let provider = create_test_provider_template();
    assert_eq!(provider.provider_id, "test-provider");
    assert_eq!(provider.api_format, ApiFormat::OpenAi);
}

#[test]
fn test_anthropic_provider_template_fixture() {
    let provider = create_anthropic_provider_template();
    assert_eq!(provider.provider_id, "anthropic-test");
    assert_eq!(provider.api_format, ApiFormat::Anthropic);
}

#[test]
fn test_user_plan_fixture_compiles() {
    let plan = create_test_user_plan();
    assert_eq!(plan.id, "plan-001");
    assert_eq!(plan.provider_id, "test-provider");
    assert!(plan.enabled);
}

#[test]
fn test_request_context_fixture_compiles() {
    let ctx = create_test_request_context();
    assert_eq!(ctx.user_plan.id, "plan-001");
    assert_eq!(ctx.endpoint_format, ApiFormat::OpenAi);
    assert!(!ctx.needs_conversion);
}

#[test]
fn test_conversion_request_context() {
    let ctx = create_conversion_request_context();
    assert_eq!(ctx.user_plan.provider_id, "anthropic-test");
    assert!(ctx.needs_conversion);
    assert_eq!(ctx.endpoint_format, ApiFormat::Anthropic);
    assert_eq!(ctx.target_format, ApiFormat::OpenAi);
}

#[test]
fn test_json_parsing_helpers() {
    let openai_val = parse_json(SAMPLE_OPENAI_REQUEST);
    assert_eq!(openai_val["model"], "test-model");

    let anthropic_val = parse_json(SAMPLE_ANTHROPIC_REQUEST);
    assert_eq!(anthropic_val["model"], "claude-sonnet-4-5");
}

#[test]
fn test_fallback_config_fixture() {
    let config = create_test_fallback_config();
    assert!(config.enabled);
    assert_eq!(config.max_attempts, 3);
    assert_eq!(config.priority_order.len(), 2);
}
