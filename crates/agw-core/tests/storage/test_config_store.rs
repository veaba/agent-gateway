//! ConfigStore integration tests

use std::path::PathBuf;
use agw_core::storage::ConfigStore;
use agw_core::model::{UserPlansConfig, FallbackConfig, UserPlan};

fn temp_config_dir() -> PathBuf {
    tempfile::tempdir().unwrap().into_path()
}

#[test]
fn test_config_store_with_path() {
    let dir = temp_config_dir();
    let store = ConfigStore::with_path(dir.clone()).unwrap();
    assert_eq!(store.config_dir(), &dir);
    assert!(dir.exists());
}

#[test]
fn test_config_store_data_dir() {
    let dir = temp_config_dir();
    let store = ConfigStore::with_path(dir).unwrap();
    let data_dir = store.data_dir();
    assert!(data_dir.to_string_lossy().contains("agent-gateway"));
}

#[tokio::test]
async fn test_config_store_init_data_dir() {
    let dir = temp_config_dir();
    let store = ConfigStore::with_path(dir).unwrap();
    store.init_data_dir().await.unwrap();

    let data_dir = store.data_dir();
    assert!(data_dir.join("logs").exists());
    assert!(data_dir.join("plugins").exists());
}

#[tokio::test]
async fn test_load_save_user_plans_round_trip() {
    let dir = temp_config_dir();
    let store = ConfigStore::with_path(dir).unwrap();

    // Load empty config
    let empty = store.load_user_plans().await.unwrap();
    assert!(empty.user_plans.is_empty());

    // Create and save a config
    let mut config = UserPlansConfig::default();
    let plan = UserPlan::new(
        "plan-1".to_string(),
        "anthropic".to_string(),
        "pro".to_string(),
        "My Plan".to_string(),
        "sk-test-key".to_string(),
        "claude-sonnet".to_string(),
    );
    config.user_plans.push(plan);
    config.default_user_plan_id = Some("plan-1".to_string());

    store.save_user_plans(&config).await.unwrap();

    // Load and verify
    let loaded = store.load_user_plans().await.unwrap();
    assert_eq!(loaded.user_plans.len(), 1);
    assert_eq!(loaded.user_plans[0].id, "plan-1");
    assert_eq!(loaded.default_user_plan_id, Some("plan-1".to_string()));
}

#[tokio::test]
async fn test_set_default_plan() {
    let dir = temp_config_dir();
    let store = ConfigStore::with_path(dir).unwrap();

    // Create a config with a plan
    let mut config = UserPlansConfig::default();
    let plan = UserPlan::new(
        "plan-1".to_string(),
        "anthropic".to_string(),
        "pro".to_string(),
        "My Plan".to_string(),
        "sk-test-key".to_string(),
        "claude-sonnet".to_string(),
    );
    config.user_plans.push(plan);
    store.save_user_plans(&config).await.unwrap();

    // Set default plan
    store.set_default_plan("plan-1").await.unwrap();

    let loaded = store.load_user_plans().await.unwrap();
    assert_eq!(loaded.default_user_plan_id, Some("plan-1".to_string()));
}

#[tokio::test]
async fn test_load_providers_empty() {
    let dir = temp_config_dir();
    let store = ConfigStore::with_path(dir).unwrap();

    let providers = store.load_providers().await.unwrap();
    assert_eq!(providers.version, "0.1.0");
    assert!(providers.providers.is_empty());
}

#[tokio::test]
async fn test_load_save_fallback_config() {
    let dir = temp_config_dir();
    let store = ConfigStore::with_path(dir).unwrap();

    // Load default when file doesn't exist
    let default = store.load_fallback_config().await.unwrap();
    assert!(default.enabled);
    assert_eq!(default.max_attempts, 3);

    // Save custom config
    let config = FallbackConfig {
        enabled: false,
        max_attempts: 5,
        priority_order: vec!["plan-a".to_string(), "plan-b".to_string()],
    };
    store.save_fallback_config(&config).await.unwrap();

    // Load and verify
    let loaded = store.load_fallback_config().await.unwrap();
    assert!(!loaded.enabled);
    assert_eq!(loaded.max_attempts, 5);
    assert_eq!(loaded.priority_order, vec!["plan-a", "plan-b"]);
}
