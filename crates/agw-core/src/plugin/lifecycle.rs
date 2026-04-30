//! 插件生命周期管理

use anyhow::Result;

use super::{PluginEngine, PluginRegistry, PluginInfo, PluginInstaller, PluginSource, PluginManifest};
use crate::model_types::{PluginStatus, PluginType};

/// 插件生命周期管理器
pub struct PluginLifecycle {
    engine: PluginEngine,
    registry: PluginRegistry,
}

impl PluginLifecycle {
    /// 创建新的生命周期管理器
    pub fn new() -> Self {
        Self {
            engine: PluginEngine::new().expect("Failed to create plugin engine"),
            registry: PluginRegistry::new(),
        }
    }

    /// 安装插件
    ///
    /// # Arguments
    /// * `source` - 安装源 (本地文件、GitHub、远程 URL)
    ///
    /// # Returns
    /// 安装后的插件信息
    pub async fn install(&self, source: &str) -> Result<PluginInfo> {
        tracing::info!("Installing plugin from: {}", source);

        // 解析源
        let plugin_source = PluginInstaller::parse_source(source);

        // 下载/读取 WASM 模块
        let wasm_bytes = match &plugin_source {
            PluginSource::Local(path) => {
                tracing::debug!("Installing from local file: {:?}", path);
                PluginInstaller::install_from_file(path).await?
            }
            PluginSource::GitHub { owner, repo, version } => {
                tracing::debug!("Installing from GitHub: {}/{}@{}", owner, repo, version);
                let resolved_version = if version == "latest" {
                    PluginInstaller::fetch_github_latest_version(owner, repo).await?
                } else {
                    version.clone()
                };
                tracing::debug!("Resolved GitHub version: {}", resolved_version);
                PluginInstaller::install_from_github(owner, repo, &resolved_version).await?
            }
            PluginSource::Url(url) => {
                tracing::debug!("Installing from URL: {}", url);
                PluginInstaller::install_from_url(url).await?
            }
        };

        // 尝试从 WASM 提取清单
        let manifest = PluginInstaller::extract_manifest_from_wasm(&wasm_bytes)?;

        // 如果 WASM 内嵌清单不存在，尝试从外部文件加载
        let manifest = if let Some(m) = manifest {
            Some(m)
        } else {
            // 对于本地文件，尝试加载同目录的 manifest.yaml
            if let PluginSource::Local(path) = &plugin_source {
                let manifest_path = path.parent()
                    .unwrap_or(path)
                    .join("manifest.yaml");

                if tokio::fs::try_exists(&manifest_path).await? {
                    tracing::debug!("Loading external manifest from {:?}", manifest_path);
                    Some(PluginManifest::load_from_file(&manifest_path).await?)
                } else {
                    None
                }
            } else {
                None
            }
        };

        // 如果没有清单，使用默认值
        let plugin_id = manifest.as_ref()
            .map(|m| m.id.clone())
            .unwrap_or_else(|| {
                // 从文件名生成 ID
                match &plugin_source {
                    PluginSource::Local(path) => path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")
                        .to_string(),
                    PluginSource::GitHub { repo, .. } => repo.clone(),
                    PluginSource::Url(url) => {
                        // 从 URL 提取文件名
                        url.split('/')
                            .last()
                            .unwrap_or("unknown")
                            .replace(".wasm", "")
                    }
                }
            });

        // 检查是否已安装
        if PluginInstaller::is_installed(&plugin_id).await {
            anyhow::bail!(
                "Plugin '{}' is already installed. Use 'update' to upgrade to a new version.",
                plugin_id
            );
        }

        // 检查依赖是否满足
        if let Some(m) = &manifest {
            let missing = PluginInstaller::check_dependencies(m).await?;
            if !missing.is_empty() {
                anyhow::bail!(
                    "Plugin '{}' has unsatisfied dependencies: {}",
                    plugin_id,
                    missing.join(", ")
                );
            }
        }

        // 保存 WASM 文件（事务性：失败时回滚）
        let wasm_path = match PluginInstaller::save_wasm(&plugin_id, &wasm_bytes).await {
            Ok(path) => path,
            Err(e) => {
                PluginInstaller::uninstall_plugin_files(&plugin_id).await.ok();
                return Err(e);
            }
        };

        // 保存清单文件（如果存在）
        if let Some(m) = &manifest {
            if let Err(e) = PluginInstaller::save_manifest(&plugin_id, m).await {
                // 回滚 WASM 文件
                PluginInstaller::uninstall_plugin_files(&plugin_id).await.ok();
                return Err(e);
            }
        }

        // 验证 WASM 模块可加载
        if let Err(e) = self.engine.load_plugin(&wasm_bytes).await {
            // 回滚所有文件
            PluginInstaller::uninstall_plugin_files(&plugin_id).await.ok();
            anyhow::bail!("Plugin WASM validation failed: {}", e);
        }
        tracing::debug!("Plugin module loaded successfully: {}", plugin_id);

        // 创建插件信息
        let plugin_info = if let Some(m) = manifest {
            PluginInfo {
                id: m.id.clone(),
                name: m.name.clone(),
                version: m.version.clone(),
                plugin_type: m.plugin_type,
                status: PluginStatus::Installed,
                description: m.description.clone(),
                author: m.author.clone(),
                entry_point: m.entry_point.clone(),
                wasm_path,
            }
        } else {
            PluginInfo {
                id: plugin_id.clone(),
                name: plugin_id.clone(),
                version: "0.0.1".to_string(),
                plugin_type: PluginType::Tool, // 默认为工具类型
                status: PluginStatus::Installed,
                description: "Plugin without manifest".to_string(),
                author: "Unknown".to_string(),
                entry_point: "main".to_string(), // 默认入口点
                wasm_path,
            }
        };

        // 注册到注册表
        self.registry.register(plugin_info.clone());

        tracing::info!("Plugin installed successfully: {} ({})", plugin_info.id, plugin_info.name);
        Ok(plugin_info)
    }

