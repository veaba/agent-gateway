//! log 命令

use anyhow::Result;
use clap::Parser;

/// 日志管理命令
#[derive(Parser, Debug)]
pub struct LogCommand {
    /// 查看日志
    #[arg(long)]
    pub view: bool,
    /// 清理日志
    #[arg(long)]
    pub clean: bool,
    /// 日志文件路径
    #[arg(long)]
    pub path: Option<String>,
    /// 显示最近 N 条
    #[arg(long, default_value = "50")]
    pub tail: usize,
    /// 按级别过滤 (debug, info, warn, error)
    #[arg(long)]
    pub level: Option<String>,
}

impl LogCommand {
    pub async fn run(&self) -> Result<()> {
        let log_dir = match &self.path {
            Some(p) => std::path::PathBuf::from(p),
            None => {
                dirs::data_local_dir()
                    .ok_or_else(|| anyhow::anyhow!("Cannot find data directory"))?
                    .join("agent-gateway")
                    .join("logs")
            }
        };

        if self.view {
            self.handle_view(&log_dir, self.tail, self.level.as_deref()).await?;
        } else if self.clean {
            self.handle_clean(&log_dir).await?;
        } else {
            println!("Use --view to view logs, or --clean to clean logs");
            println!("Log directory: {}", log_dir.display());
        }
        Ok(())
    }

    /// 查看日志
    async fn handle_view(
        &self,
        log_dir: &std::path::Path,
        tail: usize,
        level_filter: Option<&str>,
    ) -> Result<()> {
        if !log_dir.exists() {
            println!("Log directory does not exist: {}", log_dir.display());
            return Ok(());
        }

        let mut entries = tokio::fs::read_dir(log_dir).await?;
        let mut files: Vec<(std::path::PathBuf, tokio::fs::DirEntry)> = Vec::new();

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().map(|e| e == "log").unwrap_or(false) {
                let _meta = entry.metadata().await?;
                files.push((path, entry));
            }
        }

        // 按修改时间排序，最新的在前
        let mut files_with_mtime: Vec<_> = Vec::new();
        for (path, entry) in files {
            if let Ok(_meta) = entry.metadata().await {
                if let Ok(mtime) = _meta.modified() {
                    files_with_mtime.push((path, mtime));
                }
            }
        }
        files_with_mtime.sort_by(|a, b| b.1.cmp(&a.1));

        if files_with_mtime.is_empty() {
            println!("No log files found in: {}", log_dir.display());
            return Ok(());
        }

        let mut lines: Vec<String> = Vec::new();

        for (file_path, _) in files_with_mtime {
            if let Ok(content) = tokio::fs::read_to_string(&file_path).await {
                for line in content.lines() {
                    if line.is_empty() {
                        continue;
                    }
                    // 简单的级别过滤
                    if let Some(level) = level_filter {
                        let level_upper = level.to_uppercase();
                        if !line.contains(&level_upper) {
                            continue;
                        }
                    }
                    lines.push(line.to_string());
                }
            }
        }

        if lines.is_empty() {
            println!("No log entries found.");
            return Ok(());
        }

        // 取最后 N 条
        let start = lines.len().saturating_sub(tail);
        let display_lines = &lines[start..];

        println!("Showing last {} log entries:\n", display_lines.len());
        for line in display_lines {
            println!("{}", line);
        }

        Ok(())
    }

    /// 清理日志
    async fn handle_clean(&self, log_dir: &std::path::Path) -> Result<()> {
        if !log_dir.exists() {
            println!("Log directory does not exist: {}", log_dir.display());
            return Ok(());
        }

        let mut entries = tokio::fs::read_dir(log_dir).await?;
        let mut count = 0;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().map(|e| e == "log").unwrap_or(false) {
                tokio::fs::remove_file(&path).await?;
                count += 1;
            }
        }

        println!("✅ Cleaned {} log file(s) from: {}", count, log_dir.display());
        Ok(())
    }
}
