# agent-gateway 核心库使用文档

## 概述

agw-core 是 agent-gateway 的核心库，提供 Provider-Plan-Model-Agent 四层体系、自动降级、配额控制和协议转换功能。

## 模块结构

```
agw-core/
├── src/
│   ├── model.rs          # 核心数据模型
│   ├── model_types.rs   # 枚举类型定义
│   ├── business/       # 业务逻辑
│   ├── core/           # 网关核心
│   ├── storage/        # 存储层
│   ├── security/      # 安全模块
│   └── plugin/        # 插件系统
```

## 数据类型

### ApiFormat

API 协议格式:

```rust
use agw_core::ApiFormat;

ApiFormat::Anthropic   // Anthropic Messages API
ApiFormat::OpenAi     // OpenAI Chat Completions
ApiFormat::Custom     // 自定义格式
```

### PlanTier

套餐等级:

```rust
use agw_core::PlanTier;

PlanTier::Free       // 免费版
PlanTier::Pro        // 专业版
PlanTier::Enterprise // 企业版
PlanTier::Custom     // 自定义
```

### ModelCapability

模型能力:

```rust
use agw_core::ModelCapability;

ModelCapability::Code             // 代码生成
ModelCapability::Reasoning      // 推理
ModelCapability::LongContext    // 长上下文
ModelCapability::ChineseOptimized // 中文优化
ModelCapability::Math          // 数学
ModelCapability::Multimodal    // 多模态
```

### AgentConfigStatus

Agent 配置状态:

```rust
use agw_core::AgentConfigStatus;

AgentConfigStatus::NotConfigured      // 未配置
AgentConfigStatus::AutoConfigured    // 已自动配置
AgentConfigStatus::ManuallyConfigured // 已手动配置
AgentConfigStatus::ConfigError      // 配置错误
AgentConfigStatus::NeedsUpdate      // 需要更新
```

### HealthStatus

健康状态:

```rust
use agw_core::HealthStatus;

HealthStatus::Unknown   // 未知
HealthStatus::Healthy   // 健康
HealthStatus::Warning   // 警告
HealthStatus::Error     // 错误
HealthStatus::Disabled // 已禁用
```

### FallbackTrigger

Fallback 触发条件:

```rust
use agw_core::FallbackTrigger;

FallbackTrigger::RateLimit        // 429 速率限制
FallbackTrigger::ServerError      // 5xx 服务端错误
FallbackTrigger::ConnectionFailure // 连接失败
FallbackTrigger::Timeout       // 超时
FallbackTrigger::QuotaExceeded // 配额用尽
```

### PluginStatus / PluginType

插件状态和类型:

```rust
use agw_core::{PluginStatus, PluginType};

PluginStatus::Installed  // 已安装
PluginStatus::Enabled    // 已启用
PluginStatus::Disabled  // 已禁用
PluginStatus::Error     // 错误

PluginType::Provider   // Provider 扩展
PluginType::Transform   // 转换插件
PluginType::Tool        // 工具插件
```

## 核心结构

### UserPlan

用户套餐:

```rust
use agw_core::UserPlan;

let plan = UserPlan::new(
    "my-plan".to_string(),
    "anthropic".to_string(),
    "claude-sonnet-4-20250514".to_string(),
    "My Plan".to_string(),
    "sk-ant-...".to_string(),
    "claude-sonnet-4-20250514".to_string(),
);
```

### RequestContext

请求上下文:

```rust
use agw_core::RequestContext;

let ctx = RequestContext {
    user_plan: plan,
    agent_tool: Some("claude-code".to_string()),
    endpoint_format: ApiFormat::Anthropic,
    needs_conversion: false,
    target_format: ApiFormat::Anthropic,
};
```

## 业务模块

### PlanManager

套餐管理:

```rust
use agw_core::business::PlanManager;
use agw_core::storage::ConfigStore;
use std::sync::Arc;

let config_store = Arc::new(ConfigStore::new()?);
let manager = PlanManager::new(config_store);

// 获取所有套餐
let plans = manager.get_all_plans().await?;

// 添加套餐
manager.add_plan(plan).await?;

// 测试连接
let valid = manager.test_connection(plan_id).await?;
```

### ProviderEngine

Provider 引擎:

```rust
use agw_core::business::ProviderEngine;

let engine = ProviderEngine::new();

// 获取 Provider
let provider = engine.get_provider("anthropic").await?;

// 获取套餐模板
let plans = engine.get_plan_templates("anthropic").await?;
```

### FallbackManager

降级管理:

```rust
use agw_core::business::FallbackManager;
use agw_core::FallbackConfig;

let config = FallbackConfig::default();
let manager = FallbackManager::new(config);

// 检查是否需要降级
let should_fallback = manager.should_fallback(&error).await?;

// 获取下一个候选
let next_plan = manager.get_next_candidate(current_plan).await?;
```

### QuotaManager

配额管理:

```rust
use agw_core::business::QuotaManager;

let manager = QuotaManager::new();

// 检查配额
let available = manager.check_quota(plan_id).await?;

// 消耗配额
manager.consume(plan_id, tokens).await?;
```

## 网关模块

### Gateway

请求网关:

```rust
use agw_core::core::Gateway;

let gateway = Gateway::new();

// 处理请求
let response = gateway.handle_request(request, ctx).await?;
```

### Unified Router

统一路由合并，用于嵌入式服务器：

```rust
use agw_core::core::unified_router::create_unified_app;
use agw_core::core::GatewayState;
use std::sync::Arc;

// 创建 GatewayState
let gateway_state = Arc::new(GatewayState::new().await?);

// 创建管理 API 路由 (来自 agw-api)
let management_router = agw_api::handlers::create_router(api_state);

// 合并 proxy + 管理 API 路由
let unified_app = create_unified_app(gateway_state, management_router).await;

// 启动统一服务器
let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await?;
axum::serve(listener, unified_app).await?;
```

**合并后端点：**
- `/health` - 健康检查
- `/v1/messages` - Anthropic Messages API
- `/v1/chat/completions` - OpenAI Chat Completions
- `/api/v1/*` - 管理 API (30+ 端点)

### ProtocolConverter

协议转换:

```rust
use agw_core::core::converter::{
    AnthropicToOpenaiConverter,
    OpenaiToAnthropicConverter,
};

// Anthropic -> OpenAI
let openai_request = AnthropicToOpenaiConverter::convert(anthropic_request)?;

// OpenAI -> Anthropic
let anthropic_request = OpenaiToAnthropicConverter::convert(openai_request)?;
```

## 存储模块

### ConfigStore

配置存储:

```rust
use agw_core::storage::ConfigStore;

let store = ConfigStore::new()?;

// 加载配置
let config = store.load_user_plans()?;
let providers = store.load_providers()?;
```

### SqliteStore

SQLite 存储:

```rust
use agw_core::storage::sqlite::SqliteStore;

let store = SqliteStore::new("data.db")?;

// 记录请求
store.record_request(&log).await?;

// 查询统计
let stats = store.get_stats(plan_id, from, to).await?;
```

## 安全模块

### ApiKeyHelper

API Key 助手:

```rust
use agw_core::security::ApiKeyHelper;

// 检测剪贴板中的 API Key
let key_type = ApiKeyHelper::detect_from_clipboard()?;

// 从环境变量读取
let key = ApiKeyHelper::load_from_env("ANTHROPIC_API_KEY")?;
```

### Encryption

加密模块:

```rust
use agw_core::security::encryption::encrypt;
use agw_core::security::encryption::decrypt;

let encrypted = encrypt(plaintext, key)?;
let decrypted = decrypt(encrypted, key)?;
```

## 初始化的

```rust
use agw_core::init;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init().await?;
    // 你的代码
    Ok(())
}
```