    /// 更新插件
    ///
    /// 备份旧版本，下载新版本，验证后替换
    ///
    /// # Arguments
    /// * `id` - 插件 ID
    /// * `source` - 新的安装源（可选，默认使用当前来源或从注册表推断）
    pub async fn update(&self, id: &str, source: Option<&str>) -> Result<PluginInfo> {
        tracing::info!("Updating plugin: {}", id);

        // 检查插件是否已安装
        let _installed = PluginInstaller::get_installed_info(id).await?
            .ok_or_else(|| anyhow::anyhow!("Plugin not installed: {}", id))?;

        // 备份旧版本
        PluginInstaller::backup_plugin(id).await?;

        // 确定更新源
        let update_source = if let Some(s) = source {
            s.to_string()
        } else {
            // 尝试从已安装信息推断源
            anyhow::bail!("Update source must be provided for plugin: {}", id);
        };

        // 先卸载旧版本（保留备份）
        self.uninstall(id).await?;

        // 安装新版本
        match self.install(&update_source).await {
            Ok(info) => {
                tracing::info!("Plugin {} updated to version {}", id, info.version);
                Ok(info)
            }
            Err(e) => {
                tracing::error!("Update failed for {}, attempting rollback: {}", id, e);
                // TODO: 从备份恢复
                // 当前简化处理：让用户手动处理
                anyhow::bail!("Update failed for {}: {}. Backup is available in the .backup directory.", id, e);
            }
        }
    }

    /// 卸载插件
    ///
    /// # Arguments
    /// * `id` - 插件 ID
    ///
    /// # Errors
    /// 如果插件不存在或卸载失败
    pub async fn uninstall(&self, id: &str) -> Result<()> {
        tracing::info!("Uninstalling plugin: {}", id);

        // 获取插件信息
        let plugin = self.registry.get(id)
            .ok_or_else(|| anyhow::anyhow!("Plugin not found: {}", id))?;

        // 删除 WASM 文件
        PluginInstaller::delete_wasm(&plugin.wasm_path).await?;

        // 删除清单文件（如果存在）
        let manifest_path = PluginInstaller::plugin_dir().join(format!("{}.yaml", id));
        if tokio::fs::try_exists(&manifest_path).await? {
            tokio::fs::remove_file(&manifest_path).await?;
        }

        // 从注册表移除
        self.registry.unregister(id);

        tracing::info!("Plugin uninstalled successfully: {}", id);
        Ok(())
    }

