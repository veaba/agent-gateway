//! Provider 处理器

use axum::{
    extract::{State, Path},
    Json,
};

use crate::state::AppState;
use crate::error::ApiError;
use crate::types::{ApiResponse, ProviderResponse, ProviderListResponse};

/// GET /api/v1/providers
pub async fn list_providers(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<ProviderListResponse>>, ApiError> {
    let providers = state.provider_engine.list_providers().await;
    let responses: Vec<ProviderResponse> = providers.into_iter().map(ProviderResponse::from).collect();
    Ok(Json(ApiResponse::success(ProviderListResponse { providers: responses })))
}

/// POST /api/v1/providers/update
pub async fn update_providers(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<ProviderListResponse>>, ApiError> {
    match state.provider_engine.check_update().await {
        Ok(Some(new_version)) => {
            tracing::info!("Provider definitions updated to: {:?}", new_version);
        }
        Ok(None) => {
            tracing::info!("Provider definitions are up to date");
        }
        Err(e) => {
            tracing::warn!("Failed to check provider updates: {}", e);
        }
    }

    // 返回当前 provider 列表
    let providers = state.provider_engine.list_providers().await;
    let responses: Vec<ProviderResponse> = providers.into_iter().map(ProviderResponse::from).collect();
    Ok(Json(ApiResponse::success(ProviderListResponse { providers: responses })))
}
pub async fn get_provider(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<ProviderResponse>>, ApiError> {
    let provider = state.provider_engine.get_provider(&id).await
        .ok_or_else(|| ApiError::NotFound(format!("Provider not found: {}", id)))?;
    Ok(Json(ApiResponse::success(ProviderResponse::from(provider))))
}
