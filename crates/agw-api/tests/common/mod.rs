//! Shared test utilities for agw-api integration tests.

use std::sync::Arc;
use tokio::sync::RwLock;

use agw_api::AppState;
use agw_api::types::ApiConfig;
use agw_core::{
    business::{PlanManager, ProviderEngine, QuotaTracker, HealthChecker, CustomAgentManager, CustomProviderManager},
    model::FallbackConfig,
    plugin::PluginLifecycle,
    storage::{ConfigStore, RequestLogStore, SqliteStore},
};

/// Create an AppState backed by a temporary directory and in-memory SQLite.
pub async fn setup_test_state() -> (AppState, tempfile::TempDir) {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let config_dir = temp_dir.path().join("config");
    let data_dir = temp_dir.path().join("data");

    // Ensure directories exist
    tokio::fs::create_dir_all(&config_dir).await.unwrap();
    tokio::fs::create_dir_all(&data_dir).await.unwrap();
    tokio::fs::create_dir_all(data_dir.join("logs")).await.unwrap();
    tokio::fs::create_dir_all(data_dir.join("plugins")).await.unwrap();

    let config_store = Arc::new(ConfigStore::with_path(config_dir).unwrap());

    // Use temp data dir for logs and plugins to ensure test isolation
    let log_dir = data_dir.join("logs");
    let log_store = Arc::new(RequestLogStore::new(log_dir));

    let sqlite_store = Arc::new(SqliteStore::in_memory().unwrap());

    let plan_manager = Arc::new(PlanManager::new(Arc::clone(&config_store)));
    let quota_tracker = Arc::new(QuotaTracker::with_sqlite(sqlite_store.clone()));

    let provider_engine = Arc::new(ProviderEngine::new());

    let fallback_config = Arc::new(RwLock::new(FallbackConfig::default()));

    let plugin_lifecycle = Arc::new(PluginLifecycle::new());
    let plugin_registry = Arc::new(plugin_lifecycle.registry().clone());

    let health_checker = Arc::new(HealthChecker::new(
        provider_engine.clone(),
        sqlite_store.clone(),
        plan_manager.clone(),
    ));

    // Create custom agent and provider managers
    let custom_agent_manager = Arc::new(CustomAgentManager::new(Arc::clone(&config_store)));
    let custom_provider_manager = Arc::new(CustomProviderManager::new(Arc::clone(&config_store)));

    let state = AppState {
        config_store,
        plan_manager,
        provider_engine,
        quota_tracker,
        fallback_config,
        plugin_registry,
        plugin_lifecycle,
        log_store,
        sqlite_store,
        health_checker,
        custom_agent_manager,
        custom_provider_manager,
        api_config: ApiConfig::default(),
    };

    (state, temp_dir)
}
