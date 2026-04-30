//! SSE converter integration tests

use agw_core::core::converter::{anthropic_sse_to_openai, openai_sse_to_anthropic};

#[test]
fn test_anthropic_sse_to_openai_content_delta() {
    let sse_line = "event: content_block_delta\ndata: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"Hello\"}}\n\n";
    let result = anthropic_sse_to_openai(sse_line);
    assert!(result.is_some());
    let converted = result.unwrap();
    assert!(converted.starts_with("data: "));
    assert!(converted.contains("chat.completion.chunk"));
    assert!(converted.contains("Hello"));
}

#[test]
fn test_anthropic_sse_to_openai_message_start() {
    let sse_line = "event: message_start\ndata: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_123\"}}\n\n";
    let result = anthropic_sse_to_openai(sse_line);
    assert!(result.is_some());
    let converted = result.unwrap();
    assert!(converted.starts_with("data: "));
    assert!(converted.contains("msg_123"));
}

#[test]
fn test_anthropic_sse_to_openai_message_delta() {
    let sse_line = "event: message_delta\ndata: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"stop\"}}\n\n";
    let result = anthropic_sse_to_openai(sse_line);
    assert!(result.is_some());
    let converted = result.unwrap();
    assert!(converted.contains("finish_reason\":\"stop\""));
}

#[test]
fn test_anthropic_sse_to_openai_tool_use_start() {
    let sse_line = "event: content_block_start\ndata: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"tool_use\",\"id\":\"toolu_123\",\"name\":\"get_weather\"}}\n\n";
    let result = anthropic_sse_to_openai(sse_line);
    assert!(result.is_some());
    let converted = result.unwrap();
    assert!(converted.contains("tool_calls"));
    assert!(converted.contains("get_weather"));
}

#[test]
fn test_anthropic_sse_to_openai_ping() {
    let sse_line = "event: ping\ndata: {}\n\n";
    let result = anthropic_sse_to_openai(sse_line);
    assert!(result.is_none());
}

#[test]
fn test_anthropic_sse_to_openai_done() {
    let sse_line = "data: [DONE]\n\n";
    let result = anthropic_sse_to_openai(sse_line);
    assert!(result.is_some());
    assert_eq!(result.unwrap(), "data: [DONE]\n\n");
}

#[test]
fn test_openai_sse_to_anthropic_content_delta() {
    let sse_line = "data: {\"id\":\"chatcmpl-123\",\"object\":\"chat.completion.chunk\",\"created\":1234567890,\"model\":\"gpt-4\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\"Hello\"},\"finish_reason\":null}]}\n\n";
    let result = openai_sse_to_anthropic(sse_line);
    assert!(result.is_some());
    let converted = result.unwrap();
    assert!(converted.contains("content_block_delta"));
    assert!(converted.contains("text_delta"));
    assert!(converted.contains("Hello"));
}

#[test]
fn test_openai_sse_to_anthropic_done() {
    let sse_line = "data: [DONE]\n\n";
    let result = openai_sse_to_anthropic(sse_line);
    assert!(result.is_some());
    assert_eq!(result.unwrap(), "data: [DONE]\n\n");
}

#[test]
fn test_openai_sse_to_anthropic_tool_calls() {
    let sse_line = "data: {\"id\":\"chatcmpl-123\",\"object\":\"chat.completion.chunk\",\"created\":1234567890,\"model\":\"gpt-4\",\"choices\":[{\"index\":0,\"delta\":{\"tool_calls\":[{\"index\":0,\"function\":{\"name\":\"get_weather\"}}]},\"finish_reason\":null}]}\n\n";
    let result = openai_sse_to_anthropic(sse_line);
    assert!(result.is_some());
    let converted = result.unwrap();
    assert!(converted.contains("content_block_start"));
    assert!(converted.contains("tool_use"));
    assert!(converted.contains("get_weather"));
}

#[test]
fn test_openai_sse_to_anthropic_finish_reason() {
    let sse_line = "data: {\"id\":\"chatcmpl-123\",\"object\":\"chat.completion.chunk\",\"created\":1234567890,\"model\":\"gpt-4\",\"choices\":[{\"index\":0,\"delta\":{},\"finish_reason\":\"stop\"}]}\n\n";
    let result = openai_sse_to_anthropic(sse_line);
    assert!(result.is_some());
    let converted = result.unwrap();
    assert!(converted.contains("message_delta"));
    assert!(converted.contains("stop_reason"));
    assert!(converted.contains("stop"));
}

#[test]
fn test_openai_sse_to_anthropic_tool_calls_finish_reason() {
    let sse_line = "data: {\"id\":\"chatcmpl-123\",\"object\":\"chat.completion.chunk\",\"created\":1234567890,\"model\":\"gpt-4\",\"choices\":[{\"index\":0,\"delta\":{},\"finish_reason\":\"tool_calls\"}]}\n\n";
    let result = openai_sse_to_anthropic(sse_line);
    assert!(result.is_some());
    let converted = result.unwrap();
    assert!(converted.contains("message_delta"));
    // tool_calls should be mapped to tool_use
    assert!(converted.contains("tool_use"));
}

#[test]
fn test_openai_sse_to_anthropic_arguments_delta() {
    let sse_line = "data: {\"id\":\"chatcmpl-123\",\"object\":\"chat.completion.chunk\",\"created\":1234567890,\"model\":\"gpt-4\",\"choices\":[{\"index\":0,\"delta\":{\"tool_calls\":[{\"index\":0,\"function\":{\"arguments\":\"{\\\"location\\\":\\\"NYC\\\"}\"}}]},\"finish_reason\":null}]}\n\n";
    let result = openai_sse_to_anthropic(sse_line);
    assert!(result.is_some());
    let converted = result.unwrap();
    assert!(converted.contains("content_block_delta"));
    assert!(converted.contains("input_json_delta"));
    assert!(converted.contains("partial_json"));
}

#[test]
fn test_invalid_sse_lines() {
    let invalid = "not an sse line";
    assert!(anthropic_sse_to_openai(invalid).is_none());
    assert!(openai_sse_to_anthropic(invalid).is_none());

    let empty = "";
    assert!(anthropic_sse_to_openai(empty).is_none());
    assert!(openai_sse_to_anthropic(empty).is_none());
}
