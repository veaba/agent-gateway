//! API 共享状态

use std::sync::Arc;
use tokio::sync::RwLock;

use agw_core::{
    business::{PlanManager, ProviderEngine, QuotaTracker, HealthChecker},
    model::FallbackConfig,
    plugin::{PluginRegistry, PluginLifecycle},
    storage::{ConfigStore, RequestLogStore, SqliteStore},
};

use crate::types::ApiConfig;

/// API 共享状态
#[derive(Clone)]
pub struct AppState {
    /// 配置存储
    pub config_store: Arc<ConfigStore>,
    /// 套餐管理器
    pub plan_manager: Arc<PlanManager>,
    /// Provider 引擎
    pub provider_engine: Arc<ProviderEngine>,
    /// 配额追踪器
    pub quota_tracker: Arc<QuotaTracker>,
    /// Fallback 配置
    pub fallback_config: Arc<RwLock<FallbackConfig>>,
    /// 插件注册表
    pub plugin_registry: Arc<PluginRegistry>,
    /// 插件生命周期管理器
    pub plugin_lifecycle: Arc<PluginLifecycle>,
    /// 请求日志存储
    pub log_store: Arc<RequestLogStore>,
    /// SQLite 存储
    pub sqlite_store: Arc<SqliteStore>,
    /// 健康检查器
    pub health_checker: Arc<HealthChecker>,
    /// API 配置
    pub api_config: ApiConfig,
}

impl AppState {
    /// 初始化 AppState
    pub async fn init() -> anyhow::Result<Self> {
        tracing::info!("Initializing AppState...");

        // 初始化配置存储
        let config_store = Arc::new(ConfigStore::new()?);
        config_store.init_data_dir().await?;

        // 初始化日志存储
        let log_dir = config_store.data_dir().join("logs");
        let log_store = Arc::new(RequestLogStore::new(log_dir));

        // 创建 SQLite 存储
        let sqlite_path = config_store.data_dir().join("gateway.db");
        let sqlite_store = Arc::new(SqliteStore::new(sqlite_path)?);

        // 初始化业务组件
        let mut plan_manager = PlanManager::new(Arc::clone(&config_store));

        // 预加载套餐（即使为空也正常）
        let plans = plan_manager.load_all().await?;

        // 创建 Provider 引擎
        let provider_engine = Arc::new(ProviderEngine::new());

        // 设置 PlanManager 的健康检查依赖
        plan_manager.with_health_check_deps(
            Arc::clone(&provider_engine),
            Arc::clone(&sqlite_store),
        );
        let plan_manager = Arc::new(plan_manager);

        // 创建带 SQLite 持久化的 QuotaTracker
        let quota_tracker = Arc::new(QuotaTracker::with_sqlite(sqlite_store.clone()));

        // 从 SQLite 加载现有配额数据
        let plan_ids: Vec<String> = plans.iter().map(|p| p.id.clone()).collect();
        quota_tracker.load_from_sqlite(&plan_ids).await?;

        // 同步 Plan 配额限制到 QuotaTracker
        for plan in &plans {
            let limits = agw_core::business::quota::QuotaLimit {
                daily: plan.custom_quota_daily,
                monthly: plan.custom_quota_monthly,
                rpm: plan.custom_rpm_limit,
            };
            quota_tracker.set_limits(&plan.id, limits).await;
        }

        // 加载 Fallback 配置
        let fallback_config = config_store.load_fallback_config().await
            .unwrap_or_else(|e| {
                tracing::warn!("Failed to load fallback config: {}", e);
                FallbackConfig::default()
            });
        let fallback_config = Arc::new(RwLock::new(fallback_config));

        // 初始化插件系统
        let plugin_lifecycle = Arc::new(PluginLifecycle::new());
        let plugin_registry = Arc::new(plugin_lifecycle.registry().clone());

        // 加载已安装的插件
        if let Err(e) = plugin_lifecycle.load_installed_plugins().await {
            tracing::warn!("Failed to load installed plugins: {}", e);
        }

        // 创建健康检查器
        let health_checker = Arc::new(HealthChecker::new(
            Arc::clone(&provider_engine),
            Arc::clone(&sqlite_store),
            Arc::clone(&plan_manager),
        ));

        // 加载 API 配置
        let api_config = Self::load_api_config(&config_store).await;

        tracing::info!("AppState initialized successfully");
        Ok(Self {
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
            api_config,
        })
    }

