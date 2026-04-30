//! Plugin 处理器

use axum::{
    extract::{State, Path, Json as AxumJson},
    http::StatusCode,
    Json,
};

use crate::state::AppState;
use crate::error::ApiError;
use crate::types::{ApiResponse, PluginResponse, PluginListResponse, InstallPluginRequest, UpdatePluginRequest};

/// GET /api/v1/plugins
pub async fn list_plugins(
    State(state): State<AppState>,
) -> Json<ApiResponse<PluginListResponse>> {
    let plugins = state.plugin_registry.list();
    let responses: Vec<PluginResponse> = plugins.into_iter().map(PluginResponse::from).collect();
    Json(ApiResponse::success(PluginListResponse { plugins: responses }))
}

/// POST /api/v1/plugins/install
pub async fn install_plugin(
    State(state): State<AppState>,
    AxumJson(payload): AxumJson<InstallPluginRequest>,
) -> Result<(StatusCode, Json<ApiResponse<PluginResponse>>), ApiError> {
    tracing::info!("Installing plugin from: {}", payload.source);

    let plugin = state.plugin_lifecycle.install(&payload.source).await
        .map_err(|e| ApiError::Internal(format!("Failed to install plugin: {}", e)))?;

    Ok((StatusCode::CREATED, Json(ApiResponse::success(PluginResponse::from(plugin)))))
}

/// DELETE /api/v1/plugins/{id}
pub async fn uninstall_plugin(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    state.plugin_lifecycle.uninstall(&id).await
        .map_err(|e| ApiError::Internal(format!("Failed to uninstall plugin: {}", e)))?;
    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/v1/plugins/{id}/enable
pub async fn enable_plugin(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<PluginResponse>>, ApiError> {
    state.plugin_lifecycle.enable(&id)
        .map_err(|e| ApiError::Internal(format!("Failed to enable plugin: {}", e)))?;

    let plugin = state.plugin_registry.get(&id)
        .ok_or_else(|| ApiError::NotFound(format!("Plugin not found: {}", id)))?;
    Ok(Json(ApiResponse::success(PluginResponse::from(plugin))))
}

/// POST /api/v1/plugins/{id}/disable
pub async fn disable_plugin(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<PluginResponse>>, ApiError> {
    state.plugin_lifecycle.disable(&id)
        .map_err(|e| ApiError::Internal(format!("Failed to disable plugin: {}", e)))?;

    let plugin = state.plugin_registry.get(&id)
        .ok_or_else(|| ApiError::NotFound(format!("Plugin not found: {}", id)))?;
    Ok(Json(ApiResponse::success(PluginResponse::from(plugin))))
}

/// POST /api/v1/plugins/{id}/update
pub async fn update_plugin(
    State(state): State<AppState>,
    Path(id): Path<String>,
    AxumJson(payload): AxumJson<UpdatePluginRequest>,
) -> Result<Json<ApiResponse<PluginResponse>>, ApiError> {
    tracing::info!("Updating plugin: {}", id);

    let source_ref = payload.source.as_deref();
    let plugin = state.plugin_lifecycle.update(&id, source_ref).await
        .map_err(|e| ApiError::Internal(format!("Failed to update plugin: {}", e)))?;

    Ok(Json(ApiResponse::success(PluginResponse::from(plugin))))
}

/// GET /api/v1/plugins/{id}
pub async fn get_plugin(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<PluginResponse>>, ApiError> {
    let plugin = state.plugin_registry.get(&id)
        .ok_or_else(|| ApiError::NotFound(format!("Plugin not found: {}", id)))?;
    Ok(Json(ApiResponse::success(PluginResponse::from(plugin))))
}
