//! FallbackEngine integration tests

use std::sync::Arc;
use agw_core::business::{FallbackEngine, FallbackReason};
use agw_core::FallbackConfig;
use agw_core::model::UserPlan;
use agw_core::model_types::HealthStatus;
use agw_core::storage::ConfigStore;

fn create_test_plan(id: &str, name: &str, enabled: bool, health: HealthStatus) -> UserPlan {
    let mut plan = UserPlan::new(
        id.to_string(),
        "anthropic".to_string(),
        "anthropic-default".to_string(),
        name.to_string(),
        format!("test-api-key-{}", id),
        "claude-sonnet-4-5".to_string(),
    );
    plan.enabled = enabled;
    plan.health_status = health;
    plan
}

#[test]
fn test_fallback_engine_should_fallback_when_enabled() {
    let engine = FallbackEngine::new();

    assert!(engine.should_fallback(&FallbackReason::RateLimit));
    assert!(engine.should_fallback(&FallbackReason::ServerError("500".to_string())));
    assert!(engine.should_fallback(&FallbackReason::ConnectionFailure));
    assert!(engine.should_fallback(&FallbackReason::Timeout));
    assert!(engine.should_fallback(&FallbackReason::QuotaExceeded));
}

#[test]
fn test_fallback_engine_should_not_fallback_when_disabled() {
    let mut engine = FallbackEngine::new();
    engine.set_enabled(false);

    assert!(!engine.should_fallback(&FallbackReason::RateLimit));
    assert!(!engine.should_fallback(&FallbackReason::ServerError("500".to_string())));
    assert!(!engine.should_fallback(&FallbackReason::ConnectionFailure));
    assert!(!engine.should_fallback(&FallbackReason::Timeout));
    assert!(!engine.should_fallback(&FallbackReason::QuotaExceeded));
}

#[test]
fn test_fallback_engine_set_enabled() {
    let mut engine = FallbackEngine::new();
    assert!(engine.should_fallback(&FallbackReason::RateLimit));

    engine.set_enabled(false);
    assert!(!engine.should_fallback(&FallbackReason::RateLimit));

    engine.set_enabled(true);
    assert!(engine.should_fallback(&FallbackReason::RateLimit));
}

#[test]
fn test_fallback_engine_set_priority() {
    let mut engine = FallbackEngine::new();
    let priority = vec!["plan-b".to_string(), "plan-c".to_string()];
    engine.set_priority(priority.clone());

    let config = engine.get_config();
    assert_eq!(config.priority_order, priority);
}

#[test]
fn test_fallback_engine_max_attempts() {
    let engine = FallbackEngine::new();
    assert_eq!(engine.max_attempts(), 3);

    let config = FallbackConfig {
        enabled: true,
        max_attempts: 5,
        priority_order: vec![],
    };
    let engine = FallbackEngine::with_config(config);
    assert_eq!(engine.max_attempts(), 5);
}

#[tokio::test]
async fn test_fallback_engine_find_alternative_disabled() {
    let engine = FallbackEngine::new();
    let result = engine.find_alternative("plan-a").await;
    // No priority order set, so no alternative
    assert!(result.is_none());
}

#[tokio::test]
async fn test_fallback_engine_find_alternative_with_priority() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_store = Arc::new(ConfigStore::with_path(temp_dir.path().to_path_buf()).unwrap());
    let plan_manager = Arc::new(agw_core::business::PlanManager::new(config_store));

    let plan_a = create_test_plan("plan-a", "Plan A", true, HealthStatus::Healthy);
    let plan_b = create_test_plan("plan-b", "Plan B", true, HealthStatus::Healthy);
    let plan_c = create_test_plan("plan-c", "Plan C", true, HealthStatus::Healthy);

    plan_manager.add(plan_a).await.unwrap();
    plan_manager.add(plan_b).await.unwrap();
    plan_manager.add(plan_c).await.unwrap();

    let mut config = FallbackConfig::default();
    config.priority_order = vec!["plan-b".to_string(), "plan-c".to_string()];

    let engine = FallbackEngine::with_dependencies(config, plan_manager, Arc::new(agw_core::business::QuotaTracker::new()));

    let alternative = engine.find_alternative("plan-a").await;
    assert!(alternative.is_some());
    assert_eq!(alternative.unwrap(), "plan-b");
}

#[tokio::test]
async fn test_fallback_engine_find_alternative_skips_disabled() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_store = Arc::new(ConfigStore::with_path(temp_dir.path().to_path_buf()).unwrap());
    let plan_manager = Arc::new(agw_core::business::PlanManager::new(config_store));

    let plan_a = create_test_plan("plan-a", "Plan A", true, HealthStatus::Healthy);
    let plan_b = create_test_plan("plan-b", "Plan B", false, HealthStatus::Healthy);
    let plan_c = create_test_plan("plan-c", "Plan C", true, HealthStatus::Healthy);

    plan_manager.add(plan_a).await.unwrap();
    plan_manager.add(plan_b).await.unwrap();
    plan_manager.add(plan_c).await.unwrap();

    let mut config = FallbackConfig::default();
    config.priority_order = vec!["plan-b".to_string(), "plan-c".to_string()];

    let engine = FallbackEngine::with_dependencies(config, plan_manager, Arc::new(agw_core::business::QuotaTracker::new()));

    let alternative = engine.find_alternative("plan-a").await;
    assert!(alternative.is_some());
    // plan-b is disabled, should skip to plan-c
    assert_eq!(alternative.unwrap(), "plan-c");
}

