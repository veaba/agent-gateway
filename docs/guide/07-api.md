# 第 7 章：REST API 参考

## 基础信息

- **基础 URL**: `http://127.0.0.1:8081`
- **协议**: HTTP
- **格式**: JSON
- **认证**: 可选的 API Key

## 认证

### 方式一：API Key 头

```bash
curl -H "Authorization: Bearer YOUR_API_KEY" \
  http://127.0.0.1:8081/api/v1/plans
```

### 方式二：Query 参数

```bash
curl "http://127.0.0.1:8081/api/v1/plans?key=YOUR_API_KEY"
```

## 公共端点

### 健康检查

```http
GET /health
```

响应：
```json
{"status": "ok", "version": "0.1.0"}
```

### 代理请求

```http
POST /v1/messages        # Anthropic 风格
POST /v1/chat/completions  # OpenAI 风格
```

## Plan 端点

### 列表

```http
GET /api/v1/plans
```

响应：
```json
[
  {
    "id": "my-plan",
    "provider_id": "anthropic",
    "plan_id": "claude-sonnet-4-20250514",
    "name": "My Plan",
    "selected_model_id": "claude-sonnet-4-20250514",
    "enabled": true,
    "priority": 1,
    "health_status": "healthy"
  }
]
```

### 创建

```http
POST /api/v1/plans
```

请求体：
```json
{
  "provider_id": "anthropic",
  "plan_id": "claude-sonnet-4-20250514",
  "name": "My Plan",
  "api_key": "sk-ant-...",
  "selected_model_id": "claude-sonnet-4-20250514"
}
```

### 获取

```http
GET /api/v1/plans/:id
```

### 更新

```http
PUT /api/v1/plans/:id
```

请求体：
```json
{
  "name": "New Name",
  "enabled": true,
  "priority": 2
}
```

### 删除

```http
DELETE /api/v1/plans/:id
```

### 测试连接

```http
GET /api/v1/plans/:id/test
```

响应：
```json
{
  "plan_id": "my-plan",
  "success": true,
  "message": "Connection successful",
  "latency_ms": 150
}
```

## Provider 端点

### 列表

```http
GET /api/v1/providers
```

### 获取

```http
GET /api/v1/providers/:id
```

## Fallback 端点

### 获取配置

```http
GET /api/v1/fallback
```

响应：
```json
{
  "enabled": true,
  "max_attempts": 3,
  "priority_order": ["plan1", "plan2"]
}
```

### 更新配置

```http
PUT /api/v1/fallback
```

请求体：
```json
{
  "enabled": true,
  "max_attempts": 3,
  "priority_order": ["plan1", "plan2"]
}
```

## Quota 端点

### 获取配额

```http
GET /api/v1/quota?plan_id=:id
```

响应：
```json
{
  "plan_id": "my-plan",
  "usage": {
    "daily_used": 45000,
    "monthly_used": 500000,
    "rpm_used": 30
  },
  "limits": {
    "daily": 100000,
    "monthly": 3000000,
    "rpm": 60
  }
}
```

### 设置配额

```http
PUT /api/v1/quota
```

请求体：
```json
{
  "plan_id": "my-plan",
  "daily": 100000,
  "monthly": 3000000,
  "rpm": 60
}
```

## Agent 端点

### 列表

```http
GET /api/v1/agents
```

### 绑定

```http
POST /api/v1/agents/bind
```

### 解除绑定

```http
DELETE /api/v1/agents/unbind
```

## 日志端点

### 列表

```http
GET /api/v1/logs
```

Query 参数：
- `plan_id` - 套餐 ID
- `level` - 日志级别 (debug, info, warn, error)
- `limit` - 返回数量
- `offset` - 偏移量

### 详情

```http
GET /api/v1/logs/:id
```

## 插件端点

### 列表

```http
GET /api/v1/plugins
```

### 安装

```http
POST /api/v1/plugins
```

### 卸载

```http
DELETE /api/v1/plugins/:id
```

## 配置端点

### 获取

```http
GET /api/v1/config
```

### 更新

```http
PUT /api/v1/config
```

## 统计端点

```http
GET /api/v1/stats?plan_id=:id&from=:from&to=:to
```

## 错误响应

错误响应格式：
```json
{
  "error": {
    "type": "invalid_request_error",
    "message": "错误描述",
    "code": "invalid_plan_id"
  }
}
```

常见错误类型：
- `invalid_request_error` - 请求参数错误
- `authentication_error` - 认证失败
- `rate_limit_error` - 频率超限
- `quota_exceeded_error` - 配额用尽
- `server_error` - 服务端错误