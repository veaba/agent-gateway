use std::sync::Arc;
use tokio::sync::RwLock;
use napi_derive::napi;
use napi::Error;
use crate::models::*;

#[napi]
pub struct Gateway {
    #[allow(dead_code)]
    inner: Arc<agw_core::GatewayState>,
    plan_manager: Arc<agw_core::business::PlanManager>,
    provider_engine: Arc<agw_core::business::ProviderEngine>,
    quota_tracker: Arc<agw_core::business::QuotaTracker>,
    fallback_engine: Arc<RwLock<agw_core::business::FallbackEngine>>,
}

#[napi]
impl Gateway {
    #[napi(constructor)]
    pub fn new() -> Result<Self, Error> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| Error::from_reason(format!("Failed to create tokio runtime: {e}")))?;

        let (inner, plan_manager, provider_engine, quota_tracker, fallback_engine) = rt.block_on(async {
            let gw_state = agw_core::GatewayState::new()
                .await
                .map_err(|e| Error::from_reason(format!("Failed to init gateway: {e}")))?;
            
            let plan_manager = Arc::new(agw_core::business::PlanManager::new(gw_state.config_store.clone()));
            let quota_tracker = Arc::new(agw_core::business::QuotaTracker::new());
            let fallback_engine = Arc::new(RwLock::new(
                agw_core::business::FallbackEngine::with_dependencies(
                    agw_core::model::FallbackConfig::default(),
                    plan_manager.clone(),
                    quota_tracker.clone(),
                )
            ));

            Ok::<_, Error>((
                Arc::new(gw_state),
                plan_manager,
                Arc::new(agw_core::business::ProviderEngine::new()),
                quota_tracker,
                fallback_engine,
            ))
        }).map_err(|e: Error| e)?;

        Ok(Gateway { inner, plan_manager, provider_engine, quota_tracker, fallback_engine })
    }

    #[napi]
    pub async fn list_providers(&self) -> Result<Vec<ProviderInfo>, Error> {
        let providers = self.provider_engine.list_providers().await;
        Ok(providers.into_iter().map(Into::into).collect())
    }

    #[napi]
    pub async fn get_provider(&self, provider_id: String) -> Result<Option<ProviderInfo>, Error> {
        let provider = self.provider_engine.get_provider(&provider_id).await;
        Ok(provider.map(Into::into))
    }

    #[napi]
    pub async fn list_plans(&self) -> Result<Vec<PlanInfo>, Error> {
        let plans = self.plan_manager.load_all().await
            .map_err(|e| Error::from_reason(format!("Failed to load plans: {e}")))?;
        Ok(plans.into_iter().map(Into::into).collect())
    }

    #[napi]
    pub async fn get_plan(&self, id: String) -> Result<Option<PlanInfo>, Error> {
        let plan = self.plan_manager.get(&id).await;
        Ok(plan.map(Into::into))
    }

    #[napi]
    pub async fn create_plan(&self, input: CreatePlanInput) -> Result<PlanInfo, Error> {
        let id = format!("{}-{}", input.provider_id, &uuid::Uuid::new_v4().to_string()[..8]);
        let mut plan = agw_core::UserPlan::new(
            id,
            input.provider_id,
            input.plan_id,
            input.name,
            input.api_key,
            input.selected_model_id,
        );
        if let Some(d) = input.custom_quota_daily {
            plan.custom_quota_daily = Some(d as u64);
        }
        if let Some(m) = input.custom_quota_monthly {
            plan.custom_quota_monthly = Some(m as u64);
        }
        if let Some(r) = input.custom_rpm_limit {
            plan.custom_rpm_limit = Some(r);
        }
        if let Some(n) = input.notes {
            plan.notes = Some(n);
        }
        self.plan_manager.add(plan.clone()).await
            .map_err(|e| Error::from_reason(format!("Failed to create plan: {e}")))?;
        Ok(plan.into())
    }

    #[napi]
    pub async fn update_plan(&self, id: String, input: UpdatePlanInput) -> Result<PlanInfo, Error> {
        let mut plan = self.plan_manager.get(&id).await
            .ok_or_else(|| Error::from_reason(format!("Plan not found: {id}")))?;
        
        if let Some(ref name) = input.name { plan.name = name.clone(); }
        if let Some(ref key) = input.api_key { plan.api_key = key.clone(); }
        if let Some(ref model) = input.selected_model_id { plan.selected_model_id = model.clone(); }
        if let Some(enabled) = input.enabled { plan.enabled = enabled; }
        if let Some(priority) = input.priority { plan.priority = priority; }
        if let Some(d) = input.custom_quota_daily { plan.custom_quota_daily = Some(d as u64); }
        if let Some(m) = input.custom_quota_monthly { plan.custom_quota_monthly = Some(m as u64); }
        if let Some(r) = input.custom_rpm_limit { plan.custom_rpm_limit = Some(r); }
        if let Some(t) = input.alert_threshold { plan.alert_threshold = Some(t as f32); }
        if let Some(ref n) = input.notes { plan.notes = Some(n.clone()); }
        
        self.plan_manager.update(plan.clone()).await
            .map_err(|e| Error::from_reason(format!("Failed to update plan: {e}")))?;
        Ok(plan.into())
    }

    #[napi]
    pub async fn delete_plan(&self, id: String) -> Result<(), Error> {
        self.plan_manager.delete(&id).await
            .map_err(|e| Error::from_reason(format!("Failed to delete plan: {e}")))?;
        Ok(())
    }

    #[napi]
    pub async fn test_plan(&self, id: String) -> Result<TestConnectionResult, Error> {
        let success = self.plan_manager.test_connection(&id).await
            .map_err(|e| Error::from_reason(format!("Connection test failed: {e}")))?;
        Ok(TestConnectionResult {
            plan_id: id,
            success,
            message: if success { "Connection successful".to_string() } else { "Connection failed".to_string() },
            latency_ms: None,
        })
    }

    #[napi]
    pub async fn get_fallback_config(&self) -> Result<FallbackConfigInfo, Error> {
        let config = self.fallback_engine.read().await.get_config().clone();
        Ok(config.into())
    }

    #[napi]
    pub async fn set_fallback_enabled(&self, enabled: bool) -> Result<FallbackConfigInfo, Error> {
        self.fallback_engine.write().await.set_enabled(enabled);
        let config = self.fallback_engine.read().await.get_config().clone();
        Ok(config.into())
    }

    #[napi]
    pub async fn set_fallback_priority(&self, priority_order: Vec<String>) -> Result<FallbackConfigInfo, Error> {
        self.fallback_engine.write().await.set_priority(priority_order);
        let config = self.fallback_engine.read().await.get_config().clone();
        Ok(config.into())
    }

    #[napi]
    pub async fn get_quota_usage(&self, plan_id: String) -> Result<Option<QuotaInfo>, Error> {
        let usage = self.quota_tracker.get_usage(&plan_id).await;
        let limits = self.quota_tracker.get_limits(&plan_id).await;
        
        match (usage, limits) {
            (Some(u), Some(l)) => Ok(Some(QuotaInfo {
                plan_id: plan_id.clone(),
                usage: QuotaUsageInfo {
                    plan_id: plan_id.clone(),
                    daily_used: u.daily_used as f64,
                    monthly_used: u.monthly_used as f64,
                    rpm_used: u.rpm_used,
                },
                limits: QuotaLimitsInfo {
                    daily: l.daily.map(|v| v as f64),
                    monthly: l.monthly.map(|v| v as f64),
                    rpm: l.rpm,
                },
            })),
            _ => Ok(None),
        }
    }

    #[napi]
    pub async fn set_quota_limits(
        &self,
        plan_id: String,
        daily: Option<f64>,
        monthly: Option<f64>,
        rpm: Option<u32>,
    ) -> Result<(), Error> {
        let limits = agw_core::business::QuotaLimit {
            daily: daily.map(|v| v as u64),
            monthly: monthly.map(|v| v as u64),
            rpm,
        };
        self.quota_tracker.set_limits(&plan_id, limits).await;
        Ok(())
    }

    #[napi]
    pub fn validate_api_key(&self, content: String) -> bool {
        agw_core::security::ApiKeyHelper::is_likely_api_key(&content)
    }

    #[napi]
    pub fn mask_api_key(&self, api_key: String) -> String {
        if api_key.len() <= 8 {
            "****".to_string()
        } else {
            format!("{}...{}", &api_key[..4], &api_key[api_key.len()-4..])
        }
    }

    #[napi]
    pub fn health(&self) -> HealthResponse {
        HealthResponse {
            status: "ok".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}