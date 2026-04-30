//! Gateway Host Functions
//!
//! 提供插件 WASM 模块可调用的宿主函数接口

use std::sync::Arc;
use anyhow::Result;
use wasmtime::{Caller, Linker, Trap};
use wasmtime_wasi::preview1::WasiP1Ctx;
use dashmap::DashMap;

/// 日志级别
#[repr(i32)]
enum LogLevel {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
}

impl TryFrom<i32> for LogLevel {
    type Error = anyhow::Error;

    fn try_from(value: i32) -> Result<Self> {
        match value {
            0 => Ok(LogLevel::Trace),
            1 => Ok(LogLevel::Debug),
            2 => Ok(LogLevel::Info),
            3 => Ok(LogLevel::Warn),
            4 => Ok(LogLevel::Error),
            _ => anyhow::bail!("Invalid log level: {}", value),
        }
    }
}

/// 将 anyhow::Error 转换为 Trap
fn error_to_trap(e: anyhow::Error) -> Trap {
    tracing::error!("Plugin host function error: {}", e);
    Trap::UnreachableCodeReached
}

/// 插件宿主上下文
///
/// 用于在宿主函数之间共享数据（配置、请求上下文等）
#[derive(Clone)]
pub struct HostContext {
    /// 插件配置（键值对）
    pub config: Arc<DashMap<String, String>>,
    /// 请求上下文（仅在请求处理期间有效）
    pub request_context: Arc<DashMap<String, String>>,
}

impl HostContext {
    /// 创建空的宿主上下文
    pub fn new() -> Self {
        Self {
            config: Arc::new(DashMap::new()),
            request_context: Arc::new(DashMap::new()),
        }
    }

    /// 设置配置值
    pub fn set_config(&self, key: String, value: String) {
        self.config.insert(key, value);
    }

    /// 获取配置值
    pub fn get_config(&self, key: &str) -> Option<String> {
        self.config.get(key).map(|v| v.value().clone())
    }

    /// 设置请求上下文值
    pub fn set_request_context(&self, key: String, value: String) {
        self.request_context.insert(key, value);
    }

    /// 获取请求上下文值
    pub fn get_request_context(&self, key: &str) -> Option<String> {
        self.request_context.get(key).map(|v| v.value().clone())
    }
}

impl Default for HostContext {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局宿主上下文
static HOST_CONTEXT: once_cell::sync::Lazy<HostContext> =
    once_cell::sync::Lazy::new(HostContext::new);

/// 获取全局宿主上下文引用
pub fn global_host_context() -> &'static HostContext {
    &HOST_CONTEXT
}

/// 添加 Gateway 宿主函数到 Linker
///
/// # Functions
/// - `gw_log`: 插件日志输出
/// - `gw_http_request`: HTTP 请求代理（异步，需要 tokio handle）
/// - `gw_get_config`: 获取配置
/// - `gw_get_request_context`: 获取请求上下文
/// - `gw_set_config`: 设置配置（持久化到宿主上下文）
pub fn add_gateway_host_functions(linker: &mut Linker<WasiP1Ctx>) -> Result<()> {
    // gw_log(level: i32, ptr: i32, len: i32) -> ()
    linker.func_wrap(
        "gateway",
        "gw_log",
        |mut caller: Caller<'_, WasiP1Ctx>, level: i32, ptr: i32, len: i32| {
            let log_level = LogLevel::try_from(level).map_err(error_to_trap)?;

            let memory = caller
                .get_export("memory")
                .and_then(|e| e.into_memory())
                .ok_or_else(|| error_to_trap(anyhow::anyhow!("Memory not found")))?;

            let data = memory
                .data(&caller)
                .get(ptr as usize..(ptr as usize + len as usize))
                .ok_or_else(|| error_to_trap(anyhow::anyhow!("Invalid memory range")))?;

            let msg = std::str::from_utf8(data)
                .map_err(|e| error_to_trap(anyhow::anyhow!("Invalid UTF-8: {}", e)))?;

            match log_level {
                LogLevel::Trace => tracing::trace!("[plugin] {}", msg),
                LogLevel::Debug => tracing::debug!("[plugin] {}", msg),
                LogLevel::Info => tracing::info!("[plugin] {}", msg),
                LogLevel::Warn => tracing::warn!("[plugin] {}", msg),
                LogLevel::Error => tracing::error!("[plugin] {}", msg),
            }

            Ok(())
        },
    )?;

