//! Response converter integration tests

use serde_json::json;
use agw_core::core::converter::{openai_response_to_anthropic, anthropic_response_to_openai};

#[test]
fn test_openai_response_to_anthropic_basic() {
    let openai_response = json!({
        "id": "chatcmpl-123",
        "object": "chat.completion",
        "created": 1677652288,
        "model": "gpt-4",
        "choices": [
            {
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "Hello there!"
                },
                "finish_reason": "stop"
            }
        ],
        "usage": {
            "prompt_tokens": 10,
            "completion_tokens": 5,
            "total_tokens": 15
        }
    });

    let anthropic_response = openai_response_to_anthropic(&openai_response).unwrap();

    assert_eq!(anthropic_response["type"], "message");
    assert_eq!(anthropic_response["id"], "chatcmpl-123");
    assert_eq!(anthropic_response["role"], "assistant");
    assert_eq!(anthropic_response["model"], "gpt-4");

    let content = anthropic_response["content"].as_array().unwrap();
    assert_eq!(content.len(), 1);
    assert_eq!(content[0]["type"], "text");
    assert_eq!(content[0]["text"], "Hello there!");

    assert_eq!(anthropic_response["stop_reason"], "stop");
    assert_eq!(anthropic_response["usage"]["input_tokens"], 10);
    assert_eq!(anthropic_response["usage"]["output_tokens"], 5);
}

#[test]
fn test_openai_response_to_anthropic_tool_calls() {
    let openai_response = json!({
        "id": "chatcmpl-123",
        "choices": [
            {
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": null,
                    "tool_calls": [
                        {
                            "id": "call_abc123",
                            "type": "function",
                            "function": {
                                "name": "get_weather",
                                "arguments": "{\"location\":\"NYC\"}"
                            }
                        }
                    ]
                },
                "finish_reason": "tool_calls"
            }
        ],
        "usage": {
            "prompt_tokens": 10,
            "completion_tokens": 20,
            "total_tokens": 30
        }
    });

    let anthropic_response = openai_response_to_anthropic(&openai_response).unwrap();

    let content = anthropic_response["content"].as_array().unwrap();
    assert_eq!(content.len(), 1);
    assert_eq!(content[0]["type"], "tool_use");
    assert_eq!(content[0]["id"], "call_abc123");
    assert_eq!(content[0]["name"], "get_weather");
    assert_eq!(content[0]["input"]["location"], "NYC");

    assert_eq!(anthropic_response["stop_reason"], "tool_use");
}

#[test]
fn test_anthropic_response_to_openai_basic() {
    let anthropic_response = json!({
        "id": "msg_01XgYh3fF3k8pK5q",
        "type": "message",
        "role": "assistant",
        "model": "claude-3-sonnet-20240229",
        "content": [
            {"type": "text", "text": "Hello there!"}
        ],
        "stop_reason": "stop",
        "usage": {
            "input_tokens": 10,
            "output_tokens": 5
        }
    });

    let openai_response = anthropic_response_to_openai(&anthropic_response).unwrap();

    assert_eq!(openai_response["id"], "msg_01XgYh3fF3k8pK5q");
    assert_eq!(openai_response["object"], "chat.completion");
    assert_eq!(openai_response["model"], "claude-3-sonnet-20240229");

    let choices = openai_response["choices"].as_array().unwrap();
    assert_eq!(choices.len(), 1);
    assert_eq!(choices[0]["index"], 0);
    assert_eq!(choices[0]["message"]["role"], "assistant");
    assert_eq!(choices[0]["message"]["content"], "Hello there!");
    assert_eq!(choices[0]["finish_reason"], "stop");

    // The source code converts Anthropic usage to OpenAI usage format
    assert_eq!(openai_response["usage"]["prompt_tokens"], 10);
    assert_eq!(openai_response["usage"]["completion_tokens"], 5);
    assert_eq!(openai_response["usage"]["total_tokens"], 15);
}

