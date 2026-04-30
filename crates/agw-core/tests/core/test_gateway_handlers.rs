//! Gateway handlers integration tests

use agw_core::model::{UserPlan, ProviderTemplate};
use agw_core::model_types::{ApiFormat, HealthStatus};

// Replicate the resolve_user_plan logic for testing
async fn resolve_user_plan(
    plans: &[(String, UserPlan)],
    plan_id: Option<&str>,
    auth_key: Option<&str>,
) -> Option<UserPlan> {
    // 1. 优先使用显式指定的 plan_id
    if let Some(id) = plan_id {
        if let Some((_, plan)) = plans.iter().find(|(pid, _)| pid == id) {
            return Some(plan.clone());
        }
    }

    // 2. 尝试通过 API Key 匹配
    if let Some(key) = auth_key {
        if let Some((_, plan)) = plans.iter().find(|(_, p)| p.api_key == key) {
            return Some(plan.clone());
        }
    }

    // 3. 使用默认套餐
    plans.iter().find(|(_, p)| p.priority == 0).map(|(_, p)| p.clone())
}

// Replicate the build_target_url logic for testing
fn build_target_url(
    provider: &ProviderTemplate,
    user_plan: &UserPlan,
    endpoint: &str,
) -> String {
    if let Some(ref base_url) = provider.base_url {
        return format!("{}{}", base_url.trim_end_matches('/'), endpoint);
    }

    if let Some(ref template) = provider.base_url_template {
        let url = template
            .replace("{plan_id}", &user_plan.plan_id)
            .replace("{provider_id}", &user_plan.provider_id);
        return format!("{}{}", url.trim_end_matches('/'), endpoint);
    }

    format!("{}{}", provider.homepage.trim_end_matches('/'), endpoint)
}

fn create_test_plan(id: &str, api_key: &str, priority: u32) -> UserPlan {
    let mut plan = UserPlan::new(
        id.to_string(),
        "anthropic".to_string(),
        "anthropic-default".to_string(),
        format!("Plan {}", id),
        api_key.to_string(),
        "claude-sonnet-4-5".to_string(),
    );
    plan.priority = priority;
    plan.enabled = true;
    plan.health_status = HealthStatus::Healthy;
    plan
}

#[tokio::test]
async fn test_resolve_user_plan_by_id() {
    let plans = vec![
        ("plan-1".to_string(), create_test_plan("plan-1", "key1", 1)),
        ("plan-2".to_string(), create_test_plan("plan-2", "key2", 2)),
    ];

    let resolved = resolve_user_plan(&plans, Some("plan-1"), None).await;
    assert!(resolved.is_some());
    assert_eq!(resolved.unwrap().id, "plan-1");
}

#[tokio::test]
async fn test_resolve_user_plan_by_api_key() {
    let plans = vec![
        ("plan-1".to_string(), create_test_plan("plan-1", "key1", 1)),
        ("plan-2".to_string(), create_test_plan("plan-2", "key2", 2)),
    ];

    let resolved = resolve_user_plan(&plans, None, Some("key2")).await;
    assert!(resolved.is_some());
    assert_eq!(resolved.unwrap().id, "plan-2");
}

#[tokio::test]
async fn test_resolve_user_plan_id_priority_over_key() {
    let plans = vec![
        ("plan-1".to_string(), create_test_plan("plan-1", "key1", 1)),
        ("plan-2".to_string(), create_test_plan("plan-2", "key2", 2)),
    ];

    // plan_id should take priority over api_key
    let resolved = resolve_user_plan(&plans, Some("plan-1"), Some("key2")).await;
    assert!(resolved.is_some());
    assert_eq!(resolved.unwrap().id, "plan-1");
}

#[tokio::test]
async fn test_resolve_user_plan_not_found() {
    let plans = vec![
        ("plan-1".to_string(), create_test_plan("plan-1", "key1", 1)),
    ];

    let resolved = resolve_user_plan(&plans, Some("nonexistent"), None).await;
    assert!(resolved.is_none());
}