#[tokio::test]
async fn test_fallback_engine_find_alternative_skips_error_health() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_store = Arc::new(ConfigStore::with_path(temp_dir.path().to_path_buf()).unwrap());
    let plan_manager = Arc::new(agw_core::business::PlanManager::new(config_store));

    let plan_a = create_test_plan("plan-a", "Plan A", true, HealthStatus::Healthy);
    let plan_b = create_test_plan("plan-b", "Plan B", true, HealthStatus::Error);
    let plan_c = create_test_plan("plan-c", "Plan C", true, HealthStatus::Healthy);

    plan_manager.add(plan_a).await.unwrap();
    plan_manager.add(plan_b).await.unwrap();
    plan_manager.add(plan_c).await.unwrap();

    let mut config = FallbackConfig::default();
    config.priority_order = vec!["plan-b".to_string(), "plan-c".to_string()];

    let engine = FallbackEngine::with_dependencies(config, plan_manager, Arc::new(agw_core::business::QuotaTracker::new()));

    let alternative = engine.find_alternative("plan-a").await;
    assert!(alternative.is_some());
    // plan-b has Error health status, should skip to plan-c
    assert_eq!(alternative.unwrap(), "plan-c");
}

#[tokio::test]
async fn test_fallback_engine_find_alternative_skips_disabled_health() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_store = Arc::new(ConfigStore::with_path(temp_dir.path().to_path_buf()).unwrap());
    let plan_manager = Arc::new(agw_core::business::PlanManager::new(config_store));

    let plan_a = create_test_plan("plan-a", "Plan A", true, HealthStatus::Healthy);
    let plan_b = create_test_plan("plan-b", "Plan B", true, HealthStatus::Disabled);
    let plan_c = create_test_plan("plan-c", "Plan C", true, HealthStatus::Healthy);

    plan_manager.add(plan_a).await.unwrap();
    plan_manager.add(plan_b).await.unwrap();
    plan_manager.add(plan_c).await.unwrap();

    let mut config = FallbackConfig::default();
    config.priority_order = vec!["plan-b".to_string(), "plan-c".to_string()];

    let engine = FallbackEngine::with_dependencies(config, plan_manager, Arc::new(agw_core::business::QuotaTracker::new()));

    let alternative = engine.find_alternative("plan-a").await;
    assert!(alternative.is_some());
    // plan-b has Disabled health status, should skip to plan-c
    assert_eq!(alternative.unwrap(), "plan-c");
}

#[tokio::test]
async fn test_fallback_engine_find_alternative_skips_quota_exceeded() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_store = Arc::new(ConfigStore::with_path(temp_dir.path().to_path_buf()).unwrap());
    let plan_manager = Arc::new(agw_core::business::PlanManager::new(config_store));
    let quota_tracker = Arc::new(agw_core::business::QuotaTracker::new());

    let plan_a = create_test_plan("plan-a", "Plan A", true, HealthStatus::Healthy);
    let plan_b = create_test_plan("plan-b", "Plan B", true, HealthStatus::Healthy);
    let plan_c = create_test_plan("plan-c", "Plan C", true, HealthStatus::Healthy);

    plan_manager.add(plan_a).await.unwrap();
    plan_manager.add(plan_b).await.unwrap();
    plan_manager.add(plan_c).await.unwrap();

    // Set quota for plan-b to exceeded
    quota_tracker.set_limits("plan-b", agw_core::business::quota::QuotaLimit {
        daily: Some(5),
        monthly: Some(100),
        rpm: None,
    }).await;

    for _ in 0..5 {
        quota_tracker.check_and_consume("plan-b").await;
    }

    let mut config = FallbackConfig::default();
    config.priority_order = vec!["plan-b".to_string(), "plan-c".to_string()];

    let engine = FallbackEngine::with_dependencies(config, plan_manager, quota_tracker);

    let alternative = engine.find_alternative("plan-a").await;
    assert!(alternative.is_some());
    // plan-b quota exceeded, should skip to plan-c
    assert_eq!(alternative.unwrap(), "plan-c");
}

#[tokio::test]
async fn test_fallback_engine_find_alternative_no_available() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_store = Arc::new(ConfigStore::with_path(temp_dir.path().to_path_buf()).unwrap());
    let plan_manager = Arc::new(agw_core::business::PlanManager::new(config_store));

    let plan_a = create_test_plan("plan-a", "Plan A", true, HealthStatus::Healthy);
    let plan_b = create_test_plan("plan-b", "Plan B", false, HealthStatus::Healthy);

    plan_manager.add(plan_a).await.unwrap();
    plan_manager.add(plan_b).await.unwrap();

    let mut config = FallbackConfig::default();
    config.priority_order = vec!["plan-b".to_string()];

    let engine = FallbackEngine::with_dependencies(config, plan_manager, Arc::new(agw_core::business::QuotaTracker::new()));

    let alternative = engine.find_alternative("plan-a").await;
    assert!(alternative.is_none());
}

#[tokio::test]
async fn test_fallback_engine_find_alternative_skips_self() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_store = Arc::new(ConfigStore::with_path(temp_dir.path().to_path_buf()).unwrap());
    let plan_manager = Arc::new(agw_core::business::PlanManager::new(config_store));

    let plan_a = create_test_plan("plan-a", "Plan A", true, HealthStatus::Healthy);
    let plan_b = create_test_plan("plan-b", "Plan B", true, HealthStatus::Healthy);

    plan_manager.add(plan_a).await.unwrap();
    plan_manager.add(plan_b).await.unwrap();

    let mut config = FallbackConfig::default();
    config.priority_order = vec!["plan-a".to_string(), "plan-b".to_string()];

    let engine = FallbackEngine::with_dependencies(config, plan_manager, Arc::new(agw_core::business::QuotaTracker::new()));

    let alternative = engine.find_alternative("plan-a").await;
    assert!(alternative.is_some());
    // Should skip plan-a (self) and return plan-b
    assert_eq!(alternative.unwrap(), "plan-b");
}
