# @agent-gateway/node 使用文档

## 概述

@agent-gateway/node 是 agent-gateway 的 Node.js 绑定包，基于 NAPI-RS 构建。

## 安装

```bash
npm install @agent-gateway/node
# 或
bun add @agent-gateway/node
```

## 使用

```typescript
import { getGateway, createGateway, hasNativeBindings } from '@agent-gateway/node';

// 获取默认 gateway 实例
const gateway = getGateway();

// 创建新实例
const gateway2 = createGateway();

// 检查是否使用原生绑定
if (hasNativeBindings()) {
  console.log('Using native bindings');
} else {
  console.log('Using mock implementation');
}

// 调用 API
const providers = await gateway.listProviders();
const plans = await gateway.listPlans();
const health = gateway.health();
```

## 完整示例

```typescript
import { getGateway, createGateway } from '@agent-gateway/node';

async function main() {
  const gateway = getGateway();

  // 列出 Provider
  const providers = await gateway.listProviders();
  console.log('Providers:', providers);

  // 创建套餐
  const plan = await gateway.createPlan({
    provider_id: 'anthropic',
    plan_id: 'claude-sonnet-4-20250514',
    name: 'My Plan',
    api_key: process.env.ANTHROPIC_API_KEY!,
    selected_model_id: 'claude-sonnet-4-20250514',
  });

  console.log('Created plan:', plan);

  // 测试连接
  const result = await gateway.testPlan(plan.id);
  console.log('Test result:', result);

  // 获取配额
  const quota = await gateway.getQuotaUsage(plan.id);
  console.log('Quota:', quota);

  // 健康检查
  const health = gateway.health();
  console.log('Health:', health);
}

main().catch(console.error);
```

## API Key 验证

自动检测以下格式的 API Key:

- `sk-` (OpenAI)
- `sk-ant-` (Anthropic)
- `sk-proj-` (OpenAI 项目)
- `AIza` (Google)
- `gsk_` (Google AI Studio)
- `kilo_` (Kilo Code)

```typescript
const gateway = getGateway();

// 检测剪贴板内容
const isValidKey = gateway.validateApiKey('sk-ant-abc123'); // true

// 脱敏显示
const masked = gateway.maskApiKey('sk-ant-xyz123'); // "sk-an...3123"
```

## 错误处理

```typescript
import { getGateway } from '@agent-gateway/node';

async function main() {
  const gateway = getGateway();

  try {
    const plan = await gateway.getPlan('non-existent');
    if (!plan) {
      console.error('Plan not found');
      return;
    }
  } catch (error) {
    console.error('Error:', error.message);
  }
}
```

## 事件监听

```typescript
import { getGateway } from '@agent-gateway/node';

const gateway = getGateway();

// 监听配额变化
gateway.on('quota:warning', (planId, usage) => {
  console.log(`Quota warning for ${planId}: ${usage}%`);
});
```