//! 插件清单
//!
//! 定义插件的元数据格式和序列化/反序列化逻辑

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::model_types::PluginType;

/// 插件清单加载/保存错误
#[derive(Debug, Error)]
pub enum ManifestError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("YAML parsing error: {0}")]
    Parse(#[from] serde_yaml::Error),
}

/// 插件清单
///
/// 包含插件的基本信息、入口点、权限和依赖
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    /// 插件唯一标识符
    pub id: String,
    /// 插件显示名称
    pub name: String,
    /// 语义化版本
    pub version: String,
    /// 插件描述
    pub description: String,
    /// 作者
    pub author: String,
    /// 插件类型
    pub plugin_type: PluginType,
    /// WASM 入口点
    pub entry_point: String,
    /// 所需权限列表
    pub permissions: Vec<String>,
    /// 插件依赖
    pub dependencies: Vec<PluginDependency>,
    /// 编译目标
    pub wasm_target: String,
}

/// 插件依赖
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDependency {
    /// 依赖的插件 ID
    pub id: String,
    /// 版本范围 (如 ">=1.0.0")
    pub version_range: String,
}

impl PluginManifest {
    /// 从文件加载清单
    ///
    /// # Errors
    /// 返回 `ManifestError` 如果文件读取或解析失败
    pub async fn load_from_file(path: &std::path::Path) -> Result<Self, ManifestError> {
        let content = tokio::fs::read_to_string(path).await?;
        Self::from_yaml(&content)
    }

    /// 从 YAML 字符串解析
    pub fn from_yaml(yaml: &str) -> Result<Self, ManifestError> {
        // serde_yaml::Error 实现了 From<serde_yaml::Error> 对于 ManifestError::Parse
        Ok(serde_yaml::from_str(yaml)?)
    }

    /// 保存清单到文件
    ///
    /// # Errors
    /// 返回 `ManifestError` 如果序列化或写入失败
    pub async fn save_to_file(&self, path: &std::path::Path) -> Result<(), ManifestError> {
        let yaml = serde_yaml::to_string(self)?;
        tokio::fs::write(path, yaml).await?;
        Ok(())
    }

    /// 转换为 YAML 字符串
    pub fn to_yaml(&self) -> Result<String, ManifestError> {
        Ok(serde_yaml::to_string(self)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model_types::PluginType;

    #[test]
    fn test_from_yaml_valid() {
        let yaml = r#"
id: test-plugin
name: Test Plugin
version: 1.0.0
description: A test plugin
author: Test Author
plugin_type: provider
entry_point: main
permissions:
  - http
  - log
dependencies:
  - id: dep1
    version_range: ">=1.0.0"
wasm_target: wasm32-wasi
"#;
        let result = PluginManifest::from_yaml(yaml);
        assert!(result.is_ok());
        let manifest = result.unwrap();
        assert_eq!(manifest.id, "test-plugin");
        assert_eq!(manifest.name, "Test Plugin");
        assert_eq!(manifest.version, "1.0.0");
        assert_eq!(manifest.plugin_type, PluginType::Provider);
        assert_eq!(manifest.permissions.len(), 2);
        assert_eq!(manifest.entry_point, "main");
    }

    #[test]
    fn test_from_yaml_transform_type() {
        let yaml = r#"
id: transform-plugin
name: Transform Plugin
version: 2.0.0
description: A transform plugin
author: Test Author
plugin_type: transform
entry_point: transform
permissions: []
dependencies: []
wasm_target: wasm32-wasi
"#;
        let result = PluginManifest::from_yaml(yaml);
        assert!(result.is_ok());
        let manifest = result.unwrap();
        assert_eq!(manifest.plugin_type, PluginType::Transform);
    }

    #[test]
    fn test_from_yaml_invalid() {
        let yaml = "invalid yaml: [unclosed";
        let result = PluginManifest::from_yaml(yaml);
        assert!(result.is_err());
    }

    #[test]
    fn test_to_yaml() {
        let manifest = PluginManifest {
            id: "roundtrip-plugin".to_string(),
            name: "Roundtrip Plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "Testing roundtrip".to_string(),
            author: "Test".to_string(),
            plugin_type: PluginType::Tool,
            entry_point: "main".to_string(),
            permissions: vec!["http".to_string()],
            dependencies: vec![],
            wasm_target: "wasm32-wasi".to_string(),
        };

        let yaml = manifest.to_yaml().unwrap();
        assert!(yaml.contains("roundtrip-plugin"));
        assert!(yaml.contains("Roundtrip Plugin"));
    }

    #[test]
    fn test_yaml_dependencies() {
        let yaml = r#"
id: dep-plugin
name: Dep Plugin
version: 1.0.0
description: Plugin with dependencies
author: Test
plugin_type: tool
entry_point: main
permissions: []
dependencies:
  - id: core-plugin
    version_range: ">=1.0.0"
  - id: ext-plugin
    version_range: ">=2.0.0, <3.0.0"
wasm_target: wasm32-wasi
"#;
        let result = PluginManifest::from_yaml(yaml);
        assert!(result.is_ok());
        let manifest = result.unwrap();
        assert_eq!(manifest.dependencies.len(), 2);
        assert_eq!(manifest.dependencies[0].id, "core-plugin");
        assert_eq!(manifest.dependencies[0].version_range, ">=1.0.0");
        assert_eq!(manifest.dependencies[1].id, "ext-plugin");
    }
}