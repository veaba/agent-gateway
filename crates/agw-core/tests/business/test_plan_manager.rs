//! PlanManager integration tests

use std::sync::Arc;
use agw_core::business::PlanManager;
use agw_core::model::UserPlan;
use agw_core::storage::ConfigStore;

fn create_test_plan(id: &str, name: &str, provider_id: &str) -> UserPlan {
    UserPlan::new(
        id.to_string(),
        provider_id.to_string(),
        format!("{}-plan", provider_id),
        name.to_string(),
        format!("test-api-key-{}", id),
        "claude-sonnet-4-5".to_string(),
    )
}

#[tokio::test]
async fn test_plan_manager_load_all() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_store = Arc::new(ConfigStore::with_path(temp_dir.path().to_path_buf()).unwrap());
    let manager = PlanManager::new(config_store);

    let plans = manager.load_all().await.unwrap();
    assert!(plans.is_empty());
}

#[tokio::test]
async fn test_plan_manager_add_and_get() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_store = Arc::new(ConfigStore::with_path(temp_dir.path().to_path_buf()).unwrap());
    let manager = PlanManager::new(config_store);

    let plan = create_test_plan("plan-1", "Test Plan", "anthropic");
    manager.add(plan.clone()).await.unwrap();

    let retrieved = manager.get("plan-1").await;
    assert!(retrieved.is_some());
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.id, "plan-1");
    assert_eq!(retrieved.name, "Test Plan");
    assert_eq!(retrieved.provider_id, "anthropic");
}

#[tokio::test]
async fn test_plan_manager_update() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_store = Arc::new(ConfigStore::with_path(temp_dir.path().to_path_buf()).unwrap());
    let manager = PlanManager::new(config_store);

    let plan = create_test_plan("plan-1", "Test Plan", "anthropic");
    manager.add(plan).await.unwrap();

    let mut updated = create_test_plan("plan-1", "Updated Plan", "anthropic");
    updated.api_key = "new-api-key".to_string();
    manager.update(updated).await.unwrap();

    let retrieved = manager.get("plan-1").await.unwrap();
    assert_eq!(retrieved.name, "Updated Plan");
    assert_eq!(retrieved.api_key, "new-api-key");
}

#[tokio::test]
async fn test_plan_manager_delete() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_store = Arc::new(ConfigStore::with_path(temp_dir.path().to_path_buf()).unwrap());
    let manager = PlanManager::new(config_store);

    let plan = create_test_plan("plan-1", "Test Plan", "anthropic");
    manager.add(plan).await.unwrap();

    manager.delete("plan-1").await.unwrap();

    let retrieved = manager.get("plan-1").await;
    assert!(retrieved.is_none());
}

#[tokio::test]
async fn test_plan_manager_get_default_and_set_default() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_store = Arc::new(ConfigStore::with_path(temp_dir.path().to_path_buf()).unwrap());
    let manager = PlanManager::new(config_store);

    let plan = create_test_plan("plan-1", "Test Plan", "anthropic");
    manager.add(plan).await.unwrap();

    // Initially no default
    let default = manager.get_default().await;
    assert!(default.is_none());

    // Set default
    manager.set_default("plan-1").await.unwrap();

    let default = manager.get_default().await;
    assert!(default.is_some());
    assert_eq!(default.unwrap().id, "plan-1");
}

#[tokio::test]
async fn test_plan_manager_set_default_not_found() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_store = Arc::new(ConfigStore::with_path(temp_dir.path().to_path_buf()).unwrap());
    let manager = PlanManager::new(config_store);

    let result = manager.set_default("nonexistent").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_plan_manager_test_connection() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_store = Arc::new(ConfigStore::with_path(temp_dir.path().to_path_buf()).unwrap());
    let manager = PlanManager::new(config_store);

    let plan = create_test_plan("plan-1", "Test Plan", "anthropic");
    manager.add(plan).await.unwrap();

    let result = manager.test_connection("plan-1").await;
    assert!(result.is_ok());
    assert!(result.unwrap());
}

#[tokio::test]
async fn test_plan_manager_test_connection_not_found() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_store = Arc::new(ConfigStore::with_path(temp_dir.path().to_path_buf()).unwrap());
    let manager = PlanManager::new(config_store);

    let result = manager.test_connection("nonexistent").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_plan_manager_auto_config_agent() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_store = Arc::new(ConfigStore::with_path(temp_dir.path().to_path_buf()).unwrap());
    let manager = PlanManager::new(config_store);

    let mut plan = create_test_plan("plan-1", "Test Plan", "anthropic");
    plan.bound_agents.push(agw_core::model::AgentBinding {
        agent_id: "claude-code".to_string(),
        configured: false,
        config_status: agw_core::model_types::AgentConfigStatus::NotConfigured,
        last_connected: None,
        error_message: None,
    });
    manager.add(plan).await.unwrap();

    let result = manager.auto_config_agent("plan-1", "claude-code").await;
    // May succeed or fail depending on environment (shell rc file access)
    // but should not panic
    match result {
        Ok(configured) => {
            let plan = manager.get("plan-1").await.unwrap();
            let binding = plan.bound_agents.iter().find(|b| b.agent_id == "claude-code").unwrap();
            if configured {
                assert!(binding.configured);
                assert_eq!(binding.config_status, agw_core::model_types::AgentConfigStatus::AutoConfigured);
            } else {
                assert_eq!(binding.config_status, agw_core::model_types::AgentConfigStatus::ConfigError);
            }
        }
        Err(_) => {
            // Error is acceptable in test environment
        }
    }
}

#[tokio::test]
async fn test_plan_manager_bind_and_unbind_agent() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_store = Arc::new(ConfigStore::with_path(temp_dir.path().to_path_buf()).unwrap());
    let manager = PlanManager::new(config_store);

    let plan = create_test_plan("plan-1", "Test Plan", "anthropic");
    manager.add(plan).await.unwrap();

    manager.bind_agent("plan-1", "claude-code").await.unwrap();

    let plan = manager.get("plan-1").await.unwrap();
    assert_eq!(plan.bound_agents.len(), 1);
    assert_eq!(plan.bound_agents[0].agent_id, "claude-code");

    // Duplicate bind should fail
    let result = manager.bind_agent("plan-1", "claude-code").await;
    assert!(result.is_err());

    manager.unbind_agent("plan-1", "claude-code").await.unwrap();

    let plan = manager.get("plan-1").await.unwrap();
    assert!(plan.bound_agents.is_empty());

    // Unbind non-existent should fail
    let result = manager.unbind_agent("plan-1", "claude-code").await;
    assert!(result.is_err());
}
