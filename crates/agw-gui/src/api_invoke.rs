//! API Invoke Handlers
//!
//! 为嵌入式模式提供 API invoke handlers
//! 这些 handlers 直接调用 AppState 的方法，绕过 HTTP

use tauri::State;
use agw_api::state::AppState;
use agw_core::model::UserPlan;
use agw_core::model::ProviderTemplate;
use agw_core::model::FallbackConfig;

/// 获取所有 Plans
#[tauri::command]
pub async fn fetch_plans(
    state: State<'_, AppState>,
) -> Result<Vec<UserPlan>, String> {
    state.plan_manager.load_all().await
        .map_err(|e| e.to_string())
}

/// 获取单个 Plan
#[tauri::command]
pub async fn fetch_plan(
    state: State<'_, AppState>,
    id: String,
) -> Result<Option<UserPlan>, String> {
    Ok(state.plan_manager.get(&id).await)
}

/// 获取所有 Providers
#[tauri::command]
pub async fn fetch_providers(
    state: State<'_, AppState>,
) -> Result<Vec<ProviderTemplate>, String> {
    Ok(state.provider_engine.list_providers().await)
}

/// 获取单个 Provider
#[tauri::command]
pub async fn fetch_provider(
    state: State<'_, AppState>,
    id: String,
) -> Result<Option<ProviderTemplate>, String> {
    Ok(state.provider_engine.get_provider(&id).await)
}

/// 获取 Fallback 配置
#[tauri::command]
pub async fn fetch_fallback_config(
    state: State<'_, AppState>,
) -> Result<FallbackConfig, String> {
    let config = state.fallback_config.read().await;
    Ok(config.clone())
}

/// 获取 Quota 状态
#[tauri::command]
pub async fn fetch_quota_status(
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    // 返回所有 plans 的 quota 状态
    let plans = state.plan_manager.load_all().await
        .map_err(|e| e.to_string())?;

    let mut status = serde_json::Map::new();
    for plan in plans {
        let usage = state.quota_tracker.get_usage(&plan.id).await;
        if let Some(record) = usage {
            status.insert(plan.id, serde_json::json!({
                "daily_used": record.daily_used,
                "monthly_used": record.monthly_used,
                "rpm_used": record.rpm_used,
                "plan_name": plan.name
            }));
        } else {
            status.insert(plan.id, serde_json::json!({
                "daily_used": 0,
                "monthly_used": 0,
                "rpm_used": 0,
                "plan_name": plan.name
            }));
        }
    }

    Ok(serde_json::Value::Object(status))
}

/// 测试外部服务器连接
#[tauri::command]
pub async fn test_external_connection(endpoint: String) -> Result<bool, String> {
    // 使用 agw-core 的 Forwarder 客户端进行测试
    let forwarder = agw_core::core::Forwarder::new();
    let url = format!("{}/health", endpoint.trim_end_matches('/'));

    match forwarder.client.get(&url).timeout(std::time::Duration::from_secs(5)).send().await {
        Ok(resp) => {
            if resp.status().is_success() {
                Ok(true)
            } else {
                Err(format!("Server returned status {}", resp.status()))
            }
        }
        Err(e) => Err(format!("Connection failed: {}", e))
    }
}