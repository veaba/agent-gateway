//! 插件安装器

use anyhow::Result;
use std::path::PathBuf;
use tokio::fs;

use super::manifest::PluginManifest;
use crate::paths;

/// 已安装插件信息
#[derive(Debug, Clone)]
pub struct InstalledPluginInfo {
    pub id: String,
    pub manifest: Option<PluginManifest>,
    pub wasm_path: PathBuf,
    pub manifest_path: Option<PathBuf>,
    pub fingerprint: Option<String>,
}

/// 插件安装器
pub struct PluginInstaller;

impl PluginInstaller {
    /// 从本地文件安装
    pub async fn install_from_file(path: &PathBuf) -> Result<Vec<u8>> {
        let bytes = tokio::fs::read(path).await?;
        Self::validate_wasm(&bytes)?;
        Ok(bytes)
    }

    /// 从 GitHub 安装
    pub async fn install_from_github(owner: &str, repo: &str, version: &str) -> Result<Vec<u8>> {
        let url = format!(
            "https://github.com/{}/{}/releases/download/{}/plugin.wasm",
            owner, repo, version
        );

        let response = reqwest::get(&url).await?;
        if !response.status().is_success() {
            anyhow::bail!("Failed to download from GitHub: HTTP {}", response.status());
        }
        let bytes = response.bytes().await?.to_vec();

        Self::validate_wasm(&bytes)?;
        Ok(bytes)
    }

    /// 从远程 URL 安装
    pub async fn install_from_url(url: &str) -> Result<Vec<u8>> {
        let response = reqwest::get(url).await?;
        if !response.status().is_success() {
            anyhow::bail!("Failed to download from URL: HTTP {}", response.status());
        }
        let bytes = response.bytes().await?.to_vec();

        Self::validate_wasm(&bytes)?;
        Ok(bytes)
    }

    /// 验证 WASM 模块
    fn validate_wasm(bytes: &[u8]) -> Result<()> {
        // 检查 WASM magic number
        if bytes.len() < 8 {
            anyhow::bail!("Invalid WASM file: too short");
        }

        if &bytes[0..4] != b"\0asm" {
            anyhow::bail!("Invalid WASM file: bad magic number");
        }

        Ok(())
    }

    /// 获取插件安装目录
    pub fn plugin_dir() -> PathBuf {
        paths::plugins_dir()
    }

    /// 确保插件目录存在
    pub async fn ensure_plugin_dir() -> Result<PathBuf> {
        let dir = Self::plugin_dir();
        fs::create_dir_all(&dir).await?;
        Ok(dir)
    }

    /// 从 WASM 自定义段提取清单
    ///
    /// WASM 模块可以将清单嵌入为自定义段（custom section）
    /// 段名通常为 "manifest" 或 "plugin_manifest"
    pub fn extract_manifest_from_wasm(bytes: &[u8]) -> Result<Option<PluginManifest>> {
        // WASM 模块结构:
        // - magic: 4 bytes (\0asm)
        // - version: 4 bytes (1)
        // - sections: 变长

        if bytes.len() < 8 {
            return Ok(None);
        }

        // 检查 magic number
        if &bytes[0..4] != b"\0asm" {
            return Ok(None);
        }

        // 版本号 (小端序)
        let version = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        if version != 1 {
            tracing::warn!("Unsupported WASM version: {}", version);
            return Ok(None);
        }

        // 解析段
        let mut offset = 8;
        while offset < bytes.len() {
            // 段 ID (1 byte)
            let section_id = bytes[offset];
            offset += 1;

            // 段大小 (LEB128 编码)
            let (size, bytes_read) = Self::read_leb128(&bytes[offset..])?;
            offset += bytes_read;

            // 自定义段 ID = 0
            if section_id == 0 {
                // 自定义段格式: name_len (LEB128) + name + content
                let (name_len, name_len_bytes) = Self::read_leb128(&bytes[offset..])?;
                offset += name_len_bytes;

                let name = std::str::from_utf8(&bytes[offset..offset + name_len as usize])
                    .map_err(|_| anyhow::anyhow!("Invalid custom section name"))?;
                offset += name_len as usize;

                // 检查是否是清单段
                if name == "manifest" || name == "plugin_manifest" || name == "agw_manifest" {
                    let content_len = size as usize - name_len_bytes - name_len as usize;
                    let yaml_content = std::str::from_utf8(&bytes[offset..offset + content_len])
                        .map_err(|_| anyhow::anyhow!("Invalid manifest content"))?;

                    let manifest = PluginManifest::from_yaml(yaml_content)?;
                    return Ok(Some(manifest));
                }
            }

            // 跳到下一个段
            offset += size as usize;
        }

        Ok(None)
    }

