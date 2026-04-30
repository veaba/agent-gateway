//! 配额处理器

use axum::{
    extract::{State, Query, Path, Json as AxumJson},
    Json,
};
use serde::Deserialize;

use crate::state::AppState;
use crate::error::ApiError;
use crate::types::{ApiResponse, QuotaResponse, QuotaListResponse, UpdateQuotaRequest};

/// 查询参数
#[derive(Debug, Deserialize)]
pub struct QuotaQuery {
    #[serde(default)]
    pub plan_id: Option<String>,
}

/// GET /api/v1/quota
pub async fn quota_status(
    State(state): State<AppState>,
    Query(query): Query<QuotaQuery>,
) -> Json<ApiResponse<QuotaListResponse>> {
    // 获取所有套餐
    let plans = state.plan_manager.load_all().await.unwrap_or_default();
    
    let mut quotas = Vec::new();
    for plan in plans {
        // 如果指定了 plan_id，只返回该套餐的配额
        if let Some(ref filter_id) = query.plan_id {
            if plan.id != *filter_id {
                continue;
            }
        }

        let record = state.quota_tracker.get_usage(&plan.id).await
            .unwrap_or_else(|| agw_core::business::quota::QuotaRecord {
                plan_id: plan.id.clone(),
                daily_used: 0,
                monthly_used: 0,
                rpm_used: 0,
                last_reset: chrono::Utc::now(),
            });
        let limits = state.quota_tracker.get_limits(&plan.id).await;
        let alert = state.quota_tracker.get_alert(&plan.id).await;
        quotas.push(QuotaResponse::from_record_and_limit(record, limits, alert));
    }

    Json(ApiResponse::success(QuotaListResponse { quotas }))
}

/// PUT /api/v1/quota/{plan_id}
pub async fn set_quota(
    State(state): State<AppState>,
    Path(plan_id): Path<String>,
    AxumJson(payload): AxumJson<UpdateQuotaRequest>,
) -> Result<Json<ApiResponse<QuotaResponse>>, ApiError> {
    // 验证 plan 存在
    let _plan = state.plan_manager.get(&plan_id).await
        .ok_or_else(|| ApiError::NotFound(format!("Plan not found: {}", plan_id)))?;

    // 更新配额限制
    let limits = agw_core::business::quota::QuotaLimit {
        daily: payload.daily,
        monthly: payload.monthly,
        rpm: payload.rpm,
    };
    state.quota_tracker.set_limits(&plan_id, limits).await;

    // 获取更新后的配额状态
    let record = state.quota_tracker.get_usage(&plan_id).await
        .unwrap_or_else(|| agw_core::business::quota::QuotaRecord {
            plan_id: plan_id.clone(),
            daily_used: 0,
            monthly_used: 0,
            rpm_used: 0,
            last_reset: chrono::Utc::now(),
        });
    let new_limits = state.quota_tracker.get_limits(&plan_id).await;
    let alert = state.quota_tracker.get_alert(&plan_id).await;

    Ok(Json(ApiResponse::success(QuotaResponse::from_record_and_limit(record, new_limits, alert))))
}