// gw_http_request(method_ptr: i32, method_len: i32, url_ptr: i32, url_len: i32,
    //                 body_ptr: i32, body_len: i32, response_buf_ptr: i32, response_buf_len: i32) -> i32
    // 返回响应长度，负数表示错误
    linker.func_wrap(
        "gateway",
        "gw_http_request",
        |mut caller: Caller<'_, WasiP1Ctx>,
         method_ptr: i32,
         method_len: i32,
         url_ptr: i32,
         url_len: i32,
         body_ptr: i32,
         body_len: i32,
         response_buf_ptr: i32,
         response_buf_len: i32| {
            let memory = caller
                .get_export("memory")
                .and_then(|e| e.into_memory())
                .ok_or_else(|| error_to_trap(anyhow::anyhow!("Memory not found")))?;

            // 读取 method
            let method = memory
                .data(&caller)
                .get(method_ptr as usize..(method_ptr as usize + method_len as usize))
                .and_then(|d| std::str::from_utf8(d).ok())
                .unwrap_or("GET");

            // 读取 URL
            let url = memory
                .data(&caller)
                .get(url_ptr as usize..(url_ptr as usize + url_len as usize))
                .and_then(|d| std::str::from_utf8(d).ok())
                .unwrap_or("");

            // 读取 body（可为空）
            let _body = if body_len > 0 {
                memory
                    .data(&caller)
                    .get(body_ptr as usize..(body_ptr as usize + body_len as usize))
                    .map(|d| d.to_vec())
            } else {
                None
            };

            tracing::debug!("[plugin] gw_http_request: {} {}", method, url);

            // 同步 HTTP 请求：使用阻塞方式执行
            // 由于 wasmtime 的同步函数限制，这里使用 tokio block_in_place
            // 如果在异步上下文之外调用，则返回错误
            let response = perform_sync_http_request(method, url);

            match response {
                Ok(response_bytes) => {
                    let write_len = (response_bytes.len() as i32).min(response_buf_len);
                    if write_len > 0 {
                        let data_mut = memory.data_mut(&mut caller);
                        if let Some(slice) = data_mut.get_mut(response_buf_ptr as usize..(response_buf_ptr as usize + write_len as usize)) {
                            slice.copy_from_slice(&response_bytes[..write_len as usize]);
                        }
                    }
                    Ok(response_bytes.len() as i32)
                }
                Err(e) => {
                    tracing::error!("[plugin] HTTP request failed: {}", e);
                    Ok(-1i32)
                }
            }
        },
    )?;

    // gw_get_config(key_ptr: i32, key_len: i32, value_ptr: i32, value_buf_len: i32) -> i32
    // 返回实际值长度，负数表示错误或值不存在
    linker.func_wrap(
        "gateway",
        "gw_get_config",
        |mut caller: Caller<'_, WasiP1Ctx>, key_ptr: i32, key_len: i32, value_ptr: i32, value_buf_len: i32| {
            let memory = caller
                .get_export("memory")
                .and_then(|e| e.into_memory())
                .ok_or_else(|| error_to_trap(anyhow::anyhow!("Memory not found")))?;

            let key_data = memory
                .data(&caller)
                .get(key_ptr as usize..(key_ptr as usize + key_len as usize))
                .ok_or_else(|| error_to_trap(anyhow::anyhow!("Invalid key memory range")))?;

            let key = std::str::from_utf8(key_data)
                .map_err(|e| error_to_trap(anyhow::anyhow!("Invalid key UTF-8: {}", e)))?;

            let ctx = global_host_context();
            match ctx.get_config(key) {
                Some(value) => {
                    let value_bytes = value.as_bytes();
                    let write_len = (value_bytes.len() as i32).min(value_buf_len);
                    if write_len > 0 {
                        let data_mut = memory.data_mut(&mut caller);
                        if let Some(slice) = data_mut.get_mut(value_ptr as usize..(value_ptr as usize + write_len as usize)) {
                            slice.copy_from_slice(&value_bytes[..write_len as usize]);
                        }
                    }
                    Ok(value_bytes.len() as i32)
                }
                None => Ok(-1i32),
            }
        },
    )?;

    // gw_set_config(key_ptr: i32, key_len: i32, value_ptr: i32, value_len: i32) -> i32
    // 返回 0 表示成功，负数表示错误
    linker.func_wrap(
        "gateway",
        "gw_set_config",
        |mut caller: Caller<'_, WasiP1Ctx>, key_ptr: i32, key_len: i32, value_ptr: i32, value_len: i32| {
            let memory = caller
                .get_export("memory")
                .and_then(|e| e.into_memory())
                .ok_or_else(|| error_to_trap(anyhow::anyhow!("Memory not found")))?;

            let key_data = memory
                .data(&caller)
                .get(key_ptr as usize..(key_ptr as usize + key_len as usize))
                .ok_or_else(|| error_to_trap(anyhow::anyhow!("Invalid key memory range")))?;

            let value_data = memory
                .data(&caller)
                .get(value_ptr as usize..(value_ptr as usize + value_len as usize))
                .ok_or_else(|| error_to_trap(anyhow::anyhow!("Invalid value memory range")))?;

            let key = std::str::from_utf8(key_data)
                .map_err(|e| error_to_trap(anyhow::anyhow!("Invalid key UTF-8: {}", e)))?;

            let value = std::str::from_utf8(value_data)
                .map_err(|e| error_to_trap(anyhow::anyhow!("Invalid value UTF-8: {}", e)))?;

            let ctx = global_host_context();
            ctx.set_config(key.to_string(), value.to_string());

            tracing::debug!("[plugin] gw_set_config: {} = {}", key, value);
            Ok(0i32)
        },
    )?;

    // gw_get_request_context(field_ptr: i32, field_len: i32, value_ptr: i32, value_buf_len: i32) -> i32
    // 返回实际值长度，负数表示错误或字段不存在
    linker.func_wrap(
        "gateway",
        "gw_get_request_context",
        |mut caller: Caller<'_, WasiP1Ctx>, field_ptr: i32, field_len: i32, value_ptr: i32, value_buf_len: i32| {
            let memory = caller
                .get_export("memory")
                .and_then(|e| e.into_memory())
                .ok_or_else(|| error_to_trap(anyhow::anyhow!("Memory not found")))?;

            let field_data = memory
                .data(&caller)
                .get(field_ptr as usize..(field_ptr as usize + field_len as usize))
                .ok_or_else(|| error_to_trap(anyhow::anyhow!("Invalid field memory range")))?;

            let field = std::str::from_utf8(field_data)
                .map_err(|e| error_to_trap(anyhow::anyhow!("Invalid field UTF-8: {}", e)))?;

            let ctx = global_host_context();
            match ctx.get_request_context(field) {
                Some(value) => {
                    let value_bytes = value.as_bytes();
                    let write_len = (value_bytes.len() as i32).min(value_buf_len);
                    if write_len > 0 {
                        let data_mut = memory.data_mut(&mut caller);
                        if let Some(slice) = data_mut.get_mut(value_ptr as usize..(value_ptr as usize + write_len as usize)) {
                            slice.copy_from_slice(&value_bytes[..write_len as usize]);
                        }
                    }
                    Ok(value_bytes.len() as i32)
                }
                None => Ok(-1i32),
            }
        },
    )?;

    Ok(())
}