    /// 读取 LEB128 编码的无符号整数
    fn read_leb128(bytes: &[u8]) -> Result<(u64, usize)> {
        let mut result = 0u64;
        let mut shift = 0;
        let mut offset = 0;

        loop {
            if offset >= bytes.len() {
                anyhow::bail!("Incomplete LEB128 encoding");
            }

            let byte = bytes[offset];
            offset += 1;

            result |= ((byte & 0x7F) as u64) << shift;
            shift += 7;

            if byte & 0x80 == 0 {
                break;
            }

            if shift >= 64 {
                anyhow::bail!("LEB128 overflow");
            }
        }

        Ok((result, offset))
    }

    /// 保存 WASM 文件到插件目录
    pub async fn save_wasm(plugin_id: &str, bytes: &[u8]) -> Result<PathBuf> {
        let dir = Self::ensure_plugin_dir().await?;
        let wasm_path = dir.join(format!("{}.wasm", plugin_id));
        fs::write(&wasm_path, bytes).await?;
        Ok(wasm_path)
    }

    /// 删除 WASM 文件
    pub async fn delete_wasm(wasm_path: &PathBuf) -> Result<()> {
        if fs::try_exists(wasm_path).await? {
            fs::remove_file(wasm_path).await?;
        }
        Ok(())
    }

    /// 从外部清单文件加载
    pub async fn load_external_manifest(plugin_id: &str) -> Result<Option<PluginManifest>> {
        let dir = Self::plugin_dir();
        let manifest_path = dir.join(format!("{}.yaml", plugin_id));

        if fs::try_exists(&manifest_path).await? {
            let manifest = PluginManifest::load_from_file(&manifest_path).await?;
            Ok(Some(manifest))
        } else {
            Ok(None)
        }
    }

    /// 保存清单文件
    pub async fn save_manifest(plugin_id: &str, manifest: &PluginManifest) -> Result<PathBuf> {
        let dir = Self::ensure_plugin_dir().await?;
        let manifest_path = dir.join(format!("{}.yaml", plugin_id));
        manifest.save_to_file(&manifest_path).await?;
        Ok(manifest_path)
    }