    /// 加载 API 配置
    /// 优先从用户配置目录加载 api.yaml，否则使用内置默认值
    async fn load_api_config(config_store: &ConfigStore) -> ApiConfig {
        // 尝试从用户配置目录加载
        let config_path = config_store.config_dir().join("api.yaml");
        if config_path.exists() {
            match tokio::fs::read_to_string(&config_path).await {
                Ok(content) => {
                    match serde_yaml::from_str::<ApiConfig>(&content) {
                        Ok(config) => {
                            tracing::info!("Loaded API config from {}", config_path.display());
                            return config;
                        }
                        Err(e) => {
                            tracing::warn!("Failed to parse api.yaml: {}, using defaults", e);
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to read api.yaml: {}, using defaults", e);
                }
            }
        }

        // 使用内置默认值
        ApiConfig::default()
    }

    /// 创建 AppState（带自定义配置目录）
    pub async fn with_config_dir(config_dir: std::path::PathBuf) -> anyhow::Result<Self> {
        let config_store = Arc::new(ConfigStore::with_path(config_dir)?);
        config_store.init_data_dir().await?;

        // 初始化日志存储
        let log_dir = config_store.data_dir().join("logs");
        let log_store = Arc::new(RequestLogStore::new(log_dir));

        // 创建 SQLite 存储
        let sqlite_path = config_store.data_dir().join("gateway.db");
        let sqlite_store = Arc::new(SqliteStore::new(sqlite_path)?);

        let mut plan_manager = PlanManager::new(Arc::clone(&config_store));

        // 加载套餐
        let plans = plan_manager.load_all().await?;

        // 创建 Provider 引擎
        let provider_engine = Arc::new(ProviderEngine::new());

        // 设置 PlanManager 的健康检查依赖
        plan_manager.with_health_check_deps(
            Arc::clone(&provider_engine),
            Arc::clone(&sqlite_store),
        );
        let plan_manager = Arc::new(plan_manager);

        // 创建带 SQLite 持久化的 QuotaTracker
        let quota_tracker = Arc::new(QuotaTracker::with_sqlite(sqlite_store.clone()));

        // 从 SQLite 加载现有配额数据
        let plan_ids: Vec<String> = plans.iter().map(|p| p.id.clone()).collect();
        quota_tracker.load_from_sqlite(&plan_ids).await?;

        // 同步 Plan 配额限制到 QuotaTracker
        for plan in &plans {
            let limits = agw_core::business::quota::QuotaLimit {
                daily: plan.custom_quota_daily,
                monthly: plan.custom_quota_monthly,
                rpm: plan.custom_rpm_limit,
            };
            quota_tracker.set_limits(&plan.id, limits).await;
        }

        let fallback_config = config_store.load_fallback_config().await
            .unwrap_or_else(|e| {
                tracing::warn!("Failed to load fallback config: {}", e);
                FallbackConfig::default()
            });
        let fallback_config = Arc::new(RwLock::new(fallback_config));

        // 初始化插件系统
        let plugin_lifecycle = Arc::new(PluginLifecycle::new());
        let plugin_registry = Arc::new(plugin_lifecycle.registry().clone());

        // 加载已安装的插件
        if let Err(e) = plugin_lifecycle.load_installed_plugins().await {
            tracing::warn!("Failed to load installed plugins: {}", e);
        }

        // 创建健康检查器
        let health_checker = Arc::new(HealthChecker::new(
            Arc::clone(&provider_engine),
            Arc::clone(&sqlite_store),
            Arc::clone(&plan_manager),
        ));

        // 加载 API 配置
        let api_config = Self::load_api_config(&config_store).await;

        Ok(Self {
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
            api_config,
        })
    }
}
