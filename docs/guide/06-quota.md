# 第 6 章：配额管理

## 什么是配额

配额用于限制每个套餐的使用量，包括：
- **每日 tokens**：每天允许使用的 token 数量
- **每月 tokens**：每月允许使用的 token 数量
- **RPM**：每分钟请求数限制

## 查看配额状态

### CLI

```bash
agw quota status
```

输出：
```
Plan        Daily       Monthly     RPM    Used%
my-plan    100,000    3,000,000  60     45%
```

### API

```bash
curl http://127.0.0.1:8081/api/v1/quota
```

### Node.js

```javascript
const quota = await gateway.getQuotaUsage('my-plan');
console.log(quota);
// { plan_id, usage: { daily_used, monthly_used, rpm_used }, limits: {...} }
```

## 设置配额

### CLI

```bash
# 设置每日配额
agw quota set my-plan --daily 100000

# 设置每月配额
agw quota set my-plan --monthly 3000000

# 设置 RPM
agw quota set my-plan --rpm 60

# 同时设置多个
agw quota set my-plan --daily 100000 --monthly 3000000 --rpm 60
```

### API

```bash
curl -X PUT http://127.0.0.1:8081/api/v1/quota \
  -H "Content-Type: application/json" \
  -d '{
    "plan_id": "my-plan",
    "daily": 100000,
    "monthly": 3000000,
    "rpm": 60
  }'
```

### Node.js

```javascript
await gateway.setQuotaLimits('my-plan', 100000, 3000000, 60);
```

## 配额警告

设置配额使用百分比警告：

```bash
agw quota alert my-plan 80
```

当使用超过 80% 时，会收到警告通知。

## 配额用尽处理

当配额用尽时：

| 场景 | 处理 |
|------|------|
| 每日配额用尽 | 等待第二天恢复 |
| 每月配额用尽 | 等待下月恢复 |
| RPM 超限 | 等待下一分钟 |

日志输出：
```
[2026-05-01 10:00:00] Quota exceeded: daily limit (100,000)
```

## 配额继承

套餐可以从 Provider 模板继承默认配额：

```yaml
user_plans:
  - id: my-plan
    provider_id: anthropic
    # 可以覆盖默认值
    custom_quota_daily: 100000
```

## 统计查询

### API

```bash
# 获取特定时间范围的统计
curl "http://127.0.0.1:8081/api/v1/stats?plan_id=my-plan&from=2026-01-01&to=2026-05-01"
```

### 响应格式

```json
{
  "plan_id": "my-plan",
  "period": "2026-01-01 to 2026-05-01",
  "total_tokens": 2500000,
  "total_requests": 5000,
  "avg_latency_ms": 1200,
  "fallback_count": 10,
  "error_count": 5
}
```

## 下一步

- [第 7 章：API 参考](07-api.md) - REST API 完整参考
- [第 8 章：协议转换](08-converter.md) - Anthropic/OpenAI 转换