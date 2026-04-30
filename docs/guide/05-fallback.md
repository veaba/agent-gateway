# 第 5 章：Fallback 配置

## 什么是 Fallback

Fallback 是 agent-gateway 的自动故障转移机制。当主 Provider 出现错误时，自动切换到备用 Provider。

## Fallback 触发条件

| 触发条件 | 说明 |
|----------|------|
| RateLimit (429) | 请求频率超限 |
| ServerError (5xx) | 服务端错误 |
| ConnectionFailure | 连接失败 |
| Timeout | 请求超时 |
| QuotaExceeded | 配额用尽 |

## 启用 Fallback

### CLI

```bash
# 启用
agw fallback on

# 禁用
agw fallback off
```

### API

```bash
# 获取配置
curl http://127.0.0.1:8081/api/v1/fallback

# 启用
curl -X PUT http://127.0.0.1:8081/api/v1/fallback \
  -H "Content-Type: application/json" \
  -d '{"enabled": true}'
```

### Node.js

```javascript
const config = await gateway.setFallbackEnabled(true);
```

## 设置优先级

当有多个套餐时，可以设置 Fallback 顺序。

### CLI

```bash
agw fallback set primary-plan,backup-plan-1,backup-plan-2
```

### API

```bash
curl -X PUT http://127.0.0.1:8081/api/v1/fallback \
  -H "Content-Type: application/json" \
  -d '{
    "enabled": true,
    "priority_order": ["primary", "backup1", "backup2"]
  }'
```

### Node.js

```javascript
await gateway.setFallbackPriority(['primary', 'backup1', 'backup2']);
```

## 最大尝试次数

```bash
# 设置最多尝试 3 次（默认）
agw fallback set --max-attempts 3
```

注意：超过最大次数后，请求会返回错误，防止无限循环。

## Fallback 日志

查看 Fallback 触发情况：

```bash
agw log show --fallback
```

输出示例：
```
[2026-05-01 10:30:15] Fallback triggered: RateLimit (429) on plan-1
[2026-05-01 10:30:15] Switching to plan-2
[2026-05-01 10:30:16] Request succeeded on plan-2
```

## 配置示例

### 完整配置

```yaml
# config.yaml
fallback:
  enabled: true
  max_attempts: 3
  priority_order:
    - anthropic-sonnet
    - openai-gpt4o
    - google-gemini
```

### 套餐配置

```yaml
# 每个套餐可以设置优先级
user_plans:
  - id: primary
    provider_id: anthropic
    priority: 1
    enabled: true
  - id: backup
    provider_id: openai
    priority: 2
    enabled: true
```

## 最佳实践

1. **优先级顺序**：将最稳定的 Provider 放在前面
2. **模型兼容性**：确保备选 Provider 支持相同模型
3. **配额分配**：为 Fallback 预留配额
4. **监控**：定期检查 Fallback 触发频率

## 故障排查

### Fallback 不触发

检查：
- Fallback 是否已启用
- 套餐是否都已启用
- 是否在优先级列表中

### 无限循环

确保：
- `max_attempts` 设置正确
- 套餐之间不互相触发

## 下一步

- [第 6 章：配额管理](06-quota.md) - 了解配额控制
- [第 7 章：API 参考](07-api.md) - REST API 完整参考