#[test]
fn test_anthropic_response_to_openai_tool_use() {
    let anthropic_response = json!({
        "id": "msg_01XgYh3fF3k8pK5q",
        "type": "message",
        "role": "assistant",
        "model": "claude-3-sonnet",
        "content": [
            {"type": "text", "text": ""},
            {
                "type": "tool_use",
                "id": "toolu_01Abc123",
                "name": "get_weather",
                "input": {"location": "NYC"}
            }
        ],
        "stop_reason": "tool_use",
        "usage": {
            "input_tokens": 10,
            "output_tokens": 20
        }
    });

    let openai_response = anthropic_response_to_openai(&anthropic_response).unwrap();

    let choices = openai_response["choices"].as_array().unwrap();
    let message = &choices[0]["message"];

    assert!(message["tool_calls"].is_array());
    let tool_calls = message["tool_calls"].as_array().unwrap();
    assert_eq!(tool_calls.len(), 1);
    assert_eq!(tool_calls[0]["id"], "toolu_01Abc123");
    assert_eq!(tool_calls[0]["type"], "function");
    assert_eq!(tool_calls[0]["function"]["name"], "get_weather");
    assert_eq!(tool_calls[0]["function"]["arguments"], "{\"location\":\"NYC\"}");

    assert_eq!(choices[0]["finish_reason"], "tool_calls");
}

#[test]
fn test_anthropic_response_to_openai_empty_content() {
    let anthropic_response = json!({
        "id": "msg_01XgYh3fF3k8pK5q",
        "type": "message",
        "role": "assistant",
        "model": "claude-3-sonnet",
        "content": [],
        "stop_reason": "stop"
    });

    let openai_response = anthropic_response_to_openai(&anthropic_response).unwrap();

    let choices = openai_response["choices"].as_array().unwrap();
    let message = &choices[0]["message"];
    // Empty content should result in null
    assert!(message["content"].is_null());
}

#[test]
fn test_anthropic_response_to_openai_multimodal() {
    let anthropic_response = json!({
        "id": "msg_01XgYh3fF3k8pK5q",
        "type": "message",
        "role": "assistant",
        "model": "claude-3-sonnet",
        "content": [
            {"type": "text", "text": "Here's an image:"},
            {
                "type": "image",
                "source": {
                    "type": "base64",
                    "media_type": "image/jpeg",
                    "data": "base64data"
                }
            }
        ],
        "stop_reason": "stop"
    });

    let openai_response = anthropic_response_to_openai(&anthropic_response).unwrap();

    let choices = openai_response["choices"].as_array().unwrap();
    let message = &choices[0]["message"];
    let content = message["content"].as_array().unwrap();
    assert_eq!(content.len(), 2);
    assert_eq!(content[0]["type"], "text");
    assert_eq!(content[1]["type"], "image_url");
    assert!(content[1]["image_url"]["url"].as_str().unwrap().starts_with("data:image/jpeg;base64,"));
}

#[test]
fn test_openai_response_to_anthropic_empty_message() {
    let openai_response = json!({
        "id": "chatcmpl-123",
        "choices": [
            {
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": ""
                },
                "finish_reason": "stop"
            }
        ]
    });

    let anthropic_response = openai_response_to_anthropic(&openai_response).unwrap();

    let content = anthropic_response["content"].as_array().unwrap();
    assert_eq!(content.len(), 1);
    assert_eq!(content[0]["type"], "text");
    assert_eq!(content[0]["text"], "");
}

#[test]
fn test_roundtrip_conversion() {
    let original_openai = json!({
        "id": "chatcmpl-123",
        "object": "chat.completion",
        "created": 1677652288,
        "model": "gpt-4",
        "choices": [
            {
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "Hello!"
                },
                "finish_reason": "stop"
            }
        ],
        "usage": {
            "prompt_tokens": 10,
            "completion_tokens": 5,
            "total_tokens": 15
        }
    });

    let anthropic = openai_response_to_anthropic(&original_openai).unwrap();
    let back_to_openai = anthropic_response_to_openai(&anthropic).unwrap();

    // Check key fields are preserved
    assert_eq!(back_to_openai["choices"][0]["message"]["content"], "Hello!");
    assert_eq!(back_to_openai["choices"][0]["finish_reason"], "stop");
}
