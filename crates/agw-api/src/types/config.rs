//! API 服务器配置

use serde::{Deserialize, Serialize};

/// 健康检查配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthConfig {
    /// 正常检查间隔（秒）
    #[serde(default = "default_interval_secs")]
    pub interval_secs: u64,

    /// 恢复检查间隔（秒），用于快速检测 Error 状态的 plan 恢复
    #[serde(default = "default_recovery_interval_secs")]
    pub recovery_interval_secs: u64,
}

fn default_interval_secs() -> u64 {
    300 // 5 分钟
}

fn default_recovery_interval_secs() -> u64 {
    60 // 1 分钟
}

impl Default for HealthConfig {
    fn default() -> Self {
        Self {
            interval_secs: default_interval_secs(),
            recovery_interval_secs: default_recovery_interval_secs(),
        }
    }
}

/// API 服务器配置
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ApiConfig {
    /// 健康检查配置
    #[serde(default)]
    pub health: HealthConfig,
}