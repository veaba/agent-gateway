//! API Key 助手

use crate::model::ProviderTemplate;
use anyhow::Result;

/// API Key 助手
pub struct ApiKeyHelper;

impl ApiKeyHelper {
    /// 打开 Provider 的 API Key 获取页面
    pub fn open_get_key_page(provider: &ProviderTemplate) -> Result<()> {
        let url = provider.get_api_key_url
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Provider does not have a key page URL"))?;

        open::that(url)?;
        Ok(())
    }

    /// 打开配置指南
    pub fn open_setup_guide(provider: &ProviderTemplate) -> Result<()> {
        if let Some(url) = &provider.setup_guide_url {
            open::that(url)?;
        }
        Ok(())
    }

    /// 打开注册页面
    pub fn open_signup_page(provider: &ProviderTemplate) -> Result<()> {
        open::that(&provider.onboarding.signup_url)?;
        Ok(())
    }

    /// 验证 API Key 格式
    pub fn validate_key_format(provider: &ProviderTemplate, key: &str) -> Result<()> {
        let trimmed = key.trim();

        match provider.provider_id.as_str() {
            "anthropic" | "alaya" => {
                if !trimmed.starts_with("sk-") && !trimmed.starts_with("sk-ant-") {
                    anyhow::bail!("API Key should start with 'sk-' or 'sk-ant-'");
                }
            }
            "openai" => {
                if !trimmed.starts_with("sk-") {
                    anyhow::bail!("API Key should start with 'sk-'");
                }
            }
            "kimi" => {
                if !trimmed.starts_with("sk-") {
                    anyhow::bail!("API Key should start with 'sk-'");
                }
            }
            _ => {}
        }

        if trimmed.len() < 20 {
            anyhow::bail!("API Key seems too short");
        }

        Ok(())
    }

    /// 检测剪贴板内容是否为可能的 API Key
    pub fn is_likely_api_key(content: &str) -> bool {
        let trimmed = content.trim();
        let prefixes = ["sk-", "sk-ant-", "sk-proj-", "AIza", "gsk_", "kilo_"];
        prefixes.iter().any(|p| trimmed.starts_with(p)) && trimmed.len() > 20
    }
}