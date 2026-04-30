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

        let content: Vec<Value> = match msg.get("content") {
            Some(Value::String(text)) => {
                vec![json!({
                    "type": "text",
                    "text": text
                })]
            }
            Some(Value::Array(parts)) => {
                parts.iter()
                    .filter_map(convert_openai_content_part_to_anthropic)
                    .collect()
            }
            _ => {
                vec![json!({
                    "type": "text",
                    "text": ""
                })]
            }
        };

        anthropic_messages.push(json!({
            "role": role,
            "content": content
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

    // 转换 tools
    if let Some(tools) = request.get("tools") {
        if let Ok(converted) = convert_openai_tools_to_anthropic(tools) {
            anthropic_request["tools"] = converted;
        }
    }

    // 转换 tool_choice
    if let Some(tool_choice) = request.get("tool_choice") {
        if let Ok(converted) = convert_openai_tool_choice_to_anthropic(tool_choice) {
            anthropic_request["tool_choice"] = converted;
        }
    }

    Ok(anthropic_request)
}

/// 将单个 OpenAI content part 转换为 Anthropic content block
fn convert_openai_content_part_to_anthropic(part: &Value) -> Option<Value> {
    let part_type = part.get("type").and_then(|v| v.as_str())?;
    match part_type {
        "text" => {
            part.get("text").and_then(|v| v.as_str()).map(|text| {
                json!({"type": "text", "text": text})
            })
        }
        "image_url" => {
            let image_url_obj = part.get("image_url")?;
            let url = image_url_obj.get("url").and_then(|v| v.as_str())?;
            if url.starts_with("data:") {
                // Parse data URL: data:{media_type};base64,{data}
                let rest = url.strip_prefix("data:")?;
                let (media_type, data) = rest.split_once(";base64,")?;
                Some(json!({
                    "type": "image",
                    "source": {
                        "type": "base64",
                        "media_type": media_type,
                        "data": data
                    }
                }))
            } else if url.starts_with("http://") || url.starts_with("https://") {
                Some(json!({
                    "type": "image",
                    "source": {
                        "type": "url",
                        "url": url
                    }
                }))
            } else {
                None
            }
        }
        _ => None,
    }
}

/// 将 OpenAI tools 格式转换为 Anthropic tools 格式
fn convert_openai_tools_to_anthropic(tools: &Value) -> Result<Value> {
    let tools_array = tools.as_array()
        .ok_or_else(|| anyhow::anyhow!("tools must be an array"))?;
    let mut result = Vec::new();

    for tool in tools_array {
        if let Some(func) = tool.get("function") {
            let name = func.get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let description = func.get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let parameters = func.get("parameters")
                .cloned()
                .unwrap_or(json!({"type": "object"}));

            result.push(json!({
                "name": name,
                "description": description,
                "input_schema": parameters
            }));
        }
    }

    Ok(json!(result))
}

/// 将 OpenAI tool_choice 格式转换为 Anthropic tool_choice 格式
fn convert_openai_tool_choice_to_anthropic(tool_choice: &Value) -> Result<Value> {
    if let Some(s) = tool_choice.as_str() {
        let converted = match s {
            "auto" => json!({"type": "auto"}),
            "required" => json!({"type": "any"}),
            "none" => json!("none"),
            _ => json!({"type": "auto"}),
        };
        Ok(converted)
    } else if let Some(obj) = tool_choice.as_object() {
        let tool_type = obj.get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("auto");
        if tool_type == "function" {
            if let Some(func) = obj.get("function") {
                let name = func.get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                return Ok(json!({
                    "type": "tool",
                    "name": name
                }));
            }
        }
        Ok(json!({"type": tool_type}))
    } else {
        Ok(json!({"type": "auto"}))
    }
}

/// Anthropic Messages API 响应转换为 OpenAI Chat Completions API 格式
pub fn anthropic_response_to_openai(response: &Value) -> Result<Value> {
    let id = response.get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("chatcmpl_001");

    let model = response.get("model")
        .and_then(|v| v.as_str())
        .unwrap_or("gpt-4");

    // 提取 content 中的 text 和 tool_use
    let content_array = response.get("content").and_then(|v| v.as_array());
    let (text_content, tool_calls) = extract_anthropic_content_to_openai(content_array);

    let mut message = json!({
        "role": "assistant",
        "content": text_content
    });

    if !tool_calls.is_empty() {
        message["tool_calls"] = json!(tool_calls);
    }

    let stop_reason = response.get("stop_reason")
        .and_then(|v| v.as_str())
        .unwrap_or("stop");
    let finish_reason = if stop_reason == "tool_use" { "tool_calls" } else { stop_reason };

    // Convert Anthropic usage format to OpenAI usage format
    let usage = response.get("usage").map(|u| {
        json!({
            "prompt_tokens": u.get("input_tokens").and_then(|v| v.as_u64()).unwrap_or(0),
            "completion_tokens": u.get("output_tokens").and_then(|v| v.as_u64()).unwrap_or(0),
            "total_tokens": u.get("input_tokens").and_then(|v| v.as_u64()).unwrap_or(0)
                + u.get("output_tokens").and_then(|v| v.as_u64()).unwrap_or(0)
        })
    }).unwrap_or_else(|| {
        json!({
            "prompt_tokens": 0,
            "completion_tokens": 0,
            "total_tokens": 0
        })
    });

    Ok(json!({
        "id": id,
        "object": "chat.completion",
        "created": chrono::Utc::now().timestamp(),
        "model": model,
        "choices": [{
            "index": 0,
            "message": message,
            "finish_reason": finish_reason
        }],
        "usage": usage
    }))
}

/// 将 Anthropic image block 转换为 OpenAI image_url part
fn convert_anthropic_image_block_to_openai_part(block: &Value) -> Option<Value> {
    let source = block.get("source")?;
    let source_type = source.get("type").and_then(|v| v.as_str())?;
    match source_type {
        "base64" => {
            let media_type = source.get("media_type").and_then(|v| v.as_str()).unwrap_or("image/jpeg");
            let data = source.get("data").and_then(|v| v.as_str())?;
            let url = format!("data:{};base64,{}", media_type, data);
            Some(json!({
                "type": "image_url",
                "image_url": {
                    "url": url,
                    "detail": "auto"
                }
            }))
        }
        "url" => {
            let url = source.get("url").and_then(|v| v.as_str())?;
            Some(json!({
                "type": "image_url",
                "image_url": {
                    "url": url,
                    "detail": "auto"
                }
            }))
        }
        _ => None,
    }
}

/// 从 Anthropic content 数组中提取 OpenAI 格式的 content 和 tool_calls
fn extract_anthropic_content_to_openai(content: Option<&Vec<Value>>) -> (Value, Vec<Value>) {
    let mut text_parts: Vec<String> = Vec::new();
    let mut multimodal_parts: Vec<Value> = Vec::new();
    let mut tool_calls = Vec::new();

    if let Some(arr) = content {
        for block in arr {
            match block.get("type").and_then(|v| v.as_str()) {
                Some("text") => {
                    if let Some(text) = block.get("text").and_then(|v| v.as_str()) {
                        if !text.is_empty() {
                            text_parts.push(text.to_string());
                        }
                    }
                }
                Some("image") => {
                    if let Some(part) = convert_anthropic_image_block_to_openai_part(block) {
                        multimodal_parts.push(part);
                    }
                }
                Some("document") => {
                    let source = block.get("source");
                    let media_type = source.and_then(|s| s.get("media_type")).and_then(|v| v.as_str()).unwrap_or("application/pdf");
                    let data = source.and_then(|s| s.get("data")).and_then(|v| v.as_str()).unwrap_or("");
                    let placeholder = if data.is_empty() {
                        format!("[Document: {}]", media_type)
                    } else {
                        format!("[Document: {}]\n{}", media_type, data)
                    };
                    text_parts.push(placeholder);
                }
                Some("tool_use") => {
                    let id = block.get("id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let name = block.get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let input = block.get("input").cloned().unwrap_or(json!({}));

                    tool_calls.push(json!({
                        "id": id,
                        "type": "function",
                        "function": {
                            "name": name,
                            "arguments": input.to_string()
                        }
                    }));
                }
                _ => {}
            }
        }
    }

    let content_value = if multimodal_parts.is_empty() {
        let text = text_parts.join("");
        if text.is_empty() {
            Value::Null
        } else {
            json!(text)
        }
    } else {
        let mut parts: Vec<Value> = Vec::new();
        let text = text_parts.join("");
        if !text.is_empty() {
            parts.push(json!({"type": "text", "text": text}));
        }
        parts.extend(multimodal_parts);
        json!(parts)
    };

    (content_value, tool_calls)
}

/// OpenAI SSE 事件转换为 Anthropic SSE 格式
///
/// OpenAI SSE 事件示例:
/// data: {"id":"chatcmpl-xxx","object":"chat.completion.chunk","choices":[{"index":0,"delta":{"content":"Hello"},"finish_reason":null}]}
///
/// 转换为 Anthropic SSE:
/// event: content_block_delta
/// data: {"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"Hello"}}
pub fn openai_sse_to_anthropic(sse_line: &str) -> Option<String> {
    // 处理 [DONE] 事件
    if sse_line.starts_with("data: ") {
        let data = sse_line.strip_prefix("data: ")?;
        if data == "[DONE]" {
            return Some("data: [DONE]\n\n".to_string());
        }
    }

    let data_str = sse_line.strip_prefix("data: ")?.trim();
    if data_str.is_empty() || data_str == "[DONE]" {
        return Some("data: [DONE]\n\n".to_string());
    }

    // 解析 OpenAI SSE JSON
    let json: Value = match serde_json::from_str(data_str) {
        Ok(v) => v,
        Err(_) => return None,
    };

    // 检查是否是有效的 chat completion chunk
    let obj_type = json.get("object").and_then(|v| v.as_str());
    if obj_type != Some("chat.completion.chunk") {
        return None;
    }

    let index = json.get("choices")
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
        .and_then(|choice| choice.get("index"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as usize;

    let delta = json.get("choices")
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
        .and_then(|choice| choice.get("delta"))?;

    // 处理 content delta
    if let Some(content) = delta.get("content").and_then(|v| v.as_str()) {
        let escaped_content = content.replace('\\', "\\\\").replace('"', "\\\"");
        return Some(format!(
            "event: content_block_delta\ndata: {{\"type\":\"content_block_delta\",\"index\":{},\"delta\":{{\"type\":\"text_delta\",\"text\":\"{}\"}}}}\n\n",
            index,
            escaped_content
        ));
    }

    // 处理 tool_calls delta
    if let Some(tool_calls) = delta.get("tool_calls").and_then(|v| v.as_array()) {
        for tool_call in tool_calls {
            let tool_index = tool_call.get("index").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
            let func = tool_call.get("function");

            // 如果有 name，表示新的 tool call 开始
            if let Some(name) = tool_call.get("name")
                .and_then(|v| v.as_str())
                .or_else(|| func.and_then(|f| f.get("name")).and_then(|v| v.as_str()))
            {
                let id = tool_call.get("id").and_then(|v| v.as_str()).unwrap_or("");
                return Some(format!(
                    "event: content_block_start\ndata: {{\"type\":\"content_block_start\",\"index\":{},\"content_block\":{{\"type\":\"tool_use\",\"id\":\"{}\",\"name\":\"{}\"}}}}\n\n",
                    tool_index, id, name
                ));
            }

            // 如果有 arguments 片段
            if let Some(arguments) = func
                .and_then(|f| f.get("arguments"))
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
            {
                let escaped = arguments.replace('\\', "\\\\").replace('"', "\\\"");
                return Some(format!(
                    "event: content_block_delta\ndata: {{\"type\":\"content_block_delta\",\"index\":{},\"delta\":{{\"type\":\"input_json_delta\",\"partial_json\":\"{}\"}}}}\n\n",
                    tool_index,
                    escaped
                ));
            }
        }
    }

    // 处理 finish_reason (完成信号)
    if let Some(reason) = json.get("choices")
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
        .and_then(|choice| choice.get("finish_reason"))
        .and_then(|v| v.as_str())
    {
        if reason != "null" && !reason.is_empty() {
            // Anthropic 使用 message_delta 事件表示完成
            // OpenAI "tool_calls" finish_reason 映射为 Anthropic "tool_use"
            let anthropic_stop_reason = if reason == "tool_calls" { "tool_use" } else { reason };
            let usage = json.get("usage");
            let usage_str = if let Some(usage) = usage {
                format!(
                    ",\"usage\":{{\"output_tokens\":{}}}",
                    usage.get("completion_tokens").and_then(|v| v.as_u64()).unwrap_or(0)
                )
            } else {
                String::new()
            };

            return Some(format!(
                "event: message_delta\ndata: {{\"type\":\"message_delta\",\"delta\":{{\"stop_reason\":\"{}\"}}{}\n\n",
                anthropic_stop_reason,
                usage_str
            ));
        }
    }

    None
}