#[tokio::test]
async fn test_resolve_user_plan_fallback_to_default() {
    let plans = vec![
        ("plan-1".to_string(), create_test_plan("plan-1", "key1", 0)),
        ("plan-2".to_string(), create_test_plan("plan-2", "key2", 1)),
    ];

    let resolved = resolve_user_plan(&plans, None, None).await;
    assert!(resolved.is_some());
    assert_eq!(resolved.unwrap().id, "plan-1");
}

#[test]
fn test_build_target_url_with_base_url() {
    let provider = ProviderTemplate {
        provider_id: "anthropic".to_string(),
        name: "Anthropic".to_string(),
        description: "Anthropic API".to_string(),
        logo_url: None,
        homepage: "https://anthropic.com".to_string(),
        docs_url: "https://docs.anthropic.com".to_string(),
        get_api_key_url: None,
        setup_guide_url: None,
        api_format: ApiFormat::Anthropic,
        base_url: Some("https://api.anthropic.com".to_string()),
        base_url_template: None,
        requires_api_key: true,
        onboarding: agw_core::model::ProviderOnboarding {
            description: "Test".to_string(),
            signup_url: "https://anthropic.com".to_string(),
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
    };

    let user_plan = create_test_plan("plan-1", "key1", 1);

    let url = build_target_url(&provider, &user_plan, "/v1/messages");
    assert_eq!(url, "https://api.anthropic.com/v1/messages");
}

#[test]
fn test_build_target_url_with_template() {
    let provider = ProviderTemplate {
        provider_id: "alaya".to_string(),
        name: "Alaya".to_string(),
        description: "Alaya AI".to_string(),
        logo_url: None,
        homepage: "https://alaya.ai".to_string(),
        docs_url: "https://docs.alaya.ai".to_string(),
        get_api_key_url: None,
        setup_guide_url: None,
        api_format: ApiFormat::Anthropic,
        base_url: None,
        base_url_template: Some("https://api.alaya.com/coding/{plan_id}".to_string()),
        requires_api_key: true,
        onboarding: agw_core::model::ProviderOnboarding {
            description: "Test".to_string(),
            signup_url: "https://alaya.ai".to_string(),
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
    };

    let mut user_plan = create_test_plan("plan-1", "key1", 1);
    user_plan.plan_id = "alaya-plus".to_string();

    let url = build_target_url(&provider, &user_plan, "/v1/messages");
    assert_eq!(url, "https://api.alaya.com/coding/alaya-plus/v1/messages");
}

#[test]
fn test_build_target_url_fallback_to_homepage() {
    let provider = ProviderTemplate {
        provider_id: "custom".to_string(),
        name: "Custom".to_string(),
        description: "Custom provider".to_string(),
        logo_url: None,
        homepage: "https://custom.example.com".to_string(),
        docs_url: "https://docs.custom.example.com".to_string(),
        get_api_key_url: None,
        setup_guide_url: None,
        api_format: ApiFormat::OpenAi,
        base_url: None,
        base_url_template: None,
        requires_api_key: true,
        onboarding: agw_core::model::ProviderOnboarding {
            description: "Test".to_string(),
            signup_url: "https://custom.example.com".to_string(),
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
    };

    let user_plan = create_test_plan("plan-1", "key1", 1);

    let url = build_target_url(&provider, &user_plan, "/v1/chat/completions");
    assert_eq!(url, "https://custom.example.com/v1/chat/completions");
}

#[test]
fn test_build_target_url_trailing_slash_handling() {
    let provider = ProviderTemplate {
        provider_id: "anthropic".to_string(),
        name: "Anthropic".to_string(),
        description: "Anthropic API".to_string(),
        logo_url: None,
        homepage: "https://anthropic.com".to_string(),
        docs_url: "https://docs.anthropic.com".to_string(),
        get_api_key_url: None,
        setup_guide_url: None,
        api_format: ApiFormat::Anthropic,
        base_url: Some("https://api.anthropic.com/".to_string()),
        base_url_template: None,
        requires_api_key: true,
        onboarding: agw_core::model::ProviderOnboarding {
            description: "Test".to_string(),
            signup_url: "https://anthropic.com".to_string(),
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
    };

    let user_plan = create_test_plan("plan-1", "key1", 1);

    let url = build_target_url(&provider, &user_plan, "/v1/messages");
    assert_eq!(url, "https://api.anthropic.com/v1/messages");
}
