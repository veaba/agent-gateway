# @agent-gateway/core 使用文档

## 概述

@agent-gateway/core 是 agent-gateway 的 TypeScript 类型定义包。

## 安装

```bash
npm install @agent-gateway/core
# 或
bun add @agent-gateway/core
```

## 类型定义

### 枚举

```typescript
import { ApiFormat, PlanTier, ModelCapability, AgentConfigStatus, HealthStatus } from '@agent-gateway/core';

// API 格式
ApiFormat.Anthropic
ApiFormat.OpenAi
ApiFormat.Custom

// 套餐等级
PlanTier.Free
PlanTier.Pro
PlanTier.Enterprise
PlanTier.Custom

// 模型能力
ModelCapability.Code
ModelCapability.Reasoning
ModelCapability.LongContext
ModelCapability.ChineseOptimized
ModelCapability.Math
ModelCapability.Multimodal

// Agent 配置状态
AgentConfigStatus.NotConfigured
AgentConfigStatus.AutoConfigured
AgentConfigStatus.ManuallyConfigured
AgentConfigStatus.ConfigError
AgentConfigStatus.NeedsUpdate

// 健康状态
HealthStatus.Unknown
HealthStatus.Healthy
HealthStatus.Warning
HealthStatus.Error
HealthStatus.Disabled
```

### 接口

```typescript
import type { 
  ProviderInfo, 
  PlanInfo, 
  CreatePlanInput, 
  UpdatePlanInput,
  TestConnectionResult,
  FallbackConfigInfo,
  QuotaInfo,
  HealthResponse,
  IGateway
} from '@agent-gateway/core';
```

## IGateway 接口

```typescript
import { getGateway } from '@agent-gateway/node';

const gateway = getGateway();

// 列出所有 Provider
const providers = await gateway.listProviders();

// 获取 Provider
const provider = await gateway.getProvider('anthropic');

// 列出所有套餐
const plans = await gateway.listPlans();

// 获取套餐
const plan = await gateway.getPlan('my-plan');

// 创建套餐
const newPlan = await gateway.createPlan({
  provider_id: 'anthropic',
  plan_id: 'claude-sonnet-4-20250514',
  name: 'My Plan',
  api_key: 'sk-ant-...',
  selected_model_id: 'claude-sonnet-4-20250514',
});

// 更新套餐
const updated = await gateway.updatePlan('my-plan', {
  name: 'New Name',
  enabled: true,
});

// 删除套餐
await gateway.deletePlan('my-plan');

// 测试套餐连接
const result = await gateway.testPlan('my-plan');
// { plan_id, success, message, latency_ms }

// 获取 Fallback 配置
const fallback = await gateway.getFallbackConfig();

// 设置 Fallback 启用状态
const config = await gateway.setFallbackEnabled(true);

// 设置 Fallback 优先级
const priority = await gateway.setFallbackPriority(['plan1', 'plan2']);

// 获取配额使用情况
const quota = await gateway.getQuotaUsage('my-plan');
// { plan_id, usage: { daily_used, monthly_used, rpm_used }, limits: { daily, monthly, rpm } }

// 设置配额限制
await gateway.setQuotaLimits('my-plan', 1000, 30000, 60);

// 验证 API Key
const valid = gateway.validateApiKey('sk-ant-...'); // true

// 脱敏 API Key
const masked = gateway.maskApiKey('sk-ant-abc123'); // "sk-an...c123"

// 健康检查
const health = gateway.health();
// { status: "ok", version: "0.1.0" }
```