    /// 从 GitHub API 获取最新 release 版本号
    ///
    /// 调用 `https://api.github.com/repos/{owner}/{repo}/releases/latest`
    /// 返回 tag_name (如 "v1.0.0")
    pub async fn fetch_github_latest_version(owner: &str, repo: &str) -> Result<String> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/releases/latest",
            owner, repo
        );

        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .header("Accept", "application/vnd.github.v3+json")
            .header("User-Agent", "agent-gateway-plugin-installer")
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!(
                "Failed to fetch latest release from GitHub: HTTP {} for {}/{}",
                response.status(),
                owner,
                repo
            );
        }

        let json: serde_json::Value = response.json().await?;
        let tag_name = json["tag_name"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid GitHub response: missing tag_name for {}/{}", owner, repo))?;

        Ok(tag_name.to_string())
    }

    /// 检查插件是否已安装
    ///
    /// 检查插件目录中是否存在对应的 WASM 文件
    pub async fn is_installed(plugin_id: &str) -> bool {
        let wasm_path = Self::plugin_dir().join(format!("{}.wasm", plugin_id));
        fs::try_exists(&wasm_path).await.unwrap_or(false)
    }

    /// 获取已安装插件的详细信息
    ///
    /// 包括清单（如果存在）和 WASM 路径
    pub async fn get_installed_info(plugin_id: &str) -> Result<Option<InstalledPluginInfo>> {
        let wasm_path = Self::plugin_dir().join(format!("{}.wasm", plugin_id));

        if !fs::try_exists(&wasm_path).await? {
            return Ok(None);
        }

        let manifest_path = Self::plugin_dir().join(format!("{}.yaml", plugin_id));
        let manifest = if fs::try_exists(&manifest_path).await? {
            Some(PluginManifest::load_from_file(&manifest_path).await?)
        } else {
            None
        };

        // 计算指纹
        let wasm_bytes = fs::read(&wasm_path).await?;
        let fingerprint = Some(Self::compute_wasm_fingerprint(&wasm_bytes));

        Ok(Some(InstalledPluginInfo {
            id: plugin_id.to_string(),
            manifest,
            wasm_path,
            manifest_path: if fs::try_exists(&manifest_path).await? {
                Some(manifest_path)
            } else {
                None
            },
            fingerprint,
        }))
    }

    /// 备份已安装插件
    ///
    /// 将 WASM 和清单文件复制到 `.backup` 子目录
    /// 返回备份目录路径
    pub async fn backup_plugin(plugin_id: &str) -> Result<PathBuf> {
        let plugin_dir = Self::plugin_dir();
        let backup_dir = plugin_dir.join(".backup").join(plugin_id);
        fs::create_dir_all(&backup_dir).await?;

        let wasm_path = plugin_dir.join(format!("{}.wasm", plugin_id));
        let manifest_path = plugin_dir.join(format!("{}.yaml", plugin_id));

        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");

        if fs::try_exists(&wasm_path).await? {
            let backup_wasm = backup_dir.join(format!("{}.wasm", timestamp));
            fs::copy(&wasm_path, &backup_wasm).await?;
        }

        if fs::try_exists(&manifest_path).await? {
            let backup_manifest = backup_dir.join(format!("{}.yaml", timestamp));
            fs::copy(&manifest_path, &backup_manifest).await?;
        }

        tracing::debug!("Plugin {} backed up to {:?}", plugin_id, backup_dir);
        Ok(backup_dir)
    }

    /// 删除插件的备份文件
    pub async fn remove_backup(plugin_id: &str) -> Result<()> {
        let backup_dir = Self::plugin_dir().join(".backup").join(plugin_id);
        if fs::try_exists(&backup_dir).await? {
            fs::remove_dir_all(&backup_dir).await?;
        }
        Ok(())
    }

    /// 计算 WASM 文件指纹
    ///
    /// 使用 rustc-hash 的 FxHasher 计算 64 位哈希，转为 hex 字符串
    pub fn compute_wasm_fingerprint(bytes: &[u8]) -> String {
        use std::hash::Hasher;
        let mut hasher = rustc_hash::FxHasher::default();
        hasher.write(bytes);
        format!("{:016x}", hasher.finish())
    }

    /// 完整卸载插件文件
    ///
    /// 删除 WASM 文件、清单文件和任何相关元数据
    pub async fn uninstall_plugin_files(plugin_id: &str) -> Result<()> {
        let plugin_dir = Self::plugin_dir();
        let wasm_path = plugin_dir.join(format!("{}.wasm", plugin_id));
        let manifest_path = plugin_dir.join(format!("{}.yaml", plugin_id));

        if fs::try_exists(&wasm_path).await? {
            fs::remove_file(&wasm_path).await?;
        }

        if fs::try_exists(&manifest_path).await? {
            fs::remove_file(&manifest_path).await?;
        }

        tracing::debug!("Plugin files removed for {}", plugin_id);
        Ok(())
    }

    /// 检查插件依赖是否满足
    ///
    /// 遍历 manifest 中的 dependencies，检查每个依赖插件是否已安装
    /// 返回未满足的依赖列表
    pub async fn check_dependencies(manifest: &PluginManifest) -> Result<Vec<String>> {
        let mut missing = Vec::new();

        for dep in &manifest.dependencies {
            let dep_info = Self::get_installed_info(&dep.id).await?;
            match dep_info {
                Some(info) => {
                    // 如果清单存在，检查版本范围
                    if let Some(installed_manifest) = info.manifest {
                        if !Self::version_satisfies(&installed_manifest.version, &dep.version_range) {
                            missing.push(format!(
                                "{}: installed version {} does not satisfy {}",
                                dep.id, installed_manifest.version, dep.version_range
                            ));
                        }
                    }
                    // 如果没有清单，假设已安装但版本未知，视为满足
                }
                None => {
                    missing.push(format!(
                        "{}: required version {} is not installed",
                        dep.id, dep.version_range
                    ));
                }
            }
        }

        Ok(missing)
    }

    /// 简单的语义化版本范围检查
    ///
    /// 支持格式:
    /// - `>=1.0.0`
    /// - `^1.0.0` (兼容 1.x.x)
    /// - `~1.0.0` (兼容 1.0.x)
    /// - `1.0.0` (精确匹配)
    fn version_satisfies(version: &str, range: &str) -> bool {
        let range = range.trim();

        // 精确匹配
        if !range.starts_with('>') && !range.starts_with('<') && !range.starts_with('^') && !range.starts_with('~') {
            return version == range;
        }

        let Ok(v) = Self::parse_semver(version) else { return false };

        // ^1.0.0 -> >=1.0.0 && <2.0.0
        if range.starts_with('^') {
            let req = range.strip_prefix('^').unwrap_or(range);
            let Ok(req_v) = Self::parse_semver(req) else { return false };
            return v.0 == req_v.0 && v >= req_v;
        }

        // ~1.0.0 -> >=1.0.0 && <1.1.0
        if range.starts_with('~') {
            let req = range.strip_prefix('~').unwrap_or(range);
            let Ok(req_v) = Self::parse_semver(req) else { return false };
            return v.0 == req_v.0 && v.1 == req_v.1 && v >= req_v;
        }

        // >=1.0.0, <2.0.0, etc.
        if range.starts_with(">=") {
            let req = range.strip_prefix(">=").unwrap_or(range);
            let Ok(req_v) = Self::parse_semver(req) else { return false };
            return v >= req_v;
        }

        if range.starts_with(">") {
            let req = range.strip_prefix(">").unwrap_or(range);
            let Ok(req_v) = Self::parse_semver(req) else { return false };
            return v > req_v;
        }

        if range.starts_with("<=") {
            let req = range.strip_prefix("<=").unwrap_or(range);
            let Ok(req_v) = Self::parse_semver(req) else { return false };
            return v <= req_v;
        }

        if range.starts_with("<") {
            let req = range.strip_prefix("<").unwrap_or(range);
            let Ok(req_v) = Self::parse_semver(req) else { return false };
            return v < req_v;
        }

        false
    }

    /// 解析语义化版本字符串为 (major, minor, patch)
    fn parse_semver(version: &str) -> Result<(u32, u32, u32)> {
        let version = version.trim_start_matches('v');
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() < 3 {
            anyhow::bail!("Invalid semver format: {}", version);
        }
        let major = parts[0].parse::<u32>()?;
        let minor = parts[1].parse::<u32>()?;
        // patch 可能包含预发布标识，如 "1.0.0-beta"
        let patch_str = parts[2];
        let patch = patch_str.split('-').next().unwrap_or(patch_str).parse::<u32>()?;
        Ok((major, minor, patch))
    }

    /// 解析安装源
    ///
    /// 支持格式:
    /// - 本地文件: `file://path/to/plugin.wasm` 或直接路径
    /// - GitHub: `github://owner/repo@version`
    /// - 远程 URL: `https://...`
    pub fn parse_source(source: &str) -> PluginSource {
        // 本地文件
        if source.starts_with("file://") {
            let path = source.strip_prefix("file://").unwrap_or(source);
            return PluginSource::Local(PathBuf::from(path));
        }

        // 直接路径 (检查是否是文件路径)
        if !source.contains("://") && (source.ends_with(".wasm") || source.contains('/') || source.contains('\\')) {
            return PluginSource::Local(PathBuf::from(source));
        }

        // GitHub
        if source.starts_with("github://") {
            let rest = source.strip_prefix("github://").unwrap_or(source);
            // 格式: owner/repo@version 或 owner/repo
            let parts: Vec<&str> = rest.split('@').collect();
            if parts.len() >= 2 {
                let owner_repo: Vec<&str> = parts[0].split('/').collect();
                if owner_repo.len() == 2 {
                    return PluginSource::GitHub {
                        owner: owner_repo[0].to_string(),
                        repo: owner_repo[1].to_string(),
                        version: parts[1].to_string(),
                    };
                }
            } else if parts.len() == 1 {
                let owner_repo: Vec<&str> = parts[0].split('/').collect();
                if owner_repo.len() == 2 {
                    return PluginSource::GitHub {
                        owner: owner_repo[0].to_string(),
                        repo: owner_repo[1].to_string(),
                        version: "latest".to_string(),
                    };
                }
            }
        }

        // 远程 URL
        if source.starts_with("http://") || source.starts_with("https://") {
            return PluginSource::Url(source.to_string());
        }

        // 默认作为路径处理
        PluginSource::Local(PathBuf::from(source))
    }
}

