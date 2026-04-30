//! Anthropic 到 OpenAI 转换器

use anyhow::Result;
use serde_json::{json, Value};

/// Anthropic Messages API 请求转换为 OpenAI Chat Completions API 格式
pub fn anthropic_request_to_openai(request: &Value) -> Result<Value> {
    let model = request.get("model")
        .and_then(|v| v.as_str())
        .unwrap_or("gpt-4");

    let messages = request.get("messages")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let mut openai_messages: Vec<Value> = Vec::new();

    for msg in messages {
        let role = match msg.get("role").and_then(|v| v.as_str()) {
            Some("user") => "user",
            Some("assistant") => "assistant",
            _ => "user",
        };

        let content = msg.get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        openai_messages.push(json!({
            "role": role,
            "content": content
        }));
    }

    let mut openai_request = json!({
        "model": model,
        "messages": openai_messages,
    });

    // 复制可选参数
    if let Some(max_tokens) = request.get("max_tokens") {
        openai_request["max_tokens"] = max_tokens.clone();
    }
    if let Some(temperature) = request.get("temperature") {
        openai_request["temperature"] = temperature.clone();
    }
    if let Some(stream) = request.get("stream") {
        openai_request["stream"] = stream.clone();
    }

    Ok(openai_request)
}

/// OpenAI Chat Completions 响应转换为 Anthropic Messages API 格式
pub fn openai_response_to_anthropic(response: &Value) -> Result<Value> {
    let id = response.get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("msg_001");

    let content = response.get("choices")
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
        .and_then(|choice| choice.get("message"))
        .and_then(|msg| msg.get("content"))
        .and_then(|v| v.as_str())
        .unwrap_or("");

    Ok(json!({
        "type": "message",
        "id": id,
        "role": "assistant",
        "content": [{
            "type": "text",
            "text": content
        }],
        "model": response.get("model")
    }))
}

/// Anthropic SSE 事件转换为 OpenAI SSE 格式
pub fn anthropic_sse_to_openai(sse_line: &str) -> Option<String> {
    if sse_line.starts_with("data: ") {
        let data = &sse_line[6..];

        // 跳过 [DONE] 事件
        if data == "[DONE]" {
            return Some("data: [DONE]\n\n".to_string());
        }

        // TODO: 解析 Anthropic 事件并转换为 OpenAI 格式
        // 这里需要解析 data 字段中的 JSON 并提取 delta content
        return Some(sse_line.to_string());
    }

    None
}