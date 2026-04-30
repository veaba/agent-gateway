//! ApiKeyHelper integration tests

use agw_core::security::ApiKeyHelper;
use agw_core::model::ProviderTemplate;
use agw_core::model_types::ApiFormat;

fn make_provider(id: &str) -> ProviderTemplate {
    ProviderTemplate {
        provider_id: id.to_string(),
        name: id.to_string(),
        description: "".to_string(),
        logo_url: None,
        homepage: "".to_string(),
        docs_url: "".to_string(),
        get_api_key_url: None,
        setup_guide_url: None,
        api_format: ApiFormat::Anthropic,
        base_url: None,
        base_url_template: None,
        requires_api_key: true,
        onboarding: agw_core::model::ProviderOnboarding {
            description: "".to_string(),
            signup_url: "".to_string(),
            plans_comparison_url: None,
            get_key_url: None,
            setup_guide_url: None,
            faq_url: None,
            agent_setup_guides: vec![],
        },
        coding_plans: vec![],
        models: vec![],
        supported_agents: vec![],
        version: "0.1.0".to_string(),
    }
}

#[test]
fn test_validate_key_format_anthropic_sk() {
    let provider = make_provider("anthropic");
    assert!(ApiKeyHelper::validate_key_format(&provider, "sk-abcdefghijklmnopqrstuvwxyz1234567890").is_ok());
}

#[test]
fn test_validate_key_format_anthropic_sk_ant() {
    let provider = make_provider("anthropic");
    assert!(ApiKeyHelper::validate_key_format(&provider, "sk-ant-abcdefghijklmnopqrstuvwxyz1234567890").is_ok());
}

#[test]
fn test_validate_key_format_openai() {
    let provider = make_provider("openai");
    assert!(ApiKeyHelper::validate_key_format(&provider, "sk-abcdefghijklmnopqrstuvwxyz1234567890").is_ok());
}

#[test]
fn test_validate_key_format_kimi() {
    let provider = make_provider("kimi");
    assert!(ApiKeyHelper::validate_key_format(&provider, "sk-abcdefghijklmnopqrstuvwxyz1234567890").is_ok());
}

#[test]
fn test_validate_key_format_too_short() {
    let provider = make_provider("openai");
    let result = ApiKeyHelper::validate_key_format(&provider, "sk-short");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("too short"));
}

#[test]
fn test_validate_key_format_wrong_prefix_anthropic() {
    let provider = make_provider("anthropic");
    let result = ApiKeyHelper::validate_key_format(&provider, "wrong-prefix-abcdefghijklmnopqrstuvwxyz1234567890");
    assert!(result.is_err());
}

#[test]
fn test_validate_key_format_wrong_prefix_openai() {
    let provider = make_provider("openai");
    // openai accepts sk- prefix, sk-ant- also starts with sk- so it passes
    // Test with a truly wrong prefix
    let result = ApiKeyHelper::validate_key_format(&provider, "wrong-prefix-abcdefghijklmnopqrstuvwxyz1234567890");
    assert!(result.is_err());
}

#[test]
fn test_is_likely_api_key_sk() {
    assert!(ApiKeyHelper::is_likely_api_key("sk-abcdefghijklmnopqrstuvwxyz1234567890"));
}

#[test]
fn test_is_likely_api_key_sk_ant() {
    assert!(ApiKeyHelper::is_likely_api_key("sk-ant-abcdefghijklmnopqrstuvwxyz1234567890"));
}

#[test]
fn test_is_likely_api_key_sk_proj() {
    assert!(ApiKeyHelper::is_likely_api_key("sk-proj-abcdefghijklmnopqrstuvwxyz1234567890"));
}

#[test]
fn test_is_likely_api_key_aiza() {
    assert!(ApiKeyHelper::is_likely_api_key("AIzaabcdefghijklmnopqrstuvwxyz1234567890"));
}

#[test]
fn test_is_likely_api_key_gsk() {
    assert!(ApiKeyHelper::is_likely_api_key("gsk_abcdefghijklmnopqrstuvwxyz1234567890"));
}

#[test]
fn test_is_likely_api_key_kilo() {
    assert!(ApiKeyHelper::is_likely_api_key("kilo_abcdefghijklmnopqrstuvwxyz1234567890"));
}

#[test]
fn test_is_likely_api_key_too_short() {
    assert!(!ApiKeyHelper::is_likely_api_key("sk-short"));
}

#[test]
fn test_is_likely_api_key_invalid_prefix() {
    assert!(!ApiKeyHelper::is_likely_api_key("not-a-key-abcdefghijklmnopqrstuvwxyz1234567890"));
}

#[test]
fn test_is_likely_api_key_empty() {
    assert!(!ApiKeyHelper::is_likely_api_key(""));
}

#[test]
fn test_is_likely_api_key_whitespace_only() {
    assert!(!ApiKeyHelper::is_likely_api_key("   "));
}

#[test]
fn test_is_likely_api_key_with_whitespace() {
    assert!(ApiKeyHelper::is_likely_api_key("  sk-abcdefghijklmnopqrstuvwxyz1234567890  "));
}

#[test]
fn test_validate_key_format_trims_whitespace() {
    let provider = make_provider("openai");
    assert!(ApiKeyHelper::validate_key_format(&provider, "  sk-abcdefghijklmnopqrstuvwxyz1234567890  ").is_ok());
}

#[test]
fn test_validate_key_format_unknown_provider_allows_any() {
    let provider = make_provider("custom-provider");
    assert!(ApiKeyHelper::validate_key_format(&provider, "any-key-abcdefghijklmnopqrstuvwxyz1234567890").is_ok());
}
