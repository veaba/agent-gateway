//! 请求日志存储

use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::time::SystemTime;
use tokio::io::AsyncWriteExt;
use flate2::write::GzEncoder;
use flate2::read::GzDecoder;

/// 请求日志条目
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestLogEntry {
    /// 日志 ID
    pub id: String,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
    /// 日志级别
    pub level: LogLevel,
    /// 日志消息
    pub message: String,
    /// 目标模块
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    /// 关联的套餐 ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan_id: Option<String>,
    /// 关联的请求 ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    /// Agent ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    /// 模型 ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_id: Option<String>,
    /// HTTP 状态码
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_code: Option<u16>,
    /// 响应延迟 (ms)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u64>,
    /// 错误信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// 日志级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warn => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR"),
        }
    }
}

impl From<tracing::Level> for LogLevel {
    fn from(level: tracing::Level) -> Self {
        match level {
            tracing::Level::TRACE | tracing::Level::DEBUG => LogLevel::Debug,
            tracing::Level::INFO => LogLevel::Info,
            tracing::Level::WARN => LogLevel::Warn,
            tracing::Level::ERROR => LogLevel::Error,
        }
    }
}

/// 请求日志存储
pub struct RequestLogStore {
    /// 日志目录
    log_dir: PathBuf,
    /// 当前日志文件路径
    current_file: Arc<RwLock<PathBuf>>,
    /// 最大文件大小 (bytes)
    max_file_size: u64,
    /// 最大文件数量
    max_files: usize,
    /// 是否启用压缩
    compress: bool,
    /// 写入锁 (用于批量写入同步)
    write_lock: Arc<RwLock<()>>,
}

impl RequestLogStore {
    /// 创建请求日志存储
    pub fn new(log_dir: PathBuf) -> Self {
        let current_file = log_dir.join("requests.log");

        Self {
            log_dir,
            current_file: Arc::new(RwLock::new(current_file)),
            max_file_size: 10 * 1024 * 1024, // 10MB
            max_files: 10,
            compress: true,
            write_lock: Arc::new(RwLock::new(())),
        }
    }

    /// 创建请求日志存储 (带配置)
    pub fn with_config(log_dir: PathBuf, max_file_size: u64, max_files: usize, compress: bool) -> Self {
        let current_file = log_dir.join("requests.log");

        Self {
            log_dir,
            current_file: Arc::new(RwLock::new(current_file)),
            max_file_size,
            max_files,
            compress,
            write_lock: Arc::new(RwLock::new(())),
        }
    }

    /// 同步写入 (用于 tracing Layer)
    pub fn write_sync(&self, entry: &RequestLogEntry) -> anyhow::Result<()> {
        let guard = self.write_lock.blocking_read();

        let current_file = self.current_file.blocking_read();

        // 检查文件大小，如果超过限制则轮转
        if let Ok(metadata) = std::fs::metadata(&*current_file) {
            if metadata.len() > self.max_file_size {
                drop(current_file);
                drop(guard);
                // 使用同步方式轮转
                self.rotate_file_sync()?;
                let current_file = self.current_file.blocking_read();

                let json = serde_json::to_string(entry)?;
                let line = format!("{}\n", json);

                let mut file = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&*current_file)?;
                file.write_all(line.as_bytes())?;

                return Ok(());
            }
        }

