# 第 8 章：协议转换

## 概述

agent-gateway 支持 Anthropic Messages API 和 OpenAI Chat Completions 之间的协议转换。

## 转换类型

### Anthropic → OpenAI

将 Anthropic 格式请求转换为 OpenAI 格式：

```json
// Anthropic 请求
{
  "model": "claude-sonnet-4-20250514",
  "max_tokens": 1024,
  "messages": [
    {"role": "user", "content": "Hello"}
  ]
}
```

转换为：

```json
// OpenAI 请求
{
  "model": "gpt-4o",
  "max_tokens": 1024,
  "messages": [
    {"role": "user", "content": "Hello"}
  ]
}
```

### OpenAI → Anthropic

将 OpenAI 格式请求转换为 Anthropic 格式：

```json
// OpenAI 请求
{
  "model": "gpt-4o",
  "max_tokens": 1024,
  "messages": [
    {"role": "user", "content": "Hello"}
  ]
}
```

转换为：

```json
// Anthropic 请求
{
  "model": "claude-sonnet-4-20250514",
  "max_tokens": 1024,
  "messages": [
    {"role": "user", "content": [{"type": "text", "text": "Hello"}]}
  ]
}
```

## 字段映射

### 消息角色

| Anthropic | OpenAI |
|----------|--------|
| user | user |
| assistant | assistant |
| system | system |

### 消息内容

| Anthropic | OpenAI |
|----------|--------|
| `text` 字符串 | `content` 字符串 |
| `[{"type": "text", "text": "..."}]` | `content` 字符串 |

### 工具调用

| Anthropic | OpenAI |
|----------|--------|
| `tool_use` | `tool_calls` |
| `tool_use_id` | `tool_call_id` |

### 流式响应

| Anthropic | OpenAI |
|----------|--------|
| `content_block_delta` | `chunk` |
| `message_delta` | `message` |

## 使用转换

### CLI

使用 `--format` 指定输出格式：

```bash
agw serve --format openai
agw serve --format anthropic
```

### API

在请求中指定目标格式：

```bash
# 使用 Anthropic 端点
curl -X POST http://127.0.0.1:8081/v1/messages \
  -H "Content-Type: application/json" \
  -d '{"model": "claude-sonnet-4-20250514", "messages": [...]}'

# 使用 OpenAI 端点
curl -X POST http://127.0.0.1:8081/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{"model": "gpt-4o", "messages": [...]}'
```

### Node.js

```javascript
import { getGateway } from '@agent-gateway/node';

const gateway = getGateway();

// 使用默认格式
const result = await gateway.chat({
  messages: [{ role: 'user', content: 'Hello' }],
});
```

## 模型映射

在配置中指定模型映射：

```yaml
user_plans:
  - id: my-plan
    provider_id: anthropic
    model_mapping:
      gpt-4o: claude-sonnet-4-20250514
      gpt-4-turbo: claude-3-opus-20240229
```

## SSE 流式

两种格式都支持 Server-Sent Events：

### Anthropic 风格

```bash
curl -N http://127.0.0.1:8081/v1/messages \
  -d '{"model": "claude-sonnet-4-20250514", "messages": [...], "stream": true}'
```

### OpenAI 风格

```bash
curl -N http://127.0.0.1:8081/v1/chat/completions \
  -d '{"model": "gpt-4o", "messages": [...], "stream": true}'
```

## 限制

当前转换限制：
- 不支持 `thinking` 思考过程
- 不支持多模态输入（仅文本）
- 不支持自定义函数

## 下一步

- [第 9 章：GUI 使用](09-gui.md) - 桌面应用指南
- [第 10 章：插件系统](10-plugin.md) - 扩展功能