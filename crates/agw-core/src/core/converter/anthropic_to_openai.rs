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

        let content = match msg.get("content") {
            Some(Value::String(text)) => json!(text),
            Some(Value::Array(blocks)) => {
                let parts: Vec<Value> = blocks.iter()
                    .filter_map(convert_anthropic_content_block_to_openai)
                    .collect();
                if parts.is_empty() {
                    json!("")
                } else if parts.len() == 1 && parts[0].get("type") == Some(&json!("text")) {
                    // Single text part: keep string format for backward compatibility
                    parts[0].get("text").cloned().unwrap_or_else(|| json!(""))
                } else {
                    json!(parts)
                }
            }
            _ => json!(""),
        };

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

    // 转换 tools
    if let Some(tools) = request.get("tools") {
        if let Ok(converted) = convert_anthropic_tools_to_openai(tools) {
            openai_request["tools"] = converted;
        }
    }

    // 转换 tool_choice
    if let Some(tool_choice) = request.get("tool_choice") {
        if let Ok(converted) = convert_anthropic_tool_choice_to_openai(tool_choice) {
            openai_request["tool_choice"] = converted;
        }
    }

    Ok(openai_request)
}

/// 将单个 Anthropic content block 转换为 OpenAI content part
fn convert_anthropic_content_block_to_openai(block: &Value) -> Option<Value> {
    let block_type = block.get("type").and_then(|v| v.as_str())?;
    match block_type {
        "text" => {
            block.get("text").and_then(|v| v.as_str()).map(|text| {
                json!({"type": "text", "text": text})
            })
        }
        "image" => {
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
        "document" => {
            let source = block.get("source");
            let media_type = source.and_then(|s| s.get("media_type")).and_then(|v| v.as_str()).unwrap_or("application/pdf");
            let data = source.and_then(|s| s.get("data")).and_then(|v| v.as_str()).unwrap_or("");
            let placeholder = if data.is_empty() {
                format!("[Document: {}]", media_type)
            } else {
                format!("[Document: {}]\n{}", media_type, data)
            };
            Some(json!({
                "type": "text",
                "text": placeholder
            }))
        }
        _ => None,
    }
}

/// 将 Anthropic tools 格式转换为 OpenAI tools 格式
fn convert_anthropic_tools_to_openai(tools: &Value) -> Result<Value> {
    let tools_array = tools.as_array()
        .ok_or_else(|| anyhow::anyhow!("tools must be an array"))?;
    let mut result = Vec::new();

    for tool in tools_array {
        let name = tool.get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let description = tool.get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let input_schema = tool.get("input_schema")
            .cloned()
            .unwrap_or(json!({"type": "object"}));

        result.push(json!({
            "type": "function",
            "function": {
                "name": name,
                "description": description,
                "parameters": input_schema
            }
        }));
    }

    Ok(json!(result))
}

/// 将 Anthropic tool_choice 格式转换为 OpenAI tool_choice 格式
fn convert_anthropic_tool_choice_to_openai(tool_choice: &Value) -> Result<Value> {
    if let Some(s) = tool_choice.as_str() {
        // Anthropic "none" 字符串
        let converted = match s {
            "none" => json!("none"),
            _ => json!("auto"),
        };
        Ok(converted)
    } else if let Some(obj) = tool_choice.as_object() {
        let choice_type = obj.get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("auto");
        match choice_type {
            "auto" => Ok(json!("auto")),
            "any" => Ok(json!("required")),
            "none" => Ok(json!("none")),
            "tool" => {
                let name = obj.get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                Ok(json!({
                    "type": "function",
                    "function": {
                        "name": name
                    }
                }))
            }
            _ => Ok(json!("auto")),
        }
    } else {
Ok(json!("auto"))
    }
}

/// OpenAI Chat Completions 响应转换为 Anthropic Messages API 格式
pub fn openai_response_to_anthropic(response: &Value) -> Result<Value> {
    let id = response.get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("msg_001");

    let message = response.get("choices")
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
        .and_then(|choice| choice.get("message"));

    let content = convert_openai_message_to_anthropic_content(message);

    let mut anthropic_response = json!({
        "type": "message",
        "id": id,
        "role": "assistant",
        "content": content,
        "model": response.get("model").cloned().unwrap_or(json!("claude-3-sonnet"))
    });

    // 复制 stop_reason / finish_reason
    let finish_reason = response.get("choices")
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
        .and_then(|choice| choice.get("finish_reason"))
        .and_then(|v| v.as_str())
        .unwrap_or("stop");
    if finish_reason == "tool_calls" {
        anthropic_response["stop_reason"] = json!("tool_use");
    } else if finish_reason != "null" && !finish_reason.is_empty() {
        anthropic_response["stop_reason"] = json!(finish_reason);
    }

    // 复制 usage
    if let Some(usage) = response.get("usage") {
        anthropic_response["usage"] = json!({
            "input_tokens": usage.get("prompt_tokens").and_then(|v| v.as_u64()).unwrap_or(0),
            "output_tokens": usage.get("completion_tokens").and_then(|v| v.as_u64()).unwrap_or(0)
        });
    }

    Ok(anthropic_response)
}

/// 将 OpenAI message 转换为 Anthropic content 数组
fn convert_openai_message_to_anthropic_content(message: Option<&Value>) -> Vec<Value> {
    let mut content = Vec::new();

    if let Some(msg) = message {
        // 处理 text / multimodal content
        match msg.get("content") {
            Some(Value::String(text)) if !text.is_empty() => {
                content.push(json!({
                    "type": "text",
                    "text": text
                }));
            }
            Some(Value::Array(parts)) => {
                for part in parts {
                    if let Some(part_type) = part.get("type").and_then(|v| v.as_str()) {
                        match part_type {
                            "text" => {
                                if let Some(text) = part.get("text").and_then(|v| v.as_str()) {
                                    if !text.is_empty() {
                                        content.push(json!({
                                            "type": "text",
                                            "text": text
                                        }));
                                    }
                                }
                            }
                            "image_url" => {
                                if let Some(image_url_obj) = part.get("image_url") {
                                    let url = image_url_obj.get("url").and_then(|v| v.as_str()).unwrap_or("");
                                    if url.starts_with("data:") {
                                        // Parse data URL: data:{media_type};base64,{data}
                                        if let Some(rest) = url.strip_prefix("data:") {
                                            if let Some((media_type, data)) = rest.split_once(";base64,") {
                                                content.push(json!({
                                                    "type": "image",
                                                    "source": {
                                                        "type": "base64",
                                                        "media_type": media_type,
                                                        "data": data
                                                    }
                                                }));
                                            }
                                        }
                                    } else if url.starts_with("http://") || url.starts_with("https://") {
                                        content.push(json!({
                                            "type": "image",
                                            "source": {
                                                "type": "url",
                                                "url": url
                                            }
                                        }));
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }

        // 处理 tool_calls
        if let Some(tool_calls) = msg.get("tool_calls").and_then(|v| v.as_array()) {
            for tool_call in tool_calls {
                let tool_id = tool_call.get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let func = tool_call.get("function");
                let name = func.and_then(|f| f.get("name"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let arguments_str = func.and_then(|f| f.get("arguments"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("{}");

                // 尝试解析 arguments 为 JSON
                let input: Value = serde_json::from_str(arguments_str).unwrap_or_else(|_| {
                    if arguments_str.is_empty() || arguments_str == "{}" {
                        json!({})
                    } else {
                        json!(arguments_str)
                    }
                });

                content.push(json!({
                    "type": "tool_use",
                    "id": tool_id,
                    "name": name,
                    "input": input
                }));
            }
        }
    }

    if content.is_empty() {
        content.push(json!({
            "type": "text",
            "text": ""
        }));
    }

    content
}

/// Anthropic SSE 事件转换为 OpenAI SSE 格式
///
/// Anthropic SSE 事件示例:
/// event: content_block_delta
/// data: {"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"Hello"}}
///
/// 转换为 OpenAI SSE:
/// data: {"id":"chatcmpl-xxx","object":"chat.completion.chunk","created":xxx,"model":"xxx","choices":[{"index":0,"delta":{"content":"Hello"},"finish_reason":null}]}
pub fn anthropic_sse_to_openai(sse_line: &str) -> Option<String> {
    // 处理 [DONE] 事件
    if sse_line.starts_with("data: ") {
        let data = sse_line.strip_prefix("data: ")?.trim();
        if data == "[DONE]" {
            return Some("data: [DONE]\n\n".to_string());
        }
    }

    // 解析 event 和 data
    let (event_type, data_str) = if sse_line.starts_with("event: ") {
        let rest = sse_line.strip_prefix("event: ")?;
        if let Some((event, data)) = rest.split_once("\ndata: ") {
            (Some(event.trim()), data)
        } else if let Some((event, rest)) = rest.split_once('\n') {
            if rest.starts_with("data: ") {
                (Some(event.trim()), rest.strip_prefix("data: ").unwrap_or(""))
            } else {
                (None, "")
            }
        } else {
            (None, "")
        }
    } else if sse_line.starts_with("data: ") {
        (None, sse_line.strip_prefix("data: ").unwrap_or(""))
    } else {
        return None;
    };

    let data_str = data_str.trim();
    if data_str.is_empty() {
        return None;
    }

    // 解析 JSON
    let json: Value = match serde_json::from_str(data_str) {
        Ok(v) => v,
        Err(_) => return None,
    };

    let event_type = event_type.unwrap_or_else(||
        json.get("type").and_then(|v| v.as_str()).unwrap_or("")
    );

    match event_type {
        // Anthropic content_block_delta 转换为 OpenAI delta
        "content_block_delta" => {
            let delta = json.get("delta")?;
            let delta_type = delta.get("type")?.as_str()?;

            if delta_type == "text_delta" {
                let text = delta.get("text")?.as_str()?;
                let index = json.get("index")?.as_u64()? as usize;

                return Some(format!(
                    "data: {{\"id\":\"chatcmpl-stream\",\"object\":\"chat.completion.chunk\",\"created\":{},\"model\":\"claude\",\"choices\":[{{\"index\":{},\"delta\":{{\"content\":\"{}\"}},\"finish_reason\":null}}]}}\n\n",
                    chrono::Utc::now().timestamp(),
                    index,
                    text.replace('\"', "\\\"").replace('\n', "\\n")
                ));
            } else if delta_type == "input_json_delta" {
                // 处理 tool_use 的 arguments 片段
                let partial_json = delta.get("partial_json")?.as_str()?;
                let index = json.get("index")?.as_u64()? as usize;

                return Some(format!(
                    "data: {{\"id\":\"chatcmpl-stream\",\"object\":\"chat.completion.chunk\",\"created\":{},\"model\":\"claude\",\"choices\":[{{\"index\":0,\"delta\":{{\"tool_calls\":[{{\"index\":{},\"function\":{{\"arguments\":\"{}\"}}}}]}},\"finish_reason\":null}}]}}\n\n",
                    chrono::Utc::now().timestamp(),
                    index,
                    partial_json.replace('\\', "\\\\").replace('\"', "\\\"").replace('\n', "\\n")
                ));
            }
        }
        // Anthropic message_start
        "message_start" => {
            // 可以发送初始的 OpenAI chunk
            let msg_id = json.get("message")
                .and_then(|m| m.get("id"))
                .and_then(|v| v.as_str())
                .unwrap_or("msg_001");
            return Some(format!(
                "data: {{\"id\":\"{}\",\"object\":\"chat.completion.chunk\",\"created\":{},\"model\":\"claude\",\"choices\":[{{\"index\":0,\"delta\":{{}},\"finish_reason\":null}}]}}\n\n",
                msg_id,
                chrono::Utc::now().timestamp()
            ));
        }
        // Anthropic content_block_start
        "content_block_start" => {
            let index = json.get("index")?.as_u64()? as usize;
            let content_block = json.get("content_block")?;
            let block_type = content_block.get("type")?.as_str()?;

            if block_type == "tool_use" {
                // tool_use 块转换为 OpenAI tool_calls
                let id = content_block.get("id")?.as_str()?;
                let name = content_block.get("name")?.as_str()?;

                return Some(format!(
                    "data: {{\"id\":\"chatcmpl-stream\",\"object\":\"chat.completion.chunk\",\"created\":{},\"model\":\"claude\",\"choices\":[{{\"index\":0,\"delta\":{{\"tool_calls\":[{{\"index\":{},\"id\":\"{}\",\"type\":\"function\",\"function\":{{\"name\":\"{}\",\"arguments\":\"\"}}}}]}},\"finish_reason\":null}}]}}\n\n",
                    chrono::Utc::now().timestamp(),
                    index,
                    id,
                    name
                ));
            }

            // 普通文本块开始
            return Some(format!(
                "data: {{\"id\":\"chatcmpl-stream\",\"object\":\"chat.completion.chunk\",\"created\":{},\"model\":\"claude\",\"choices\":[{{\"index\":{},\"delta\":{{}},\"finish_reason\":null}}]}}\n\n",
                chrono::Utc::now().timestamp(),
                index
            ));
        }
        // Anthropic message_delta (完成)
        "message_delta" => {
            let usage = json.get("usage");
            let stop_reason = json.get("delta")
                .and_then(|d| d.get("stop_reason"))
                .and_then(|v| v.as_str())
                .unwrap_or("stop");

            // Anthropic "tool_use" stop_reason 映射为 OpenAI "tool_calls"
            let openai_finish_reason = if stop_reason == "tool_use" { "tool_calls" } else { stop_reason };

            let usage_str = if let Some(usage) = usage {
                format!(
                    ",\"usage\":{{\"prompt_tokens\":0,\"completion_tokens\":{},\"total_tokens\":{}}}",
                    usage.get("output_tokens").and_then(|v| v.as_u64()).unwrap_or(0),
                    usage.get("output_tokens").and_then(|v| v.as_u64()).unwrap_or(0)
                )
            } else {
                String::new()
            };

            return Some(format!(
                "data: {{\"id\":\"chatcmpl-stream\",\"object\":\"chat.completion.chunk\",\"created\":{},\"model\":\"claude\",\"choices\":[{{\"index\":0,\"delta\":{{}},\"finish_reason\":\"{}\"}}]{}}}\n\n",
                chrono::Utc::now().timestamp(),
                openai_finish_reason,
                usage_str
            ));
        }
        // 忽略 ping 事件
        "ping" => return None,
        _ => {}
    }

    None
}