        // 追加写入
        let json = serde_json::to_string(entry)?;
        let line = format!("{}\n", json);

        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&*current_file)?;
        file.write_all(line.as_bytes())?;

        Ok(())
    }

    /// 异步写入
    pub async fn write(&self, entry: &RequestLogEntry) -> anyhow::Result<()> {
        let _guard = self.write_lock.write().await;
        let file = self.current_file.read().await.clone();

        // 检查文件大小，如果超过限制则轮转
        if let Ok(metadata) = tokio::fs::metadata(&file).await {
            if metadata.len() > self.max_file_size {
                self.rotate_file().await?;
            }
        }

        // 追加写入
        let json = serde_json::to_string(entry)?;
        let line = format!("{}\n", json);

        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file)
            .await?;
        file.write_all(line.as_bytes()).await?;

        Ok(())
    }

    /// 批量写入
    pub async fn write_batch(&self, entries: &[RequestLogEntry]) -> anyhow::Result<()> {
        let _guard = self.write_lock.write().await;
        let file = self.current_file.read().await.clone();

        // 检查累积大小
        let batch_size: u64 = entries.iter()
            .map(|e| serde_json::to_string(e).map(|s| s.len() as u64 + 1).unwrap_or(0))
            .sum();

        if let Ok(metadata) = tokio::fs::metadata(&file).await {
            if metadata.len() + batch_size > self.max_file_size {
                self.rotate_file().await?;
            }
        }

        // 批量追加
        let lines: String = entries.iter()
            .map(|e| serde_json::to_string(e).map(|s| format!("{}\n", s)).unwrap_or_default())
            .collect();

        tokio::fs::write(&file, lines).await?;

        Ok(())
    }

    /// 同步轮转日志文件
    fn rotate_file_sync(&self) -> anyhow::Result<()> {
        let _guard = self.write_lock.blocking_write();

        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let rotated_name = format!("requests_{}.log", timestamp);
        let rotated_path = self.log_dir.join(&rotated_name);

        let current_file = self.current_file.blocking_read().clone();

        // 重命名当前文件
        if current_file.exists() {
            std::fs::rename(&current_file, &rotated_path)?;

            // 压缩轮转后的文件
            if self.compress {
                let gz_path = rotated_path.with_extension("log.gz");
                let file = File::open(&rotated_path)?;
                let mut encoder = GzEncoder::new(File::create(&gz_path)?, flate2::Compression::default());
                std::io::copy(&mut &file, &mut encoder)?;
                encoder.finish()?;
                let _ = std::fs::remove_file(&rotated_path);
            }
        }

        // 创建新文件
        std::fs::write(&current_file, "")?;

        // 清理旧文件
        self.cleanup_old_files_sync();

        Ok(())
    }

    /// 轮转日志文件
    async fn rotate_file(&self) -> anyhow::Result<()> {
        let current_file = self.current_file.write().await;
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let rotated_name = format!("requests_{}.log", timestamp);
        let rotated_path = self.log_dir.join(rotated_name);

        // 重命名当前文件
        if current_file.exists() {
            tokio::fs::rename(&*current_file, &rotated_path).await?;

            // 压缩轮转后的文件
            if self.compress {
                let gz_path = rotated_path.with_extension("log.gz");
                let file = File::open(&rotated_path)?;
                let mut encoder = GzEncoder::new(File::create(&gz_path)?, flate2::Compression::default());
                let mut decoder = GzDecoder::new(file);
                std::io::copy(&mut decoder, &mut encoder)?;
                encoder.finish()?;
                tokio::fs::remove_file(&rotated_path).await?;
            }
        }

        // 创建新文件
        tokio::fs::write(&*current_file, "").await?;

        // 清理旧文件
        self.cleanup_old_files().await?;

        Ok(())
    }

    /// 同步清理旧日志文件
    fn cleanup_old_files_sync(&self) {
        let mut files: Vec<(PathBuf, SystemTime)> = Vec::new();

        if let Ok(entries) = std::fs::read_dir(&self.log_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map(|e| e == "log" || e == "gz").unwrap_or(false) {
                    if let Ok(metadata) = entry.metadata() {
                        if let Ok(modified) = metadata.modified() {
                            files.push((path, modified));
                        }
                    }
                }
            }
        }

        // 按时间排序
        files.sort_by(|a, b| b.1.cmp(&a.1));

        // 删除超出数量的文件
        for (path, _) in files.iter().skip(self.max_files) {
            let _ = std::fs::remove_file(path);
        }
    }

    /// 清理旧日志文件
    async fn cleanup_old_files(&self) -> anyhow::Result<()> {
        let mut files: Vec<(PathBuf, std::time::SystemTime)> = Vec::new();

        let mut entries = tokio::fs::read_dir(&self.log_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().map(|e| e == "log" || e == "gz").unwrap_or(false) {
                let metadata = entry.metadata().await?;
                let modified = metadata.modified()?;
                files.push((path, modified));
            }
        }

        // 按时间排序
        files.sort_by(|a, b| b.1.cmp(&a.1));

        // 删除超出数量的文件
        for (path, _) in files.iter().skip(self.max_files) {
            tokio::fs::remove_file(path).await?;
        }

        Ok(())
    }

    /// 读取日志 (支持压缩文件)
    pub async fn read(
        &self,
        limit: usize,
        offset: usize,
        level_filter: Option<LogLevel>,
        plan_id_filter: Option<String>,
    ) -> anyhow::Result<Vec<RequestLogEntry>> {
        let mut all_entries: Vec<RequestLogEntry> = Vec::new();

        // 收集所有日志文件
        let mut files: Vec<PathBuf> = Vec::new();
        let mut entries = tokio::fs::read_dir(&self.log_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().map(|e| e == "log" || e == "gz").unwrap_or(false) {
                files.push(path);
            }
        }

        // 添加当前文件
        let current = self.current_file.read().await.clone();
        if !files.contains(&current) && current.exists() {
            files.push(current);
        }

        // 按文件名排序（最新的先读）
        files.sort_by(|a, b| b.file_name().cmp(&a.file_name()));

        // 从每个文件读取
        for file in files {
            if let Ok(content) = self.read_file_content(&file).await {
                for line in content.lines() {
                    if line.is_empty() {
                        continue;
                    }
                    if let Ok(entry) = serde_json::from_str::<RequestLogEntry>(line) {
                        // 应用过滤器
                        if let Some(level) = level_filter {
                            if entry.level != level {
                                continue;
                            }
                        }
                        if let Some(plan_id) = &plan_id_filter {
                            if entry.plan_id.as_ref() != Some(plan_id) {
                                continue;
                            }
                        }
                        all_entries.push(entry);
                    }
                }
            }
        }

        // 按时间排序（最新的在前）
        all_entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        // 应用 offset 和 limit
        let start = offset.min(all_entries.len());
        let end = (offset + limit).min(all_entries.len());

        Ok(all_entries[start..end].to_vec())
    }

    /// 读取文件内容 (支持压缩)
    async fn read_file_content(&self, file: &PathBuf) -> anyhow::Result<String> {
        let extension = file.extension().and_then(|e| e.to_str()).unwrap_or("");

        match extension {
            "gz" => {
                let bytes = tokio::fs::read(file).await?;
                let mut decoder = GzDecoder::new(&bytes[..]);
                let mut content = String::new();
                decoder.read_to_string(&mut content)?;
                Ok(content)
            }
            _ => {
                tokio::fs::read_to_string(file).await.map_err(Into::into)
            }
        }
    }

    /// 导出为 CSV
    pub async fn export_csv(&self, output_path: &PathBuf) -> anyhow::Result<()> {
        let entries = self.read(usize::MAX, 0, None, None).await?;

        let mut file = File::create(output_path)?;
        writeln!(file, "id,timestamp,level,message,target,plan_id,request_id,agent_id,model_id,status_code,latency_ms,error")?;

        for entry in entries {
            writeln!(
                file,
                "{},{},{},{},{},{},{},{},{},{},{},{}",
                entry.id,
                entry.timestamp.to_rfc3339(),
                entry.level,
                entry.message.replace(',', ";"),
                entry.target.as_deref().unwrap_or(""),
                entry.plan_id.as_deref().unwrap_or(""),
                entry.request_id.as_deref().unwrap_or(""),
                entry.agent_id.as_deref().unwrap_or(""),
                entry.model_id.as_deref().unwrap_or(""),
                entry.status_code.map(|s| s.to_string()).unwrap_or_default(),
                entry.latency_ms.map(|l| l.to_string()).unwrap_or_default(),
                entry.error.as_deref().unwrap_or("")
            )?;
        }

        Ok(())
    }

    /// 获取日志总数
    pub async fn count(
        &self,
        level_filter: Option<LogLevel>,
        plan_id_filter: Option<String>,
    ) -> anyhow::Result<usize> {
        let entries = self.read(usize::MAX, 0, level_filter, plan_id_filter).await?;
        Ok(entries.len())
    }

    /// 获取日志目录
    pub fn log_dir(&self) -> &PathBuf {
        &self.log_dir
    }

    /// 设置最大文件大小
    pub fn set_max_file_size(&mut self, size: u64) {
        self.max_file_size = size;
    }

    /// 设置最大文件数量
    pub fn set_max_files(&mut self, count: usize) {
        self.max_files = count;
    }

    /// 获取日志文件列表
    pub async fn get_log_files(&self) -> anyhow::Result<Vec<LogFileInfo>> {
        let mut files: Vec<LogFileInfo> = Vec::new();

        let mut entries = tokio::fs::read_dir(&self.log_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().map(|e| e == "log" || e == "gz").unwrap_or(false) {
                let metadata = entry.metadata().await?;
                let file_name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                let is_compressed = path.extension().and_then(|e| e.to_str()) == Some("gz");
                let size = if is_compressed {
                    metadata.len()
                } else {
                    metadata.len()
                };

                files.push(LogFileInfo {
                    name: file_name,
                    path: path.display().to_string(),
                    size_bytes: size,
                    is_compressed,
                    modified_at: metadata.modified()?.into(),
                });
            }
        }

        // 按修改时间排序
        files.sort_by(|a, b| b.modified_at.cmp(&a.modified_at));

        Ok(files)
    }
}

/// 日志文件信息
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LogFileInfo {
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
    pub is_compressed: bool,
    pub modified_at: chrono::DateTime<chrono::Utc>,
}
