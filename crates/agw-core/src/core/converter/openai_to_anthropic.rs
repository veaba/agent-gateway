//! OpenAI 到 Anthropic 转换器

use anyhow::Result;
use serde_json::{json, Value};

/// OpenAI Chat Completions API 请求转换为 Anthropic Messages API 格式
pub fn openai_request_to_anthropic(request: &Value) -> Result<Value> {
    let model = request.get("model")
        .and_then(|v| v.as_str())
        .unwrap_or("claude-3-sonnet-20240229");

    let messages = request.get("messages")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let mut anthropic_messages: Vec<Value> = Vec::new();

    for msg in messages {
        let role = match msg.get("role").and_then(|v| v.as_str()) {
            Some("user") | Some("system") => "user",
            Some("assistant") => "assistant",
            _ => "user",
        };

        let content = msg.get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        anthropic_messages.push(json!({
            "role": role,
            "content": [{
                "type": "text",
                "text": content
            }]
        }));
    }

    let mut anthropic_request = json!({
        "model": model,
        "messages": anthropic_messages,
    });

    // 复制可选参数
    if let Some(max_tokens) = request.get("max_tokens") {
        anthropic_request["max_tokens"] = max_tokens.clone();
    } else {
        // Anthropic 要求 max_tokens
        anthropic_request["max_tokens"] = json!(4096);
    }
    if let Some(temperature) = request.get("temperature") {
        anthropic_request["temperature"] = temperature.clone();
    }
    if let Some(stream) = request.get("stream") {
        anthropic_request["stream"] = stream.clone();
    }
    if let Some(system) = request.get("system") {
        // 将 system 消息添加到消息数组前面
        let msgs = anthropic_request["messages"].as_array_mut().unwrap();
        msgs.insert(0, json!({
            "role": "user",
            "content": [{
                "type": "text",
                "text": system
            }]
        }));
    }

    Ok(anthropic_request)
}

/// Anthropic Messages API 响应转换为 OpenAI Chat Completions API 格式
pub fn anthropic_response_to_openai(response: &Value) -> Result<Value> {
    let id = response.get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("chatcmpl_001");

    let content = response.get("content")
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
        .and_then(|item| item.get("text"))
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let model = response.get("model")
        .and_then(|v| v.as_str())
        .unwrap_or("gpt-4");

    Ok(json!({
        "id": id,
        "object": "chat.completion",
        "created": chrono::Utc::now().timestamp(),
        "model": model,
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": content
            },
            "finish_reason": "stop"
        }],
        "usage": {
            "prompt_tokens": 0,
            "completion_tokens": 0,
            "total_tokens": 0
        }
    }))
}

/// OpenAI SSE 事件转换为 Anthropic SSE 格式
pub fn openai_sse_to_anthropic(sse_line: &str) -> Option<String> {
    if sse_line.starts_with("data: ") {
        let data = &sse_line[6..];

        // 跳过 [DONE] 事件
        if data == "[DONE]" {
            return Some("data: [DONE]\n\n".to_string());
        }

        // TODO: 解析 OpenAI 事件并转换为 Anthropic 格式
        // 这里需要解析 data 字段中的 JSON 并提取 delta content
        return Some(sse_line.to_string());
    }

    None
}