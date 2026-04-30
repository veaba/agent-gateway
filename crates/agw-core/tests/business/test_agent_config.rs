//! AgentAutoConfig integration tests

use agw_core::business::AgentAutoConfig;
use agw_core::model::{ProviderOnboarding, AgentSetupGuide};

#[tokio::test]
async fn test_agent_config_configure_claude_code() {
    let result = AgentAutoConfig::configure("claude-code", "127.0.0.1:8080").await;
    // Should succeed in creating config report
    assert!(result.is_ok());
    let report = result.unwrap();
    assert_eq!(report.agent, "Claude Code");
    assert!(!report.method.is_empty());
}

#[tokio::test]
async fn test_agent_config_configure_kimi_cli() {
    let result = AgentAutoConfig::configure("kimi-cli", "127.0.0.1:8080").await;
    assert!(result.is_ok());
    let report = result.unwrap();
    assert_eq!(report.agent, "Kimi CLI");
}

#[tokio::test]
async fn test_agent_config_configure_opencode() {
    let result = AgentAutoConfig::configure("opencode", "127.0.0.1:8080").await;
    assert!(result.is_ok());
    let report = result.unwrap();
    assert_eq!(report.agent, "OpenCode");
}

#[tokio::test]
async fn test_agent_config_configure_kilo_cli() {
    let result = AgentAutoConfig::configure("kilo-cli", "127.0.0.1:8080").await;
    assert!(result.is_ok());
    let report = result.unwrap();
    assert_eq!(report.agent, "Kilo CLI");
}

#[tokio::test]
async fn test_agent_config_configure_unknown() {
    let result = AgentAutoConfig::configure("unknown-agent", "127.0.0.1:8080").await;
    assert!(result.is_err());
}

#[test]
fn test_agent_config_get_setup_guide() {
    let onboarding = ProviderOnboarding {
        description: "Test provider".to_string(),
        signup_url: "https://example.com/signup".to_string(),
        plans_comparison_url: None,
        get_key_url: None,
        setup_guide_url: None,
        faq_url: None,
        agent_setup_guides: vec![
            AgentSetupGuide {
                agent_id: "claude-code".to_string(),
                agent_name: "Claude Code".to_string(),
                auto_config_supported: true,
                auto_config_script: None,
                manual_steps: vec![],
                config_file_paths: agw_core::model::PlatformPaths {
                    macos: None,
                    linux: None,
                    windows: None,
                },
                env_vars: vec![],
            },
            AgentSetupGuide {
                agent_id: "kimi-cli".to_string(),
                agent_name: "Kimi CLI".to_string(),
                auto_config_supported: false,
                auto_config_script: None,
                manual_steps: vec![],
                config_file_paths: agw_core::model::PlatformPaths {
                    macos: None,
                    linux: None,
                    windows: None,
                },
                env_vars: vec![],
            },
        ],
    };

    let guide = AgentAutoConfig::get_setup_guide(&onboarding, "claude-code");
    assert!(guide.is_some());
    assert_eq!(guide.unwrap().agent_id, "claude-code");

    let guide = AgentAutoConfig::get_setup_guide(&onboarding, "kimi-cli");
    assert!(guide.is_some());
    assert_eq!(guide.unwrap().agent_id, "kimi-cli");

    let guide = AgentAutoConfig::get_setup_guide(&onboarding, "nonexistent");
    assert!(guide.is_none());
}
