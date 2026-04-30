# agent-gateway API Server 使用文档

## 概述

agw-api 是 agent-gateway 的 REST API 服务器，基于 Axum 框架构建。

## 启动

```bash
# 开发模式
cargo run -p agw-api

# 生产模式
cargo build -p agw-api --release
./target/release/agw-api.exe &
```

服务默认监听 `http://127.0.0.1:8081`

## 健康检查

```bash
curl http://127.0.0.1:8081/health
```

响应:
```json
{"status":"ok","version":"0.1.0"}
```

## API 端点

### Plan 管理

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | /api/v1/plans | 列出所有套餐 |
| POST | /api/v1/plans | 创建套餐 |
| GET | /api/v1/plans/:id | 获取套餐详情 |
| PUT | /api/v1/plans/:id | 更新套餐 |
| DELETE | /api/v1/plans/:id | 删除套餐 |

### Provider 管理

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | /api/v1/providers | 列出所有 Provider |
| GET | /api/v1/providers/:id | 获取 Provider 详情 |

### Agent 管理

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | /api/v1/agents | 列出所有 Agent |
| POST | /api/v1/agents/bind | 绑定 Agent 到套餐 |
| DELETE | /api/v1/agents/unbind | 解除绑定 |

### Fallback

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | /api/v1/fallback | 获取 Fallback 配置 |
| PUT | /api/v1/fallback | 更新 Fallback 配置 |

### Quota

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | /api/v1/quota | 获取配额状态 |
| PUT | /api/v1/quota | 设置配额 |

### API Key

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | /api/v1/apikey/test | 测试 API Key |
| POST | /api/v1/apikey/open-page | 打开 Key 页面 |

### 插件

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | /api/v1/plugins | 列出插件 |
| POST | /api/v1/plugins | 安装插件 |
| DELETE | /api/v1/plugins/:id | 卸载插件 |

### 日志

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | /api/v1/logs | 获取请求日志 |
| GET | /api/v1/logs/:id | 获取日志详情 |

### 配置

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | /api/v1/config | 获取配置 |
| PUT | /api/v1/config | 更新配置 |

### 统计

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | /api/v1/stats | 获取使用统计 |

## 代理端点

### Anthropic 风格

```
POST /v1/messages
```

请求格式参考 [Anthropic Messages API](https://docs.anthropic.com/en/api/messages)

### OpenAI 风格

```
POST /v1/chat/completions
```

请求格式参考 [OpenAI Chat Completions API](https://platform.openai.com/docs/api-reference/chat)

## 中间件

- **_rate-limit**: 每分钟 100 请求
- **auth**: 可选的 API Key 认证
- **cors**: 跨域支持
- **trace**: 请求追踪

## 错误响应

```json
{
  "error": {
    "type": "invalid_request_error",
    "message": "错误描述"
  }
}
```