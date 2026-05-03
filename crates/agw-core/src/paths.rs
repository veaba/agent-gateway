//! 统一路径管理模块
//!
//! 所有模块通过此模块获取配置/数据路径
//! 支持 AGENT_GATEWAY_HOME 环境变量覆盖

use std::path::PathBuf;

/// 环境变量名称
pub const APP_HOME_ENV: &str = "AGENT_GATEWAY_HOME";

/// 应用目录名称
pub const APP_DIR_NAME: &str = ".agent-gateway";

// ============================================================================
// 根目录与模块目录
// ============================================================================

/// 获取应用根目录
///
/// 优先级:
/// 1. 环境变量 AGENT_GATEWAY_HOME
/// 2. 用户主目录下的 .agent-gateway
///
/// Windows: C:\Users\<用户名>\.agent-gateway
/// macOS:   /Users/<用户名>\.agent-gateway
/// Linux:   /home/<用户名>\.agent-gateway
pub fn app_home() -> PathBuf {
    if let Ok(custom) = std::env::var(APP_HOME_ENV) {
        tracing::debug!("Using custom AGENT_GATEWAY_HOME: {}", custom);
        return PathBuf::from(custom);
    }

    dirs::home_dir()
        .expect("Cannot determine home directory")
        .join(APP_DIR_NAME)
}

/// 公共配置目录（根目录）
pub fn root_dir() -> PathBuf {
    app_home()
}

/// Core 模块目录
pub fn core_dir() -> PathBuf {
    app_home().join("agw-core")
}

/// CLI 模块目录
pub fn cli_dir() -> PathBuf {
    app_home().join("agw-cli")
}

/// GUI 模块目录
pub fn gui_dir() -> PathBuf {
    app_home().join("agw-gui")
}

// ============================================================================
// 公共配置文件（根目录）
// ============================================================================

/// 公共配置文件路径
pub fn app_config_path() -> PathBuf {
    root_dir().join("agw.yaml")
}

// ============================================================================
// Core 模块路径
// ============================================================================

/// 用户套餐配置
pub fn user_plans_path() -> PathBuf {
    core_dir().join("user_plans.yaml")
}

/// 内置 Provider 定义
pub fn providers_builtin_path() -> PathBuf {
    core_dir().join("providers_builtin.yaml")
}

/// Fallback 配置
pub fn fallback_path() -> PathBuf {
    core_dir().join("fallback.yaml")
}

/// 自定义 Agent 配置
pub fn custom_agents_path() -> PathBuf {
    core_dir().join("custom_agents.yaml")
}

/// 自定义 Provider 配置
pub fn custom_providers_path() -> PathBuf {
    core_dir().join("custom_providers.yaml")
}

/// API 配置
pub fn api_config_path() -> PathBuf {
    core_dir().join("api.yaml")
}

/// 加密密钥
pub fn encryption_key_path() -> PathBuf {
    core_dir().join("encryption.key")
}

/// SQLite 数据库
pub fn db_path() -> PathBuf {
    core_dir().join("gateway.db")
}

/// 日志目录
pub fn logs_dir() -> PathBuf {
    core_dir().join("logs")
}

/// 插件目录
pub fn plugins_dir() -> PathBuf {
    core_dir().join("plugins")
}

// ============================================================================
// CLI 模块路径
// ============================================================================

/// CLI 配置文件
pub fn cli_config_path() -> PathBuf {
    cli_dir().join("config.yaml")
}

/// PID 文件
pub fn pid_path() -> PathBuf {
    cli_dir().join("gateway.pid")
}

// ============================================================================
// GUI 模块路径
// ============================================================================

/// GUI 配置文件
pub fn gui_config_path() -> PathBuf {
    gui_dir().join("config.yaml")
}

/// GUI 托盘配置
pub fn gui_tray_path() -> PathBuf {
    gui_dir().join("tray.yaml")
}

// ============================================================================
// 初始化函数
// ============================================================================

/// 确保所有必要目录存在
pub fn ensure_dirs() -> anyhow::Result<()> {
    std::fs::create_dir_all(root_dir())?;
    std::fs::create_dir_all(core_dir())?;
    std::fs::create_dir_all(cli_dir())?;
    std::fs::create_dir_all(gui_dir())?;
    std::fs::create_dir_all(logs_dir())?;
    std::fs::create_dir_all(plugins_dir())?;
    tracing::debug!("All directories ensured at {}", app_home().display());
    Ok(())
}

/// 初始化应用目录
pub async fn init_app_dirs() -> anyhow::Result<()> {
    ensure_dirs()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    // 使用原子计数器生成唯一测试路径
    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn get_unique_test_path() -> String {
        let id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        format!("/test_path_{}", id)
    }

    #[test]
    fn test_app_home_env_override() {
        let test_path = get_unique_test_path();
        std::env::set_var(APP_HOME_ENV, &test_path);

        let home = app_home();
        assert_eq!(home, PathBuf::from(&test_path));

        std::env::remove_var(APP_HOME_ENV);
    }

    #[test]
    fn test_module_dirs_with_env() {
        let test_path = get_unique_test_path();
        std::env::set_var(APP_HOME_ENV, &test_path);

        let home = app_home();
        assert_eq!(core_dir(), home.join("agw-core"));
        assert_eq!(cli_dir(), home.join("agw-cli"));
        assert_eq!(gui_dir(), home.join("agw-gui"));

        std::env::remove_var(APP_HOME_ENV);
    }

    #[test]
    fn test_path_functions_return_expected_names() {
        let test_path = get_unique_test_path();
        std::env::set_var(APP_HOME_ENV, &test_path);

        assert!(user_plans_path().to_string_lossy().contains("user_plans.yaml"));
        assert!(db_path().to_string_lossy().contains("gateway.db"));
        assert!(logs_dir().to_string_lossy().contains("logs"));
        assert!(plugins_dir().to_string_lossy().contains("plugins"));
        assert!(pid_path().to_string_lossy().contains("gateway.pid"));

        std::env::remove_var(APP_HOME_ENV);
    }
}