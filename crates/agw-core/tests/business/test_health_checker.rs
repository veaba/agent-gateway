//! HealthChecker tests

use std::sync::Arc;
use agw_core::business::{HealthChecker, PlanManager, ProviderEngine};
use agw_core::model::{UserPlan, ProviderTemplate, ProviderOnboarding, AgentToolRef};
use agw_core::model_types::{ApiFormat, HealthStatus, PlanTier, ModelCapability};
use agw_core::storage::{ConfigStore, SqliteStore};

fn create_test_provider(provider_id: &str, api_format: ApiFormat, base_url: Option<&str>) -> ProviderTemplate {
    ProviderTemplate {
        provider_id: provider_id.to_string(),
        name: format!("{} Provider", provider_id),
        description: "Test provider".to_string(),
        logo_url: None,
        homepage: "https://example.com".to_string(),
        docs_url: "https://docs.example.com".to_string(),
        get_api_key_url: None,
        setup_guide_url: None,
        api_format,
        base_url: base_url.map(|s| s.to_string()),
        base_url_template: None,
        requires_api_key: true,
        onboarding: ProviderOnboarding {
            description: "Test".to_string(),
            signup_url: "https://example.com".to_string(),
            plans_comparison_url: None,
            get_key_url: None,
            setup_guide_url: None,
            faq_url: None,
            agent_setup_guides: vec![],
        },
        coding_plans: vec![agw_core::model::CodingPlanTemplate {
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
        }],
        models: vec![agw_core::model::ModelTemplate {
            model_id: "test-model".to_string(),
            name: "Test Model".to_string(),
            description: Some("A model used in tests.".to_string()),
            context_length: Some(128_000),
            capabilities: vec![ModelCapability::Code, ModelCapability::Reasoning],
            provider_id: provider_id.to_string(),
        }],
        supported_agents: vec![AgentToolRef {
            agent_id: "claude-code".to_string(),
            name: "Claude Code".to_string(),
        }],
        version: "0.1.0".to_string(),
    }
}

fn create_test_plan(id: &str, provider_id: &str, api_key: &str, enabled: bool) -> UserPlan {
    let mut plan = UserPlan::new(
        id.to_string(),
        provider_id.to_string(),
        format!("{}-plan", provider_id),
        format!("Plan {}", id),
        api_key.to_string(),
        "claude-sonnet-4-5".to_string(),
    );
    plan.enabled = enabled;
    plan.health_status = HealthStatus::Unknown;
    plan
}

#[test]
fn test_health_check_result_status() {
    // Test that HealthCheckResult can be created with different statuses
    use agw_core::business::health_checker::HealthCheckResult;

    let result = HealthCheckResult {
        status: HealthStatus::Healthy,
        response_time_ms: 100,
        error_message: None,
    };
    assert_eq!(result.status, HealthStatus::Healthy);
    assert_eq!(result.response_time_ms, 100);

    let result = HealthCheckResult {
        status: HealthStatus::Error,
        response_time_ms: 5000,
        error_message: Some("Connection timeout".to_string()),
    };
    assert_eq!(result.status, HealthStatus::Error);
    assert!(result.error_message.is_some());
}

#[test]
fn test_classify_status_from_code() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_store = Arc::new(ConfigStore::with_path(temp_dir.path().to_path_buf()).unwrap());
    let sqlite = Arc::new(SqliteStore::in_memory().unwrap());
    let provider_engine = Arc::new(ProviderEngine::new());
    let plan_manager = Arc::new(PlanManager::new(config_store));

    let checker = HealthChecker::new(provider_engine, sqlite, plan_manager);

    // 2xx - Healthy
    assert_eq!(checker.classify_status_from_code(200), HealthStatus::Healthy);
    assert_eq!(checker.classify_status_from_code(201), HealthStatus::Healthy);
    assert_eq!(checker.classify_status_from_code(204), HealthStatus::Healthy);
    assert_eq!(checker.classify_status_from_code(299), HealthStatus::Healthy);

    // 429 - Warning (rate limited but reachable)
    assert_eq!(checker.classify_status_from_code(429), HealthStatus::Warning);

    // 5xx - Error
    assert_eq!(checker.classify_status_from_code(500), HealthStatus::Error);
    assert_eq!(checker.classify_status_from_code(502), HealthStatus::Error);
    assert_eq!(checker.classify_status_from_code(503), HealthStatus::Error);

    // 4xx - Warning (auth issues, etc)
    assert_eq!(checker.classify_status_from_code(401), HealthStatus::Warning);
    assert_eq!(checker.classify_status_from_code(403), HealthStatus::Warning);
    assert_eq!(checker.classify_status_from_code(404), HealthStatus::Warning);

    // Other - Unknown
    assert_eq!(checker.classify_status_from_code(100), HealthStatus::Unknown);
    assert_eq!(checker.classify_status_from_code(301), HealthStatus::Unknown);
}

#[test]
fn test_classify_status_boundary() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_store = Arc::new(ConfigStore::with_path(temp_dir.path().to_path_buf()).unwrap());
    let sqlite = Arc::new(SqliteStore::in_memory().unwrap());
    let provider_engine = Arc::new(ProviderEngine::new());
    let plan_manager = Arc::new(PlanManager::new(config_store));

    let checker = HealthChecker::new(provider_engine, sqlite, plan_manager);

    // Boundary tests
    // Just before 2xx
    assert_eq!(checker.classify_status_from_code(199), HealthStatus::Unknown);
    // Start of 2xx
    assert_eq!(checker.classify_status_from_code(200), HealthStatus::Healthy);
    // End of 2xx
    assert_eq!(checker.classify_status_from_code(299), HealthStatus::Healthy);
    // Start of 4xx (non-429)
    assert_eq!(checker.classify_status_from_code(400), HealthStatus::Warning);
    // Just before 5xx
    assert_eq!(checker.classify_status_from_code(499), HealthStatus::Warning);
    // Start of 5xx
    assert_eq!(checker.classify_status_from_code(500), HealthStatus::Error);
}

#[tokio::test]
async fn test_health_checker_check_plan_not_found() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_store = Arc::new(ConfigStore::with_path(temp_dir.path().to_path_buf()).unwrap());
    let sqlite = Arc::new(SqliteStore::in_memory().unwrap());
    let provider_engine = Arc::new(ProviderEngine::new());
    let plan_manager = Arc::new(PlanManager::new(config_store));

    let checker = HealthChecker::new(provider_engine, sqlite, plan_manager);

    // Plan not found should return error
    let result = checker.check_plan("nonexistent").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_health_checker_check_disabled_plan() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_store = Arc::new(ConfigStore::with_path(temp_dir.path().to_path_buf()).unwrap());
    let sqlite = Arc::new(SqliteStore::in_memory().unwrap());
    let provider_engine = Arc::new(ProviderEngine::new());
    let plan_manager = Arc::new(PlanManager::new(config_store.clone()));

    // Add disabled plan
    let plan = create_test_plan("disabled-plan", "anthropic", "test-key", false);
    plan_manager.add(plan).await.unwrap();

    // Add provider using add_custom
    let provider = create_test_provider("anthropic", ApiFormat::Anthropic, Some("https://api.anthropic.com"));
    provider_engine.add_custom(provider);

    let checker = HealthChecker::new(provider_engine, sqlite, plan_manager);

    // Disabled plan should return Disabled status
    let result = checker.check_plan("disabled-plan").await.unwrap();
    assert_eq!(result.status, HealthStatus::Disabled);
    assert_eq!(result.response_time_ms, 0);
    assert!(result.error_message.is_some());
    assert!(result.error_message.unwrap().contains("disabled"));
}