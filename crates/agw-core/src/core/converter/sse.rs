//! SSE 流式响应处理
//!
//! 提供 SSE (Server-Sent Events) 格式的转换功能
//! 支持 OpenAI 和 Anthropic 两种 SSE 格式的相互转换

use std::borrow::Cow;

/// SSE 行前缀
const SSE_DATA_PREFIX: &str = "data: ";
/// SSE 结束标记
const SSE_DONE_MARKER: &str = "[DONE]";
/// SSE 空行分隔符
const SSE_TERMINATOR: &str = "\n\n";

/// SSE 事件类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SseEvent {
    /// 数据事件
    Data(String),
    /// 完成事件
    Done,
    /// 错误事件
    Error(String),
}

impl std::fmt::Display for SseEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SseEvent::Data(data) => write!(f, "data: {}\n\n", data),
            SseEvent::Done => write!(f, "data: [DONE]\n\n"),
            SseEvent::Error(msg) => write!(f, "event: error\ndata: {}\n\n", msg),
        }
    }
}

/// SSE 流式响应转换器
///
/// 使用 `FxHashMap` 替代 `std::collections::HashMap` 以提升性能
#[allow(dead_code)]
pub struct SseConverter {
    /// 事件类型映射 (OpenAI -> Anthropic)
    event_type_map: rustc_hash::FxHashMap<Cow<'static, str>, Cow<'static, str>>,
}

impl SseConverter {
    /// 创建新的 SSE 转换器
    #[must_use]
    pub fn new() -> Self {
        Self {
            event_type_map: rustc_hash::FxHashMap::from_iter([
                (Cow::Borrowed("chat.completion.chunk"), Cow::Borrowed("message_start")),
                (Cow::Borrowed("chat.completion"), Cow::Borrowed("message_delta")),
            ]),
        }
    }

    /// 转换 SSE 行
    ///
    /// 如果行不是 SSE 格式或需要特殊处理，返回 `None`
    /// 否则返回转换后的行（可能只是原样返回）
    pub fn convert_line<'a>(&self, line: &'a str, _from_format: &str, _to_format: &str) -> Option<Cow<'a, str>> {
        // 快速路径：检查 SSE 数据行前缀
        let data = line.strip_prefix(SSE_DATA_PREFIX)?.trim();

        // 检测结束标记
        if data == SSE_DONE_MARKER {
            return Some(Cow::Owned(format!("{}{}{}", SSE_DATA_PREFIX, SSE_DONE_MARKER, SSE_TERMINATOR)));
        }

        // TODO: 根据 format 进行实际转换
        // 目前直接返回原行
        Some(Cow::Borrowed(line))
    }

    /// 检查是否为 SSE 结束标记
    #[inline]
    #[must_use]
    pub fn is_done_marker(line: &str) -> bool {
        line.trim() == SSE_DONE_MARKER
    }

    /// 检查行是否以 SSE 数据前缀开头
    #[inline]
    #[must_use]
    pub fn is_data_line(line: &str) -> bool {
        line.starts_with(SSE_DATA_PREFIX)
    }

    /// 提取 SSE 数据内容（不含前缀）
    #[inline]
    pub fn extract_data(line: &str) -> Option<&str> {
        line.strip_prefix(SSE_DATA_PREFIX).map(str::trim)
    }

    /// 格式化 SSE 事件为字节串
    #[inline]
    #[must_use]
    pub fn format_sse_event(event_type: &str, data: &str) -> Vec<u8> {
        format!("event: {}\ndata: {}\n\n", event_type, data).into_bytes()
    }
}

impl Default for SseConverter {
    fn default() -> Self {
        Self::new()
    }
}