/// 插件来源
#[derive(Debug, Clone)]
pub enum PluginSource {
    /// 本地文件
    Local(PathBuf),
    /// GitHub 仓库
    GitHub {
        owner: String,
        repo: String,
        version: String,
    },
    /// 远程 URL
    Url(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_wasm_valid() {
        // Valid WASM magic number + version
        let valid_wasm = b"\0asm\x01\x00\x00\x00";
        let result = PluginInstaller::validate_wasm(valid_wasm);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_wasm_invalid_magic() {
        let invalid_wasm = b"INVALID!!";
        let result = PluginInstaller::validate_wasm(invalid_wasm);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("bad magic number"));
    }

    #[test]
    fn test_validate_wasm_too_short() {
        let short_wasm = b"\0asm";
        let result = PluginInstaller::validate_wasm(short_wasm);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too short"));
    }

    #[test]
    fn test_parse_source_local_file() {
        let source = "/path/to/plugin.wasm";
        let parsed = PluginInstaller::parse_source(source);
        assert!(matches!(parsed, PluginSource::Local(_)));
    }

    #[test]
    fn test_parse_source_file_uri() {
        let source = "file:///path/to/plugin.wasm";
        let parsed = PluginInstaller::parse_source(source);
        match parsed {
            PluginSource::Local(path) => assert_eq!(path.to_str(), Some("/path/to/plugin.wasm")),
            _ => panic!("Expected Local source"),
        }
    }

    #[test]
    fn test_parse_source_github_with_version() {
        let source = "github://owner/repo@v1.0.0";
        let parsed = PluginInstaller::parse_source(source);
        match parsed {
            PluginSource::GitHub { owner, repo, version } => {
                assert_eq!(owner, "owner");
                assert_eq!(repo, "repo");
                assert_eq!(version, "v1.0.0");
            }
            _ => panic!("Expected GitHub source"),
        }
    }

    #[test]
    fn test_parse_source_github_latest() {
        let source = "github://owner/repo";
        let parsed = PluginInstaller::parse_source(source);
        match parsed {
            PluginSource::GitHub { owner, repo, version } => {
                assert_eq!(owner, "owner");
                assert_eq!(repo, "repo");
                assert_eq!(version, "latest");
            }
            _ => panic!("Expected GitHub source"),
        }
    }

    #[test]
    fn test_parse_source_url() {
        let source = "https://example.com/plugin.wasm";
        let parsed = PluginInstaller::parse_source(source);
        match parsed {
            PluginSource::Url(url) => assert_eq!(url, source),
            _ => panic!("Expected Url source"),
        }
    }

    #[test]
    fn test_read_leb128() {
        // LEB128 encoding of 128 (0x80)
        // = 0x80 0x01 (since 128 requires continuation bit)
        let bytes = [0x80, 0x01];
        let (value, read) = PluginInstaller::read_leb128(&bytes).unwrap();
        assert_eq!(value, 128);
        assert_eq!(read, 2);

        // LEB128 encoding of 127 (fits in one byte)
        let bytes = [0x7F];
        let (value, read) = PluginInstaller::read_leb128(&bytes).unwrap();
        assert_eq!(value, 127);
        assert_eq!(read, 1);
    }

    #[test]
    fn test_extract_manifest_from_wasm_no_manifest() {
        // Minimal valid WASM module (magic + version)
        let wasm = b"\0asm\x01\x00\x00\x00";
        let result = PluginInstaller::extract_manifest_from_wasm(wasm);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_extract_manifest_from_wasm_invalid() {
        let wasm = b"invalid wasm content";
        let result = PluginInstaller::extract_manifest_from_wasm(wasm);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_plugin_dir() {
        let dir = PluginInstaller::plugin_dir();
        // 应该包含 plugins 和 .agent-gateway 或自定义路径
        assert!(dir.to_string_lossy().contains("plugins"));
    }
}