//! 自定义 Provider 处理器

use axum::{
    extract::{State, Path, Json as AxumJson},
    Json,
};

use crate::state::AppState;
use crate::error::ApiError;
use crate::types::{
    ApiResponse, CustomProviderListResponse, CustomProviderResponse,
    CreateCustomProviderRequest, UpdateCustomProviderRequest,
    parse_api_format,
};
use agw_core::business::CustomProviderUpdate;
use agw_core::model::CustomModel;

/// GET /api/v1/custom-providers
/// 列出所有自定义 Provider
pub async fn list_custom_providers(
    State(state): State<AppState>,
) -> Json<ApiResponse<CustomProviderListResponse>> {
    let providers = state.custom_provider_manager.list().await
        .unwrap_or_else(|e| {
            tracing::warn!("Failed to load custom providers: {}", e);
            Vec::new()
        });

    let responses = providers.into_iter()
        .map(CustomProviderResponse::from)
        .collect();

    Json(ApiResponse::success(CustomProviderListResponse { custom_providers: responses }))
}

/// POST /api/v1/custom-providers
/// 创建自定义 Provider
pub async fn create_custom_provider(
    State(state): State<AppState>,
    AxumJson(payload): AxumJson<CreateCustomProviderRequest>,
) -> Result<Json<ApiResponse<CustomProviderResponse>>, ApiError> {
    // 验证必填字段
    if payload.provider_id.trim().is_empty() {
        return Err(ApiError::Validation("provider_id is required".to_string()));
    }
    if payload.name.trim().is_empty() {
        return Err(ApiError::Validation("name is required".to_string()));
    }
    if payload.base_url.trim().is_empty() {
        return Err(ApiError::Validation("base_url is required".to_string()));
    }

    // 解析 API 格式
    let api_format = parse_api_format(&payload.api_format)
        .map_err(|e| ApiError::Validation(e))?;

    // 转换模型
    let models: Vec<CustomModel> = payload.models.into_iter()
        .map(CustomModel::from)
        .collect();

    let provider = state.custom_provider_manager.create(
        payload.provider_id.trim().to_string(),
        payload.name.trim().to_string(),
        api_format,
        payload.base_url.trim().to_string(),
        payload.requires_api_key,
        payload.description,
        payload.logo_url,
        payload.homepage,
        payload.docs_url,
        payload.get_api_key_url,
        models,
    ).await.map_err(|e| {
        if e.to_string().contains("already exists") {
            ApiError::Conflict(e.to_string())
        } else {
            ApiError::Internal(e.to_string())
        }
    })?;

    Ok(Json(ApiResponse::success(CustomProviderResponse::from(provider))))
}

/// GET /api/v1/custom-providers/:id
/// 获取单个自定义 Provider
pub async fn get_custom_provider(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<CustomProviderResponse>>, ApiError> {
    let provider = state.custom_provider_manager.get(&id).await
        .ok_or_else(|| ApiError::NotFound(format!("Custom provider not found: {}", id)))?;

    Ok(Json(ApiResponse::success(CustomProviderResponse::from(provider))))
}

/// PUT /api/v1/custom-providers/:id
/// 更新自定义 Provider
pub async fn update_custom_provider(
    State(state): State<AppState>,
    Path(id): Path<String>,
    AxumJson(payload): AxumJson<UpdateCustomProviderRequest>,
) -> Result<Json<ApiResponse<CustomProviderResponse>>, ApiError> {
    let api_format = payload.api_format
        .map(|f| parse_api_format(&f))
        .transpose()
        .map_err(|e| ApiError::Validation(e))?;

    let models = payload.models.map(|m| {
        m.into_iter().map(CustomModel::from).collect()
    });

    let updates = CustomProviderUpdate {
        name: payload.name.map(|n| n.trim().to_string()),
        description: payload.description,
        logo_url: payload.logo_url,
        homepage: payload.homepage,
        docs_url: payload.docs_url,
        get_api_key_url: payload.get_api_key_url,
        base_url: payload.base_url.map(|u| u.trim().to_string()),
        api_format,
        requires_api_key: payload.requires_api_key,
        models,
    };

    let provider = state.custom_provider_manager.update(&id, updates).await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(ApiResponse::success(CustomProviderResponse::from(provider))))
}

/// DELETE /api/v1/custom-providers/:id
/// 删除自定义 Provider
pub async fn delete_custom_provider(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<()>>, ApiError> {
    state.custom_provider_manager.delete(&id).await
        .map_err(|e| ApiError::NotFound(e.to_string()))?;

    Ok(Json(ApiResponse::success(())))
}