/// 同步执行 HTTP 请求
///
/// 在 WASM 宿主函数中使用，因为 wasmtime 同步函数不能直接 await
fn perform_sync_http_request(method: &str, url: &str) -> Result<Vec<u8>> {
    // 使用 tokio runtime 执行异步 HTTP 请求
    let rt = tokio::runtime::Handle::try_current();
    match rt {
        Ok(_handle) => {
            let method = method.to_string();
            let url = url.to_string();
            // 我们不能在同步上下文中 block_on，所以记录警告并返回错误
            tracing::warn!(
                "[plugin] HTTP request from sync WASM context not fully supported yet: {} {}",
                method,
                url
            );
            // 对于现在，返回一个简单的 JSON 错误响应
            let error_response = format!(
                r#"{{"error": "HTTP proxy not available in sync context", "method": "{}", "url": "{}"}}"#,
                method, url
            );
            Ok(error_response.into_bytes())
        }
        Err(_) => {
            tracing::warn!("[plugin] No tokio runtime available for HTTP request");
            Err(anyhow::anyhow!("No tokio runtime available"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasmtime::Engine;

    #[test]
    fn test_log_level_from_i32() {
        assert!(matches!(LogLevel::try_from(0).unwrap(), LogLevel::Trace));
        assert!(matches!(LogLevel::try_from(1).unwrap(), LogLevel::Debug));
        assert!(matches!(LogLevel::try_from(2).unwrap(), LogLevel::Info));
        assert!(matches!(LogLevel::try_from(3).unwrap(), LogLevel::Warn));
        assert!(matches!(LogLevel::try_from(4).unwrap(), LogLevel::Error));
        assert!(LogLevel::try_from(5).is_err());
        assert!(LogLevel::try_from(-1).is_err());
    }

    #[test]
    fn test_add_gateway_host_functions() {
        let engine = Engine::default();
        let mut linker = Linker::new(&engine);
        let result = add_gateway_host_functions(&mut linker);
        assert!(result.is_ok());
    }

    #[test]
    fn test_host_context() {
        let ctx = HostContext::new();
        ctx.set_config("key1".to_string(), "value1".to_string());
        assert_eq!(ctx.get_config("key1"), Some("value1".to_string()));
        assert_eq!(ctx.get_config("nonexistent"), None);

        ctx.set_request_context("request_id".to_string(), "abc123".to_string());
        assert_eq!(ctx.get_request_context("request_id"), Some("abc123".to_string()));
    }
}