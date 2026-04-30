//! ProviderEngine integration tests

use agw_core::business::ProviderEngine;
use agw_core::model::ProviderTemplate;
use agw_core::model_types::ApiFormat;

#[tokio::test]
async fn test_provider_engine_load_builtin_providers() {
    let engine = ProviderEngine::new();
    let providers = engine.list_providers().await;
    // Should have at least alaya and anthropic builtins
    assert!(!providers.is_empty());
    let ids: Vec<String> = providers.iter().map(|p| p.provider_id.clone()).collect();
    assert!(ids.contains(&"alaya".to_string()));
    assert!(ids.contains(&"anthropic".to_string()));
}

#[tokio::test]
async fn test_provider_engine_list_providers() {
    let engine = ProviderEngine::new();
    let providers = engine.list_providers().await;
    assert!(!providers.is_empty());
}

#[tokio::test]
async fn test_provider_engine_get_provider_builtin() {
    let engine = ProviderEngine::new();

    let alaya = engine.get_provider("alaya").await;
    assert!(alaya.is_some());
    let alaya = alaya.unwrap();
    assert_eq!(alaya.provider_id, "alaya");
    assert_eq!(alaya.name, "Alaya");

    let anthropic = engine.get_provider("anthropic").await;
    assert!(anthropic.is_some());
    let anthropic = anthropic.unwrap();
    assert_eq!(anthropic.provider_id, "anthropic");
    assert_eq!(anthropic.name, "Anthropic");
}

#[tokio::test]
async fn test_provider_engine_get_provider_not_found() {
    let engine = ProviderEngine::new();
    let provider = engine.get_provider("nonexistent").await;
    assert!(provider.is_none());
}

#[tokio::test]
async fn test_provider_engine_get_plan_template() {
    let engine = ProviderEngine::new();

    let plan = engine.get_plan_template("alaya", "alaya-lite").await;
    assert!(plan.is_some());
    let plan = plan.unwrap();
    assert_eq!(plan.plan_id, "alaya-lite");
    assert_eq!(plan.name, "Lite");

    let not_found = engine.get_plan_template("alaya", "nonexistent").await;
    assert!(not_found.is_none());
}

#[tokio::test]
async fn test_provider_engine_get_model_template() {
    let engine = ProviderEngine::new();

    let model = engine.get_model_template("alaya", "glm-5").await;
    assert!(model.is_some());
    let model = model.unwrap();
    assert_eq!(model.model_id, "glm-5");
    assert_eq!(model.name, "GLM-5");

    let not_found = engine.get_model_template("alaya", "nonexistent").await;
    assert!(not_found.is_none());
}

#[tokio::test]
async fn test_provider_engine_add_custom() {
    let engine = ProviderEngine::new();

    let custom = ProviderTemplate {
        provider_id: "custom-provider".to_string(),
        name: "Custom Provider".to_string(),
        description: "A custom provider".to_string(),
        logo_url: None,
        homepage: "https://custom.example.com".to_string(),
        docs_url: "https://docs.custom.example.com".to_string(),
        get_api_key_url: None,
        setup_guide_url: None,
        api_format: ApiFormat::OpenAi,
        base_url: Some("https://api.custom.example.com".to_string()),
        base_url_template: None,
        requires_api_key: true,
        onboarding: agw_core::model::ProviderOnboarding {
            description: "Custom provider onboarding".to_string(),
            signup_url: "https://custom.example.com/signup".to_string(),
            plans_comparison_url: None,
            get_key_url: None,
            setup_guide_url: None,
            faq_url: None,
            agent_setup_guides: vec![],
        },
        coding_plans: vec![],
        models: vec![],
        supported_agents: vec![],
        version: "1.0.0".to_string(),
    };

    engine.add_custom(custom.clone());

    let retrieved = engine.get_provider("custom-provider").await;
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().name, "Custom Provider");

    let all = engine.list_providers().await;
    let ids: Vec<String> = all.iter().map(|p| p.provider_id.clone()).collect();
    assert!(ids.contains(&"custom-provider".to_string()));
}

#[tokio::test]
async fn test_provider_engine_remove_custom() {
    let engine = ProviderEngine::new();

    let custom = ProviderTemplate {
        provider_id: "custom-provider".to_string(),
        name: "Custom Provider".to_string(),
        description: "A custom provider".to_string(),
        logo_url: None,
        homepage: "https://custom.example.com".to_string(),
        docs_url: "https://docs.custom.example.com".to_string(),
        get_api_key_url: None,
        setup_guide_url: None,
        api_format: ApiFormat::OpenAi,
        base_url: Some("https://api.custom.example.com".to_string()),
        base_url_template: None,
        requires_api_key: true,
        onboarding: agw_core::model::ProviderOnboarding {
            description: "Custom provider onboarding".to_string(),
            signup_url: "https://custom.example.com/signup".to_string(),
            plans_comparison_url: None,
            get_key_url: None,
            setup_guide_url: None,
            faq_url: None,
            agent_setup_guides: vec![],
        },
        coding_plans: vec![],
        models: vec![],
        supported_agents: vec![],
        version: "1.0.0".to_string(),
    };

    engine.add_custom(custom);
    assert!(engine.get_provider("custom-provider").await.is_some());

    let removed = engine.remove_custom("custom-provider");
    assert!(removed);

    assert!(engine.get_provider("custom-provider").await.is_none());

    // Removing again should return false
    let removed_again = engine.remove_custom("custom-provider");
    assert!(!removed_again);
}

#[tokio::test]
async fn test_provider_engine_remove_builtin_not_allowed() {
    let engine = ProviderEngine::new();

    // Cannot remove builtin providers via remove_custom
    let removed = engine.remove_custom("alaya");
    assert!(!removed);

    // alaya should still exist
    assert!(engine.get_provider("alaya").await.is_some());
}
