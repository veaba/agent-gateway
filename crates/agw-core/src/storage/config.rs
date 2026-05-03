//! 配置文件存储

use std::path::PathBuf;
use anyhow::Result;

use crate::model::{UserPlansConfig, ProvidersConfig, FallbackConfig, CustomAgentsConfig, CustomProvidersConfig};

/// 配置存储
pub struct ConfigStore {
    config_dir: PathBuf,
}

impl ConfigStore {
    /// 创建配置存储
    pub fn new() -> Result<Self> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Cannot find config directory"))?
            .join("agent-gateway");

        std::fs::create_dir_all(&config_dir)?;

        Ok(Self { config_dir })
    }

    /// 创建配置存储（带自定义路径）
    pub fn with_path(path: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&path)?;
        Ok(Self { config_dir: path })
    }

    /// 获取配置目录
    pub fn config_dir(&self) -> &PathBuf {
        &self.config_dir
    }

    /// 加载用户套餐配置
    pub async fn load_user_plans(&self) -> Result<UserPlansConfig> {
        let path = self.config_dir.join("user_plans.yaml");

        if !path.exists() {
            return Ok(UserPlansConfig::default());
        }

        let content = tokio::fs::read_to_string(&path).await?;
        let config: UserPlansConfig = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    /// 保存用户套餐配置
    pub async fn save_user_plans(&self, config: &UserPlansConfig) -> Result<()> {
        let path = self.config_dir.join("user_plans.yaml");
        let content = serde_yaml::to_string(config)?;
        tokio::fs::write(&path, content).await?;
        Ok(())
    }

    /// 设置默认套餐
    pub async fn set_default_plan(&self, plan_id: &str) -> Result<()> {
        let mut config = self.load_user_plans().await?;
        config.default_user_plan_id = Some(plan_id.to_string());
        self.save_user_plans(&config).await
    }

    /// 加载内置 Provider 配置
    pub async fn load_providers(&self) -> Result<ProvidersConfig> {
        let path = self.config_dir.join("providers_builtin.yaml");

        if !path.exists() {
            return Ok(ProvidersConfig {
                version: "0.1.0".to_string(),
                providers: vec![],
            });
        }

        let content = tokio::fs::read_to_string(&path).await?;
        let config: ProvidersConfig = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    /// 加载 Fallback 配置
    pub async fn load_fallback_config(&self) -> Result<FallbackConfig> {
        let path = self.config_dir.join("fallback.yaml");

        if !path.exists() {
            return Ok(FallbackConfig::default());
        }

        let content = tokio::fs::read_to_string(&path).await?;
        let config: FallbackConfig = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    /// 保存 Fallback 配置
    pub async fn save_fallback_config(&self, config: &FallbackConfig) -> Result<()> {
        let path = self.config_dir.join("fallback.yaml");
        let content = serde_yaml::to_string(config)?;
        tokio::fs::write(&path, content).await?;
        Ok(())
    }

    /// 获取数据目录
    pub fn data_dir(&self) -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("agent-gateway")
    }

    /// 初始化数据目录
    pub async fn init_data_dir(&self) -> Result<()> {
        let data_dir = self.data_dir();
        tokio::fs::create_dir_all(&data_dir).await?;
        tokio::fs::create_dir_all(data_dir.join("logs")).await?;
        tokio::fs::create_dir_all(data_dir.join("plugins")).await?;
        Ok(())
    }

    /// 加载自定义 Agent 配置
    pub async fn load_custom_agents(&self) -> Result<CustomAgentsConfig> {
        let path = self.config_dir.join("custom_agents.yaml");

        if !path.exists() {
            return Ok(CustomAgentsConfig::default());
        }

        let content = tokio::fs::read_to_string(&path).await?;
        let config: CustomAgentsConfig = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    /// 保存自定义 Agent 配置
    pub async fn save_custom_agents(&self, config: &CustomAgentsConfig) -> Result<()> {
        let path = self.config_dir.join("custom_agents.yaml");
        let content = serde_yaml::to_string(config)?;
        tokio::fs::write(&path, content).await?;
        Ok(())
    }

    /// 加载自定义 Provider 配置
    pub async fn load_custom_providers(&self) -> Result<CustomProvidersConfig> {
        let path = self.config_dir.join("custom_providers.yaml");

        if !path.exists() {
            return Ok(CustomProvidersConfig::default());
        }

        let content = tokio::fs::read_to_string(&path).await?;
        let config: CustomProvidersConfig = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    /// 保存自定义 Provider 配置
    pub async fn save_custom_providers(&self, config: &CustomProvidersConfig) -> Result<()> {
        let path = self.config_dir.join("custom_providers.yaml");
        let content = serde_yaml::to_string(config)?;
        tokio::fs::write(&path, content).await?;
        Ok(())
    }
}

impl Default for ConfigStore {
    fn default() -> Self {
        Self::new().expect("Failed to create config store")
    }
}