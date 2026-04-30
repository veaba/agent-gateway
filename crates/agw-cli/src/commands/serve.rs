//! serve 命令

use std::path::PathBuf;
use std::fs;

use anyhow::Result;
use clap::Parser;

use agw_core::serve;

/// 启动网关服务
#[derive(Parser, Debug)]
pub struct ServeCommand {
    /// 后台运行
    #[arg(long)]
    pub daemon: bool,
    /// 监听地址
    #[arg(long, default_value = "127.0.0.1:8080")]
    pub listen: String,
    /// PID 文件路径
    #[arg(long)]
    pub pid_file: Option<String>,
}

impl ServeCommand {
    pub async fn run(&self) -> Result<()> {
        tracing::info!("Starting agent-gateway on {}", self.listen);

        let pid_file = self.pid_file.clone()
            .or_else(|| {
                dirs::data_local_dir()
                    .map(|d| d.join("agent-gateway").join("gateway.pid"))
                    .map(|p| p.to_string_lossy().to_string())
            });

        if self.daemon {
            self.run_daemon(&pid_file)?;
            return Ok(());
        }

        // 写入 PID 文件
        if let Some(ref path) = pid_file {
            self.write_pid_file(path)?;
        }

        tracing::info!("Gateway service started");

        // 启动网关
        serve(&self.listen).await?;

        // 清理 PID 文件
        if let Some(ref path) = pid_file {
            self.remove_pid_file(path);
        }

        tracing::info!("Gateway service stopped");
        Ok(())
    }

    /// 后台运行
    fn run_daemon(&self, pid_file: &Option<String>) -> Result<()> {
        tracing::info!("Starting gateway in daemon mode...");

        // 获取当前可执行文件路径
        let exe_path = std::env::current_exe()?;
        let exe_dir = exe_path.parent()
            .ok_or_else(|| anyhow::anyhow!("Cannot get executable directory"))?;

        // Windows: 使用 start 命令启动后台进程
        #[cfg(target_os = "windows")]
        {
            let mut cmd = std::process::Command::new("cmd");
            cmd.args([
                "/C",
                "start",
                "/B",
                "",
                &exe_path.to_string_lossy(),
                "serve",
                "--listen",
                &self.listen,
            ]);

            if let Some(ref path) = pid_file {
                cmd.args(["--pid-file", path]);
            }

            cmd.current_dir(exe_dir);
            cmd.spawn()?;
        }

        // Linux/macOS: 使用 nohup
        #[cfg(not(target_os = "windows"))]
        {
            let log_file = dirs::data_local_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("agent-gateway")
                .join("logs")
                .join("gateway.log");

            let mut cmd = std::process::Command::new("nohup");
            cmd.args([
                &exe_path.to_string_lossy(),
                "serve",
                "--listen",
                &self.listen,
            ]);

            if let Some(ref path) = pid_file {
                cmd.args(["--pid-file", path]);
            }

            cmd.arg(&log_file);
            cmd.current_dir(exe_dir);
            cmd.spawn()?;
        }

        tracing::info!("Gateway started in background");
        println!("Gateway started in background on {}", self.listen);
        Ok(())
    }

    /// 写入 PID 文件
    fn write_pid_file(&self, path: &str) -> Result<()> {
        let pid = std::process::id();
        let path = PathBuf::from(path);

        // 确保目录存在
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&path, pid.to_string())?;
        tracing::info!("Written PID {} to {}", pid, path.display());
        Ok(())
    }

    /// 删除 PID 文件
    fn remove_pid_file(&self, path: &str) {
        let path = PathBuf::from(path);
        if path.exists() {
            if let Err(e) = fs::remove_file(&path) {
                tracing::warn!("Failed to remove PID file: {}", e);
            }
        }
    }
}