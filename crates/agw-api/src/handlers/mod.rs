//! API 处理器模块

mod health;
mod plan;
mod provider;
mod quota;
mod fallback;
mod plugin;
mod logs;
mod agent;
mod stats;
mod apikey;
mod config;
mod log_detail;
mod custom_agent;
mod custom_provider;

pub use self::{
    health::health,
    plan::{list_plans, get_plan, create_plan, update_plan, delete_plan, test_plan, set_default_plan},
    provider::{list_providers, get_provider, update_providers},
    quota::{quota_status, set_quota},
    fallback::{get_fallback, update_fallback, get_fallback_events, get_fallback_stats, get_fallback_performance},
    plugin::{list_plugins, install_plugin, uninstall_plugin, enable_plugin, disable_plugin, get_plugin, update_plugin},
    logs::get_logs,
    agent::{list_agents, bind_agent, unbind_agent, auto_config_agent},
    stats::{get_global_stats, get_provider_stats, get_plan_stats, get_usage_trend, get_plan_health, trigger_health_check},
    apikey::{test_api_key, get_api_key, update_api_key},
    config::{get_config, update_config, export_config, import_config, reset_config},
    log_detail::{get_log_by_id, export_logs, get_log_files},
    custom_agent::{list_custom_agents, create_custom_agent, get_custom_agent, update_custom_agent, delete_custom_agent},
    custom_provider::{list_custom_providers, create_custom_provider, get_custom_provider, update_custom_provider, delete_custom_provider},
};

use axum::{
    routing::{get, post, put, delete},
    Router,
};
use crate::state::AppState;

/// 创建 API 路由
pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Health check
        .route("/health", get(health))
        // Plans
        .route("/api/v1/plans", get(list_plans))
        .route("/api/v1/plans", post(create_plan))
        .route("/api/v1/plans/:id", get(get_plan))
        .route("/api/v1/plans/:id", put(update_plan))
        .route("/api/v1/plans/:id", delete(delete_plan))
        .route("/api/v1/plans/:id/test", post(test_plan))
        .route("/api/v1/plans/:id/default", post(set_default_plan))
        // Providers
        .route("/api/v1/providers", get(list_providers))
        .route("/api/v1/providers/:id", get(get_provider))
        .route("/api/v1/providers/update", post(update_providers))
        // Quota
        .route("/api/v1/quota", get(quota_status))
        .route("/api/v1/quota/:plan_id", put(set_quota))
        // Fallback
        .route("/api/v1/fallback", get(get_fallback))
        .route("/api/v1/fallback", put(update_fallback))
        .route("/api/v1/fallback/events", get(get_fallback_events))
        .route("/api/v1/fallback/stats", get(get_fallback_stats))
        .route("/api/v1/fallback/performance", get(get_fallback_performance))
        // Plugins
        .route("/api/v1/plugins", get(list_plugins))
        .route("/api/v1/plugins/install", post(install_plugin))
        .route("/api/v1/plugins/:id", get(get_plugin))
        .route("/api/v1/plugins/:id", delete(uninstall_plugin))
        .route("/api/v1/plugins/:id/update", post(update_plugin))
        .route("/api/v1/plugins/:id/enable", post(enable_plugin))
        .route("/api/v1/plugins/:id/disable", post(disable_plugin))
        // Logs
        .route("/api/v1/logs", get(get_logs))
        .route("/api/v1/logs/:id", get(get_log_by_id))
        .route("/api/v1/logs/export", get(export_logs))
        .route("/api/v1/logs/files", get(get_log_files))
        // Agents
        .route("/api/v1/agents", get(list_agents))
        .route("/api/v1/plans/:id/agents/:agent_id/bind", post(bind_agent))
        .route("/api/v1/plans/:id/agents/:agent_id/unbind", delete(unbind_agent))
        .route("/api/v1/plans/:id/agents/:agent_id/auto-config", post(auto_config_agent))
        // Custom Agents
        .route("/api/v1/custom-agents", get(list_custom_agents))
        .route("/api/v1/custom-agents", post(create_custom_agent))
        .route("/api/v1/custom-agents/:id", get(get_custom_agent))
        .route("/api/v1/custom-agents/:id", put(update_custom_agent))
        .route("/api/v1/custom-agents/:id", delete(delete_custom_agent))
        // Custom Providers
        .route("/api/v1/custom-providers", get(list_custom_providers))
        .route("/api/v1/custom-providers", post(create_custom_provider))
        .route("/api/v1/custom-providers/:id", get(get_custom_provider))
        .route("/api/v1/custom-providers/:id", put(update_custom_provider))
        .route("/api/v1/custom-providers/:id", delete(delete_custom_provider))
        // Stats
        .route("/api/v1/stats", get(get_global_stats))
        .route("/api/v1/stats/providers", get(get_provider_stats))
        .route("/api/v1/stats/usage", get(get_usage_trend))
        .route("/api/v1/stats/:plan_id", get(get_plan_stats))
        .route("/api/v1/health/:plan_id", get(get_plan_health))
        .route("/api/v1/health/:plan_id/check", post(trigger_health_check))
        // API Key
        .route("/api/v1/plans/:id/key", get(get_api_key))
        .route("/api/v1/plans/:id/key", put(update_api_key))
        .route("/api/v1/plans/:id/key/test", post(test_api_key))
        // Config
        .route("/api/v1/config", get(get_config))
        .route("/api/v1/config", put(update_config))
        .route("/api/v1/config/export", get(export_config))
        .route("/api/v1/config/import", post(import_config))
        .route("/api/v1/config/reset", post(reset_config))
        .with_state(state)
}