    /// 启用插件
    pub fn enable(&self, id: &str) -> Result<()> {
        if !self.registry.enable(id) {
            anyhow::bail!("Plugin not found: {}", id);
        }
        tracing::info!("Plugin enabled: {}", id);
        Ok(())
    }

    /// 禁用插件
    pub fn disable(&self, id: &str) -> Result<()> {
        if !self.registry.disable(id) {
            anyhow::bail!("Plugin not found: {}", id);
        }
        tracing::info!("Plugin disabled: {}", id);
        Ok(())
    }

    /// 获取插件详情
    pub fn get(&self, id: &str) -> Option<PluginInfo> {
        self.registry.get(id)
    }

    /// 获取注册表
    pub fn registry(&self) -> &PluginRegistry {
        &self.registry
    }

    /// 获取引擎
    pub fn engine(&self) -> &PluginEngine {
        &self.engine
    }

    /// 加载已安装的插件（从插件目录）
    ///
    /// 扫描插件目录，加载所有已安装的插件到注册表
    pub async fn load_installed_plugins(&self) -> Result<usize> {
        let plugin_dir = PluginInstaller::plugin_dir();

        if !tokio::fs::try_exists(&plugin_dir).await? {
            tracing::debug!("Plugin directory does not exist: {:?}", plugin_dir);
            return Ok(0);
        }

        let mut count = 0;
        let mut entries = tokio::fs::read_dir(&plugin_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            // 只处理 .wasm 文件
            if path.extension().map(|e| e == "wasm").unwrap_or(false) {
                let plugin_id = path.file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.replace(".wasm", ""))
                    .unwrap_or_default();

                if plugin_id.is_empty() {
                    continue;
                }

                // 尝试加载清单
                let manifest = PluginInstaller::load_external_manifest(&plugin_id).await?;

                // 如果没有清单，跳过或使用默认值
                let manifest = match manifest {
                    Some(m) => m,
                    None => {
                        tracing::warn!("Plugin {} has no manifest file, skipping", plugin_id);
                        continue;
                    }
                };

                // 创建插件信息
                let plugin_info = PluginInfo {
                    id: manifest.id.clone(),
                    name: manifest.name.clone(),
                    version: manifest.version.clone(),
                    plugin_type: manifest.plugin_type,
                    status: PluginStatus::Installed,
                    description: manifest.description.clone(),
                    author: manifest.author.clone(),
                    entry_point: manifest.entry_point.clone(),
                    wasm_path: path,
                };

                // 注册到注册表
                self.registry.register(plugin_info);
                count += 1;
                tracing::debug!("Loaded installed plugin: {}", plugin_id);
            }
        }

        tracing::info!("Loaded {} installed plugins", count);
        Ok(count)
    }

    /// 执行插件函数
    ///
    /// # Arguments
    /// * `plugin_id` - 插件 ID
    /// * `function` - 函数名
    /// * `input` - 输入数据
    ///
    /// # Returns
    /// 函数输出
    pub async fn execute(&self, plugin_id: &str, function: &str, input: &[u8]) -> Result<Vec<u8>> {
        let plugin = self.registry.get(plugin_id)
            .ok_or_else(|| anyhow::anyhow!("Plugin not found: {}", plugin_id))?;

        if plugin.status != PluginStatus::Enabled {
            anyhow::bail!("Plugin {} is not enabled", plugin_id);
        }

        // 加载 WASM 模块
        let wasm_bytes = tokio::fs::read(&plugin.wasm_path).await?;
        let mut module = self.engine.load_plugin(&wasm_bytes).await?;

        // 初始化模块
        module.initialize()?;

        // 执行函数
        let input_str = std::str::from_utf8(input)
            .map_err(|_| anyhow::anyhow!("Invalid UTF-8 input"))?;

        module.call_string(function, input_str)
    }
}

impl Default for PluginLifecycle {
    fn default() -> Self {
        Self::new()
    }
}