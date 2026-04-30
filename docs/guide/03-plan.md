# 第 3 章：添加套餐

## 什么是套餐

套餐（Plan）是 agent-gateway 的核心配置单元，包含：
- Provider（AI 服务商）
- API Key（访问凭证）
- 选择的模型
- 绑定的 Agent

## 添加套餐

### 方式一：交互式向导

```bash
agw plan add --wizard
```

向导会引导你完成：
1. 选择 Provider
2. 选择套餐
3. 输入 API Key
4. 选择模型
5. 配置配额（可选）

### 方式二：命令行参数

```bash
agw plan add \
  --provider anthropic \
  --plan claude-sonnet-4-20250514 \
  --name "My Plan" \
  --model claude-sonnet-4-20250514
```

### 方式三：API 请求

```bash
curl -X POST http://127.0.0.1:8081/api/v1/plans \
  -H "Content-Type: application/json" \
  -d '{
    "provider_id": "anthropic",
    "plan_id": "claude-sonnet-4-20250514",
    "name": "My Plan",
    "api_key": "sk-ant-...",
    "selected_model_id": "claude-sonnet-4-20250514"
  }'
```

### 方式四：Node.js

```javascript
import { getGateway } from '@agent-gateway/node';

const gateway = getGateway();

const plan = await gateway.createPlan({
  provider_id: 'anthropic',
  plan_id: 'claude-sonnet-4-20250514',
  name: 'My Plan',
  api_key: process.env.ANTHROPIC_API_KEY,
  selected_model_id: 'claude-sonnet-4-20250514',
});

console.log('Created:', plan.id);
```

## 获取 API Key

### 打开 Provider 页面

```bash
agw key open-page anthropic
```

这会在浏览器中打开 Anthropic 的 API Key 页面。

### 剪贴板自动检测

当你在其他应用中复制 API Key 后，回到 GUI 或使用：

```bash
# GUI 会自动检测剪贴板中的 Key
# 支持格式：sk-, sk-ant-, sk-proj-, AIza, gsk_, kilo_
```

### 从环境变量

```bash
# 设置环境变量
export ANTHROPIC_API_KEY="sk-ant-..."

# 测试
agw key test my-plan
```

## 验证套餐

### 测试连接

```bash
agw plan test my-plan
```

成功输出：
```
Testing API key for plan: my-plan...
✅ API key is valid!
```

### 查看套餐列表

```bash
agw plan list
```

输出：
```
ID          Provider    Model                    Status
my-plan     anthropic   claude-sonnet-4-20250514  healthy
```

### 设置默认套餐

```bash
agw plan use my-plan
```

## 管理套餐

### 更新套餐

```bash
agw plan update my-plan --name "New Name" --model claude-opus-4-20250514
```

### 启用/禁用

```bash
# 禁用
agw plan disable my-plan

# 启用
agw plan enable my-plan
```

### 删除套餐

```bash
agw plan delete my-plan
```

## 配额配置

```bash
# 设置每日配额 (token)
agw quota set my-plan --daily 100000

# 设置每月配额 (token)
agw quota set my-plan --monthly 1000000

# 设置请求速率限制 (每分钟)
agw quota set my-plan --rpm 60
```

## 下一步

- [第 4 章：使用 CLI](04-cli.md) - 命令行操作指南
- [第 5 章：Fallback 配置](05-fallback.md) - 配置自动故障转移