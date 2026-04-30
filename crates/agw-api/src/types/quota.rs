//! Quota 配额 DTO 类型

use serde::{Deserialize, Serialize};

use agw_core::business::quota::{QuotaRecord, QuotaLimit, QuotaAlert};

/// 配额列表响应
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuotaListResponse {
    pub quotas: Vec<QuotaResponse>,
}

/// 配额响应
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuotaResponse {
    pub plan_id: String,
    pub usage: QuotaUsageResponse,
    pub limits: QuotaLimitsResponse,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alert: Option<QuotaAlert>,
}

/// 配额使用情况
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuotaUsageResponse {
    pub daily_used: u64,
    pub monthly_used: u64,
    pub rpm_used: u32,
}

/// 配额限制
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuotaLimitsResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub daily: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monthly: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rpm: Option<u32>,
}

impl QuotaResponse {
    /// 从 QuotaRecord 和 QuotaLimit 创建响应
    pub fn from_record_and_limit(record: QuotaRecord, limits: Option<QuotaLimit>, alert: Option<QuotaAlert>) -> Self {
        let limits = limits.unwrap_or(QuotaLimit {
            daily: None,
            monthly: None,
            rpm: None,
        });

        Self {
            plan_id: record.plan_id,
            usage: QuotaUsageResponse {
                daily_used: record.daily_used,
                monthly_used: record.monthly_used,
                rpm_used: record.rpm_used,
            },
            limits: QuotaLimitsResponse {
                daily: limits.daily,
                monthly: limits.monthly,
                rpm: limits.rpm,
            },
            alert,
        }
    }
}

/// 更新配额请求
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateQuotaRequest {
    #[serde(default)]
    pub daily: Option<u64>,
    #[serde(default)]
    pub monthly: Option<u64>,
    #[serde(default)]
    pub rpm: Option<u32>,
}
