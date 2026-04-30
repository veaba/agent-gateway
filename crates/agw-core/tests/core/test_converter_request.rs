//! Request converter integration tests

use serde_json::json;
use agw_core::core::converter::{anthropic_request_to_openai, openai_request_to_anthropic};

#[test]
fn test_anthropic_request_to_openai_basic() {
    let anthropic_request = json!({
        "model": "claude-3-sonnet-20240229",
        "messages": [
            {
                "role": "user",
                "content": [
                    {"type": "text", "text": "Hello, world!"}
                ]
            }
        ],
        "max_tokens": 1024,
        "temperature": 0.7
    });

    let openai_request = anthropic_request_to_openai(&anthropic_request).unwrap();

    assert_eq!(openai_request["model"], "claude-3-sonnet-20240229");
    assert!(openai_request["messages"].is_array());
    let messages = openai_request["messages"].as_array().unwrap();
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0]["role"], "user");
    assert_eq!(messages[0]["content"], "Hello, world!");
    assert_eq!(openai_request["max_tokens"], 1024);
    assert_eq!(openai_request["temperature"], 0.7);
}

#[test]
fn test_anthropic_request_to_openai_assistant_message() {
    let anthropic_request = json!({
        "model": "claude-3-sonnet",
        "messages": [
            {"role": "user", "content": [{"type": "text", "text": "Hi"}]},
            {"role": "assistant", "content": [{"type": "text", "text": "Hello!"}]}
        ],
        "max_tokens": 1024
    });

    let openai_request = anthropic_request_to_openai(&anthropic_request).unwrap();
    let messages = openai_request["messages"].as_array().unwrap();
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[1]["role"], "assistant");
    assert_eq!(messages[1]["content"], "Hello!");
}

#[test]
fn test_anthropic_request_to_openai_with_tools() {
    let anthropic_request = json!({
        "model": "claude-3-sonnet",
        "messages": [
            {"role": "user", "content": [{"type": "text", "text": "What's the weather?"}]}
        ],
        "max_tokens": 1024,
        "tools": [
            {
                "name": "get_weather",
                "description": "Get weather for a location",
                "input_schema": {
                    "type": "object",
                    "properties": {
                        "location": {"type": "string"}
                    }
                }
            }
        ]
    });

    let openai_request = anthropic_request_to_openai(&anthropic_request).unwrap();
    assert!(openai_request["tools"].is_array());
    let tools = openai_request["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0]["type"], "function");
    assert_eq!(tools[0]["function"]["name"], "get_weather");
}

#[test]
fn test_anthropic_request_to_openai_with_image() {
    let anthropic_request = json!({
        "model": "claude-3-sonnet",
        "messages": [
            {
                "role": "user",
                "content": [
                    {"type": "text", "text": "Describe this image"},
                    {
                        "type": "image",
                        "source": {
                            "type": "base64",
                            "media_type": "image/jpeg",
                            "data": "base64encodeddata"
                        }
                    }
                ]
            }
        ],
        "max_tokens": 1024
    });

    let openai_request = anthropic_request_to_openai(&anthropic_request).unwrap();
    let messages = openai_request["messages"].as_array().unwrap();
    let content = messages[0]["content"].as_array().unwrap();
    assert_eq!(content.len(), 2);
    assert_eq!(content[0]["type"], "text");
    assert_eq!(content[1]["type"], "image_url");
}

#[test]
fn test_openai_request_to_anthropic_basic() {
    let openai_request = json!({
        "model": "gpt-4",
        "messages": [
            {"role": "user", "content": "Hello, world!"}
        ],
        "max_tokens": 1024,
        "temperature": 0.7
    });

    let anthropic_request = openai_request_to_anthropic(&openai_request).unwrap();

    assert_eq!(anthropic_request["model"], "gpt-4");
    assert!(anthropic_request["messages"].is_array());
    let messages = anthropic_request["messages"].as_array().unwrap();
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0]["role"], "user");
    let content = messages[0]["content"].as_array().unwrap();
    assert_eq!(content[0]["text"], "Hello, world!");
    assert_eq!(anthropic_request["max_tokens"], 1024);
    assert_eq!(anthropic_request["temperature"], 0.7);
}

#[test]
fn test_openai_request_to_anthropic_system_message() {
    let openai_request = json!({
        "model": "gpt-4",
        "messages": [
            {"role": "system", "content": "You are a helpful assistant."},
            {"role": "user", "content": "Hello!"}
        ],
        "max_tokens": 1024
    });

    let anthropic_request = openai_request_to_anthropic(&openai_request).unwrap();
    let messages = anthropic_request["messages"].as_array().unwrap();
    // System message should be converted to user role in Anthropic format
    assert_eq!(messages[0]["role"], "user");
}

#[test]
fn test_openai_request_to_anthropic_with_tools() {
    let openai_request = json!({
        "model": "gpt-4",
        "messages": [
            {"role": "user", "content": "What's the weather?"}
        ],
        "max_tokens": 1024,
        "tools": [
            {
                "type": "function",
                "function": {
                    "name": "get_weather",
                    "description": "Get weather for a location",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "location": {"type": "string"}
                        }
                    }
                }
            }
        ]
    });

    let anthropic_request = openai_request_to_anthropic(&openai_request).unwrap();
    assert!(anthropic_request["tools"].is_array());
    let tools = anthropic_request["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0]["name"], "get_weather");
    assert_eq!(tools[0]["input_schema"]["type"], "object");
}

#[test]
fn test_openai_request_to_anthropic_tool_choice() {
    let openai_request = json!({
        "model": "gpt-4",
        "messages": [{"role": "user", "content": "Hi"}],
        "max_tokens": 1024,
        "tool_choice": "auto"
    });

    let anthropic_request = openai_request_to_anthropic(&openai_request).unwrap();
    assert_eq!(anthropic_request["tool_choice"], json!({"type": "auto"}));

    let openai_request_required = json!({
        "model": "gpt-4",
        "messages": [{"role": "user", "content": "Hi"}],
        "max_tokens": 1024,
        "tool_choice": "required"
    });

    let anthropic_request = openai_request_to_anthropic(&openai_request_required).unwrap();
    assert_eq!(anthropic_request["tool_choice"], json!({"type": "any"}));
}

#[test]
fn test_openai_request_to_anthropic_default_max_tokens() {
    let openai_request = json!({
        "model": "gpt-4",
        "messages": [{"role": "user", "content": "Hi"}]
    });

    let anthropic_request = openai_request_to_anthropic(&openai_request).unwrap();
    // Anthropic requires max_tokens, should default to 4096
    assert_eq!(anthropic_request["max_tokens"], 4096);
}

#[test]
fn test_openai_request_to_anthropic_multimodal() {
    let openai_request = json!({
        "model": "gpt-4-vision",
        "messages": [
            {
                "role": "user",
                "content": [
                    {"type": "text", "text": "Describe this"},
                    {
                        "type": "image_url",
                        "image_url": {
                            "url": "data:image/jpeg;base64,base64encodeddata"
                        }
                    }
                ]
            }
        ],
        "max_tokens": 1024
    });

    let anthropic_request = openai_request_to_anthropic(&openai_request).unwrap();
    let messages = anthropic_request["messages"].as_array().unwrap();
    let content = messages[0]["content"].as_array().unwrap();
    assert_eq!(content.len(), 2);
    assert_eq!(content[0]["type"], "text");
    assert_eq!(content[1]["type"], "image");
    assert_eq!(content[1]["source"]["type"], "base64");
}
