//! 配置管理处理器

use axum::{
    extract::{State, Query, Json as AxumJson},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use crate::state::AppState;
use crate::types::ApiResponse;
use crate::error::ApiError;

/// 脱敏的运行时配置
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigResponse {
    pub version: String,
    pub data_dir: String,
    pub config_dir: String,
    pub plans_count: usize,
    pub providers_count: usize,
    pub fallback_enabled: bool,
    pub fallback_max_attempts: u32,
    pub plugins_count: usize,
}

/// 更新配置请求
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateConfigRequest {
    #[serde(default)]
    pub fallback_enabled: Option<bool>,
    #[serde(default)]
    pub fallback_max_attempts: Option<u32>,
    #[serde(default)]
    pub log_level: Option<String>,
}

/// 导出配置请求
#[derive(Debug, Deserialize)]
pub struct ExportConfigQuery {
    #[serde(default = "default_format")]
    pub format: String,
}

fn default_format() -> String {
    "json".to_string()
}

/// 导入配置请求
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportConfigRequest {
    pub config: serde_json::Value,
    #[serde(default)]
    pub merge: bool,
}

/// GET /api/v1/config
pub async fn get_config(
    State(state): State<AppState>,
) -> Json<ApiResponse<ConfigResponse>> {
    let plans = state.plan_manager.load_all().await.unwrap_or_default();
    let providers = state.provider_engine.list_providers().await;
    let fallback_config = state.fallback_config.read().await.clone();

    let plugins = state.plugin_registry.list();

    Json(ApiResponse::success(ConfigResponse {
        version: env!("CARGO_PKG_VERSION").to_string(),
        data_dir: state.config_store.data_dir().display().to_string(),
        config_dir: state.config_store.config_dir().display().to_string(),
        plans_count: plans.len(),
        providers_count: providers.len(),
        fallback_enabled: fallback_config.enabled,
        fallback_max_attempts: fallback_config.max_attempts,
        plugins_count: plugins.len(),
    }))
}

/// PUT /api/v1/config
pub async fn update_config(
    State(state): State<AppState>,
    AxumJson(payload): AxumJson<UpdateConfigRequest>,
) -> Result<Json<ApiResponse<ConfigResponse>>, ApiError> {
    // 更新 fallback 配置
    let mut fallback = state.fallback_config.write().await;

    if let Some(enabled) = payload.fallback_enabled {
        fallback.enabled = enabled;
    }
    if let Some(max_attempts) = payload.fallback_max_attempts {
        fallback.max_attempts = max_attempts;
    }

    // 持久化
    state.config_store.save_fallback_config(&fallback).await?;

    let fallback_config = fallback.clone();
    drop(fallback);

    // 获取更新后的统计
    let plans = state.plan_manager.load_all().await.unwrap_or_default();
    let providers = state.provider_engine.list_providers().await;
    let plugins = state.plugin_registry.list();

    Ok(Json(ApiResponse::success(ConfigResponse {
        version: env!("CARGO_PKG_VERSION").to_string(),
        data_dir: state.config_store.data_dir().display().to_string(),
        config_dir: state.config_store.config_dir().display().to_string(),
        plans_count: plans.len(),
        providers_count: providers.len(),
        fallback_enabled: fallback_config.enabled,
        fallback_max_attempts: fallback_config.max_attempts,
        plugins_count: plugins.len(),
    })))
}

/// GET /api/v1/config/export
pub async fn export_config(
    State(state): State<AppState>,
    Query(query): Query<ExportConfigQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let plans = state.plan_manager.load_all().await.unwrap_or_default();
    let fallback_config = state.fallback_config.read().await.clone();

    let config = match query.format.as_str() {
        "yaml" => {
            let yaml = serde_yaml::to_string(&serde_json::json!({
                "plans": plans,
                "fallback": fallback_config,
            })).map_err(|e| ApiError::Internal(e.to_string()))?;

            serde_json::json!({
                "format": "yaml",
                "content": yaml,
            })
        }
        _ => {
            serde_json::json!({
                "format": "json",
                "version": env!("CARGO_PKG_VERSION"),
                "plans": plans,
                "fallback": fallback_config,
            })
        }
    };

    Ok(Json(config))
}

/// POST /api/v1/config/import
pub async fn import_config(
    State(state): State<AppState>,
    AxumJson(payload): AxumJson<ImportConfigRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    let mut imported_plans = 0;
    let mut imported_fallback = false;

    // 导入 plans
    if let Some(plans) = payload.config.get("plans").and_then(|v| v.as_array()) {
        for plan_value in plans {
            if let Ok(plan) = serde_json::from_value::<agw_core::model::UserPlan>(plan_value.clone()) {
                if state.plan_manager.get(&plan.id).await.is_some() {
                    state.plan_manager.update(plan).await?;
                } else {
                    state.plan_manager.add(plan).await?;
                }
                imported_plans += 1;
            }
        }
    }

    // 导入 fallback 配置
    if let Some(fallback_value) = payload.config.get("fallback") {
        if let Ok(fallback) = serde_json::from_value::<agw_core::model::FallbackConfig>(fallback_value.clone()) {
            state.config_store.save_fallback_config(&fallback).await?;
            let mut current = state.fallback_config.write().await;
            *current = fallback;
            imported_fallback = true;
        }
    }

    Ok(Json(ApiResponse::success(serde_json::json!({
        "imported_plans": imported_plans,
        "imported_fallback": imported_fallback,
        "message": format!(
            "Imported {} plans and fallback config",
            imported_plans
        ),
    }))))
}

/// 重置配置到默认值
pub async fn reset_config(
    State(state): State<AppState>,
) -> Result<StatusCode, ApiError> {
    // 重置 fallback 配置
    let fallback = agw_core::model::FallbackConfig::default();
    state.config_store.save_fallback_config(&fallback).await?;

    let mut current = state.fallback_config.write().await;
    *current = fallback;

    Ok(StatusCode::NO_CONTENT)
}