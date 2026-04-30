# agent-gateway 完整实施方案

## 一、项目概述

agent-gateway 是一款面向开发者的多AI编码工具统一网关。开发者只需配置一次网关地址，即可无缝使用 Claude Code、Kimi Code、OpenCode、Kilo CLI 等AI编码工具。网关提供多套餐管理、智能Fallback降级、配额管控、CLI+GUI双模式操作、插件扩展能力，所有数据本地存储，开箱即用。

**核心定位**：

- 对终端用户：一个桌面应用，双击即用，套餐式配置，5秒上手
- 对开发者：一组灵活的模块，可按需组合
- 对生态：一个可插拔的插件平台，支持任意AI工具接入

**产品形态矩阵**：

| 形态              | 说明                          | 适用场景              |
|-------------------|-------------------------------|-----------------------|
| **Desktop**（默认） | 完整桌面应用，CLI+GUI+网关一体 | 普通开发者日常使用    |
| CLI Only          | 命令行工具+网关，无GUI         | 服务器部署、自动化脚本 |
| API Only          | 纯网关服务，HTTP API供外部调用 | 第三方集成、团队共享   |
| Library (Crate)   | Rust库，被其他项目引用         | Rust开发者二次开发    |
| Node.js Package   | npm包，提供Node.js绑定         | 前端/Node.js开发者    |
| Plugin            | 动态扩展，接入新AI工具         | 生态扩展              |

---

## 二、核心设计变化：Provider-Plan-Model-Agent 联动体系

### 2.1 传统配置 vs 新方案

| 维度       | 传统（逐个字段填写）                               | 新方案（套餐式向导）                           |
|------------|--------------------------------------------------|----------------------------------------------|
| 用户操作   | 手动输入Base URL、API Key、模型ID、工具类型等8+字段 | 选择Provider → 选择Plan → 粘贴API Key → 完成 |
| 认知负担   | 需要了解各Provider的API格式、模型名称、端点地址    | 只需了解「我想用哪家」和「我是什么套餐」         |
| 出错概率   | 高（URL拼写错误、模型ID错误、格式不匹配）            | 极低（内置校验，自动填充所有技术字段）          |
| 新用户上手 | 需要阅读文档，理解概念                            | 向导式3步完成                                |

### 2.2 概念模型：四层联动

```txt
┌────────────────────────────────────────────────────────────────┐
│  Provider（提供商）                                             │
│  ──────────────                                                │
│  如：Alaya、Anthropic、Kimi、OpenAI、OpenCode、Kilo            │
│  · 内置所有技术配置（Base URL、API格式、端点、文档链接）       │
│  · 包含多个 Coding Plan                                        │
│  · 包含支持的模型列表                                          │
│  · 包含支持的 Agent 工具列表                                   │
│  · 包含获取API Key的直达链接                                   │
├────────────────────────────────────────────────────────────────┤
│  Coding Plan（套餐方案）                                        │
│  ────────────────                                              │
│  如：Alaya Lite、Alaya Plus、Alaya Max                         │
│  · 属于特定 Provider                                           │
│  · 包含该Plan支持的模型子集                                     │
│  · 包含该Plan支持的Agent工具子集                                │
│  · 包含配额限制（日/月/RPM）                                    │
│  · 包含优先级（Fallback排序）                                   │
├────────────────────────────────────────────────────────────────┤
│  Model（模型）                                                 │
│  ──────                                                        │
│  如：MiniMax-2.5、GLM-5、DeepSeek-V4-Pro、claude-sonnet-4-5    │
│  · 属于特定 Provider                                           │
│  · 模型ID（用于API请求）                                        │
│  · 上下文长度、能力标签（代码/推理/多模态）                      │
├────────────────────────────────────────────────────────────────┤
│  Agent Tool（编码工具）                                         │
│  ───────────────                                               │
│  如：Claude Code、Kimi CLI、OpenCode、Kilo CLI                 │
│  · 独立的可执行工具                                              │
│  · 连接到网关的方式（环境变量/配置文件）                         │
│  · 支持的API格式要求                                             │
└────────────────────────────────────────────────────────────────┘

规则：
· 选择 Provider → 展示该 Provider 的所有 Coding Plan
· 选择 Coding Plan → 展示该 Plan 支持的 Model 列表（可选，默认用Plan推荐模型）
· 选择 Coding Plan → 展示该 Plan 支持的 Agent Tool 列表（可选，可多选）
· 只有选定了 Agent Tool，才需要为该 Plan 配置 API Key（用于该工具连接网关）
· 一个 Coding Plan 实例可绑定多个 Agent Tool（如 Lite Plan 同时给 Claude Code 用）
```

### 2.3 实例数据：Alaya Provider

```yaml
# Provider: Alaya（内置配置）
provider_id: "alaya"
name: "Alaya"
description: "Alaya AI Coding Platform"
logo_url: "https://.../alaya-logo.svg"
homepage: "https://alaya.ai"
docs_url: "https://docs.alaya.ai"
get_api_key_url: "https://console.alaya.ai/settings/api-keys"
setup_guide_url: "https://docs.alaya.ai/getting-started"
api_format: "anthropic"           # 默认API格式
base_url_template: "https://api.alaya.com/coding/{plan_id}"

# Alaya 提供的 Coding Plans
coding_plans:
  - plan_id: "alaya-lite"
    name: "Lite"
    description: "轻量版，适合个人日常开发"
    tier: "free"                    # free / pro / enterprise
    supported_models:               # 该Plan支持的模型
      - model_id: "minimax-2.5"
        name: "MiniMax-2.5"
        context_length: 256000
        capabilities: ["code", "reasoning"]
      - model_id: "minimax-2.5-pro"
        name: "MiniMax-2.5-Pro"
        context_length: 256000
        capabilities: ["code", "reasoning", "long-context"]
    supported_agents:               # 该Plan支持的Agent工具
      - agent_id: "claude-code"
        name: "Claude Code"
        requirement: "需要 Anthropic Messages API 格式"
    default_model: "minimax-2.5"
    default_agent: "claude-code"
    quota_daily: 100
    quota_monthly: 2000
    rpm_limit: 20
    price_per_1k_tokens: "$0.001"
    
  - plan_id: "alaya-plus"
    name: "Plus"
    description: "进阶版，适合专业开发者"
    tier: "pro"
    supported_models:
      - model_id: "minimax-2.5"
        name: "MiniMax-2.5"
      - model_id: "minimax-2.5-pro"
        name: "MiniMax-2.5-Pro"
      - model_id: "glm-5"
        name: "GLM-5"
        context_length: 128000
        capabilities: ["code", "reasoning", "chinese-optimized"]
    supported_agents:
      - agent_id: "claude-code"
      - agent_id: "kimi-cli"
    default_model: "glm-5"
    default_agent: "claude-code"
    quota_daily: 500
    quota_monthly: 10000
    rpm_limit: 60
    
  - plan_id: "alaya-max"
    name: "Max"
    description: "旗舰版，适合团队和高频使用"
    tier: "enterprise"
    supported_models:
      - model_id: "minimax-2.5"
      - model_id: "minimax-2.5-pro"
      - model_id: "glm-5"
      - model_id: "glm-5.1"
        name: "GLM-5.1"
        context_length: 256000
      - model_id: "deepseek-v4-pro"
        name: "DeepSeek-V4-Pro"
        context_length: 128000
        capabilities: ["code", "reasoning", "math"]
    supported_agents:
      - agent_id: "claude-code"
      - agent_id: "kimi-cli"
      - agent_id: "opencode"
      - agent_id: "kilo-cli"
    default_model: "deepseek-v4-pro"
    default_agent: "claude-code"
    quota_daily: 2000
    quota_monthly: 50000
    rpm_limit: 200
```

### 2.4 实例数据：Anthropic Provider（直连型）

```yaml
# Provider: Anthropic（直连型，无Coding Plan分层）
provider_id: "anthropic"
name: "Anthropic"
description: "Claude API 官方直连"
logo_url: "https://.../anthropic-logo.svg"
homepage: "https://anthropic.com"
docs_url: "https://docs.anthropic.com"
get_api_key_url: "https://console.anthropic.com/settings/keys"
setup_guide_url: "https://docs.anthropic.com/claude-code/setup"
api_format: "anthropic"
base_url: "https://api.anthropic.com"

# 直连型Provider只有一个默认Plan
coding_plans:
  - plan_id: "anthropic-default"
    name: "Anthropic API"
    description: "Anthropic官方API直连"
    tier: "custom"
    supported_models:
      - model_id: "claude-sonnet-4-5"
        name: "Claude Sonnet 4.5"
      - model_id: "claude-opus-4"
        name: "Claude Opus 4"
    supported_agents:
      - agent_id: "claude-code"
    default_model: "claude-sonnet-4-5"
    default_agent: "claude-code"
    quota_daily: null          # 用户自定义
    quota_monthly: null
    rpm_limit: null
```

---

## 三、API Key 获取方案（降低门槛的关键）

### 3.1 方案总览

| 方案                  | 描述                                                             | 适用场景                | 复杂度 |
|-----------------------|------------------------------------------------------------------|-------------------------|--------|
| **方案A：一键直达**    | GUI点击"获取API Key"，唤起系统浏览器打开Provider的API Key管理页面 | 所有Provider            | 低     |
| **方案B：内置浏览器**  | GUI内嵌webview，用户在应用内完成登录和创建Key                     | 支持OAuth的Provider     | 中     |
| **方案C：剪贴板监听**  | 用户从网页复制AK后，应用自动检测并提示粘贴                        | 所有Provider            | 低     |
| **方案D：OAuth自动化** | 完整的OAuth流程，应用自动获取和刷新Token                          | 支持OAuth 2.0的Provider | 高     |
| **方案E：QR码扫描**    | 手机扫码在手机上完成授权，回调到桌面应用                          | 移动端友好型Provider    | 中     |

### 3.2 推荐实现：方案A + 方案C 组合

**原因**：

- 方案A最简单可靠，任何有API管理页面的Provider都支持
- 方案C作为辅助，捕获用户从网页复制的AK，减少手动粘贴步骤
- 方案B/D/E可作为后续迭代增强

### 3.3 方案A：一键直达浏览器

**实现**：

```rust
// crates/agw-core/src/security/api_key_helper.rs

pub struct ApiKeyHelper;

impl ApiKeyHelper {
    /// 打开Provider的API Key获取页面
    pub fn open_get_key_page(provider: &Provider) -> Result<()> {
        let url = provider.get_api_key_url
            .as_ref()
            .ok_or_else(|| anyhow!("Provider does not support automatic key page"))?;
        
        open::that(url)?;  // 使用 `open` crate 唤起系统默认浏览器
        Ok(())
    }
    
    /// 打开Provider的配置指南
    pub fn open_setup_guide(provider: &Provider) -> Result<()> {
        if let Some(url) = &provider.setup_guide_url {
            open::that(url)?;
        }
        Ok(())
    }
}
```

```typescript
// web/src/components/ApiKeyInput.vue
// GUI界面中的API Key输入组件

template>
  <div class="api-key-input">
    <el-input
      v-model="apiKey"
      type="password"
      show-password
      placeholder="粘贴您的 API Key"
    />
    <el-button-group>
      <!-- 一键直达获取页面 -->
      <el-button
        type="primary"
        @click="openGetKeyPage"
        :disabled="!provider.getApiKeyUrl"
      >
        <el-icon><Link /></el-icon>
        去获取 API Key
      </el-button>
      
      <!-- 查看配置指南 -->
      <el-button
        @click="openSetupGuide"
        :disabled="!provider.setupGuideUrl"
        <el-icon><Document /></el-icon>
        配置指南
      </el-button>
      
      <!-- 测试连接 -->
      >
      <el-button
        type="success"
        @click="testConnection"
        :loading="testing"
        <el-icon><Check /></el-icon>
        测试连接
      </el-button>
    </el-button-group>
    
    <!-- 剪贴板自动检测提示 -->
    <el-alert
      v-if="clipboardDetected"
      type="info"
      :closable="false"
      >
      检测到剪贴板中有 API Key，是否使用？
      <el-button size="small" @click="useClipboardKey">使用</el-button>
      <el-button size="small" @click="ignoreClipboardKey">忽略</el-button>
    </el-alert>
  </div>
</template>
```

**GUI交互流程**：

```
用户点击"去获取 API Key"
    │
    ▼
调用系统浏览器打开 https://console.alaya.ai/settings/api-keys
    >
    │
    ▼
用户在浏览器中登录 → 创建新Key → 复制Key到剪贴板
    │
    ▼
回到agent-gateway GUI
    │
    ▼
应用检测到剪贴板内容符合API Key格式（如 sk-开头、长度匹配）
    │
    ▼
弹出提示："检测到剪贴板中的 API Key，是否使用？"
    │
    ▼
用户点击"使用" → 自动填充到输入框
```

### 3.4 方案C：剪贴板监听

```rust
// crates/agw-gui/src/clipboard.rs（Tauri侧）
use tauri::Manager;

#[tauri::command]
async fn monitor_clipboard(app: tauri::AppHandle) -> Result<String, String> {
    let clipboard = app.clipboard();
    let content = clipboard.read_text().map_err(|e| e.to_string())?;
    
    // 检测是否是API Key格式
    if is_likely_api_key(&content) {
        return Ok(content);
    }
    
    Ok(String::new())
}

fn is_likely_api_key(content: &str) -> bool {
    let trimmed = content.trim();
    // 常见API Key前缀检测
    let prefixes = [
        "sk-",        // OpenAI / 通用
        "sk-ant-",    // Anthropic
        "sk-proj-",   // OpenAI Project
        "AIza",       // Google
        "gsk_",       // Groq
        "kilo_",      // Kilo
    ];
    
    prefixes.iter().any(|p| trimmed.starts_with(p)) && trimmed.len() > 20
}
```

```typescript
// frontend/src/composables/useClipboardMonitor.ts

import { ref, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';

export function useClipboardMonitor() {
  const detectedKey = ref<string>('');
  let interval: number;

  onMounted(() => {
    // 每2秒检测一次剪贴板
    interval = window.setInterval(async () => {
      const content = await invoke<string>('monitor_clipboard');
      if (content && content !== detectedKey.value) {
        detectedKey.value = content;
      }
    }, 2000);
  });

  onUnmounted(() => {
    clearInterval(interval);
  });

  return { detectedKey };
}
```

### 3.5 方案D：OAuth自动化（预留设计）

```rust
// crates/agw-core/src/security/oauth.rs（预留）

pub struct OAuthFlow {
    provider: Provider,
    client_id: String,
    redirect_uri: String,
}

impl OAuthFlow {
    /// 启动OAuth流程
    pub async fn start(&self) -> Result<AuthUrl> {
        // 1. 生成PKCE verifier
        // 2. 构建授权URL
        // 3. 启动本地回调服务器（127.0.0.1:随机端口）
        // 4. 打开浏览器访问授权URL
        // 5. 等待回调，获取authorization code
        // 6. 用code换取access_token和refresh_token
        // 7. 存储token
        unimplemented!("OAuth automation reserved for v1.x")
    }
}
```

### 3.6 API Key 安全性处理

```rust
// 输入处理流程
pub fn sanitize_api_key(input: &str) -> String {
    input.trim().to_string()
}

// 格式验证
pub fn validate_api_key_format(provider: &Provider, key: &str) -> Result<()> {
    match provider.provider_id.as_str() {
        "anthropic" => {
            if !key.starts_with("sk-ant-") {
                bail!("Anthropic API Key should start with 'sk-ant-'");
            }
        }
        "openai" => {
            if !key.starts_with("sk-") {
                bail!("OpenAI API Key should start with 'sk-'");
            }
        }
        _ => {}
    }
    
    if key.len() < 20 {
        bail!("API Key seems too short, please check");
    }
    
    Ok(())
}
```

---

## 四、Provider 配置更新机制

### 4.1 内置Provider配置

```rust
// crates/agw-core/src/business/provider_builtin.rs

/// 内置Provider配置，随应用发布
pub const BUILTIN_PROVIDERS: &[ProviderTemplate] = &[
    ProviderTemplate {
        provider_id: "alaya",
        name: "Alaya",
        // ... 完整配置见上方
    },
    ProviderTemplate {
        provider_id: "anthropic",
        name: "Anthropic",
        // ...
    },
    ProviderTemplate {
        provider_id: "kimi",
        name: "Kimi",
        // ...
    },
    ProviderTemplate {
        provider_id: "openai",
        name: "OpenAI",
        // ...
    },
    ProviderTemplate {
        provider_id: "opencode",
        name: "OpenCode",
        // ...
    },
    ProviderTemplate {
        provider_id: "kilo",
        name: "Kilo",
        // ...
    },
];
```

### 4.2 远程更新机制

```rust
pub struct ProviderUpdater {
    registry_url: String,
    local_version: String,
}

impl ProviderUpdater {
    /// 检查是否有更新
    pub async fn check_update(&self) -> Result<Option<ProviderRegistryUpdate>> {
        let remote = reqwest::get(&format!("{}/providers/index.json", self.registry_url))
            .await?
            .json::<RegistryIndex>()
            .await?;
        
        if remote.version != self.local_version {
            Ok(Some(ProviderRegistryUpdate {
                new_version: remote.version,
                changes: remote.changelog,
            }))
        } else {
            Ok(None)
        }
    }
    
    /// 下载并应用更新
    pub async fn apply_update(&self) -> Result<()> {
        let providers = reqwest::get(&format!("{}/providers/latest.yaml", self.registry_url))
            .await?
            .text()
            .await?;
        
        // 验证签名
        // 合并到本地配置（用户自定义配置不会被覆盖）
        // 更新内置Provider模板
        self.merge_providers(providers).await?;
        Ok(())
    }
}
```

**更新策略**：

- 启动时自动检查（每天最多一次）
- 仅更新Provider模板（plans、models、agents、URLs）
- 用户的API Key、自定义配额等配置不会被覆盖
- 新增Provider会直接出现在列表中

### 4.3 更新交互（GUI）

```
启动时检查
    │
    ▼
┌─────────────────────────┐
│ 发现Provider配置更新    │
│ · 新增：Kilo Provider   │
│ · 更新：Alaya Plus新增模型│
│ · 修复：Kimi base URL   │
└─────────────────────────┘
    │
    ▼
提示用户（非阻塞）
    │
    ├──── 立即更新 ──→ 下载 → 应用 → 刷新界面
    │
    └──── 稍后 ─────→ 下次启动时再次提示
```

---

## 五、GUI向导式配置流程

### 5.1 新用户首次添加套餐

```
┌─────────────────────────────────────────────────────────────────────────┐
│  添加 AI 编码套餐                                           [取消] [?] │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │ Step 1: 选择 Provider（AI服务商）                                 │   │
│  │                                                                  │   │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐       │   │
│  │  │  🤖      │  │  🔵      │  │  🟢      │  │  🟣      │       │   │
│  │  │  Alaya   │  │ Anthropic│  │   Kimi   │  │ OpenCode │  ...  │   │
│  │  │ 中国团队  │  │ Claude   │  │ 月之暗面 │  │          │       │   │
│  │  │ [推荐]   │  │  官方    │  │          │  │          │       │   │
│  │  └──────────┘  └──────────┘  └──────────┘  └──────────┘       │   │
│  │                                                                  │   │
│  │  [我没有账号，去注册 →]                                          │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │ Step 2: 选择 Coding Plan（套餐方案）                              │   │
│  │                                                                  │   │
│  │  ○ Lite   - 轻量版   (MiniMax-2.5, Claude Code)     免费 / 100次/日│   │
│  │  ● Plus   - 进阶版   (MiniMax-2.5, GLM-5, Claude Code) ¥29/月   │   │
│  │  ○ Max    - 旗舰版   (全模型, 全工具)              ¥99/月        │   │
│  │                                                                  │   │
│  │  [各方案对比 →]                                                  │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │ Step 3: 配置 Agent 工具（可多选）                                 │   │
│  │                                                                  │   │
│  │  [✓] Claude Code    [✓] Kimi CLI    [ ] OpenCode    [ ] Kilo  │   │
│  │                                                                  │   │
│  │  提示：Alaya Plus 支持 Claude Code 和 Kimi CLI                    │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │ Step 4: 获取并填入 API Key                                        │   │
│  │                                                                  │   │
│  │  API Key:  [sk-xxxxxxxxxxxxxxxx         ]                        │   │
│  │                                                                  │   │
│  │  [🌐 去 Alaya 控制台获取 API Key →]   ← 一键直达浏览器            │   │
│  │  [📖 查看配置指南]                                                 │   │
│  │                                                                  │   │
│  │  ⚡ 检测到剪贴板中的 API Key，是否使用？ [使用] [忽略]           │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │ Step 5: 高级设置（可选）                                          │   │
│  │                                                                  │   │
│  │  模型： [GLM-5 ▼]（Plan推荐，可切换为该Plan支持的其他模型）        │   │
│  │  日配额：[ 500 ]（留空使用Plan默认）                               │   │
│  │  优先级：[ 1 ]（Fallback排序，越小越优先）                        │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
│                              [← 上一步]  [完成 →]                       │
└─────────────────────────────────────────────────────────────────────────┘
```

### 5.2 已配置套餐展示

```
┌──────────────────────────────────────────────────────────────────────────┐
│  我的套餐                                                    [+ 添加套餐] │
│                                                                          │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │  🔵 Alaya Plus (当前使用)                                        │   │
│  │                                                                  │   │
│  │  Provider: Alaya                                                │   │
│  │  Plan: Plus（进阶版）                                            │   │
│  │  模型: GLM-5                                                     │   │
│  │  Agent工具: Claude Code ✓   Kimi CLI ✓                          │   │
│  │                                                                  │   │
│  │  用量: ████████░░░░  80% (400/500 日配额)                        │   │
│  │  状态: 🟢 正常    健康: 🟢 上次检查2分钟前                        │   │
│  │                                                                  │   │
│  │  [切换模型 ▼] [更换Plan ▼] [添加Agent工具 ▼] [管理API Key]       │   │
│  │  [编辑] [设为默认] [测试连接] [禁用] [删除]                      │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│                                                                          │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │  ⚪ Anthropic Claude Pro                                         │   │
│  │  Provider: Anthropic | Plan: 官方API                             │   │
│  │  模型: claude-sonnet-4-5 | Agent: Claude Code                    │   │
│  │  用量: ████░░░░░░░░  40% (120/300)                              │   │
│  │  [使用此套餐] [编辑] [删除]                                       │   │
│  └──────────────────────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────────────────────┘
```

### 5.3 套餐详情展开

```
点击套餐卡片展开详情：

┌──────────────────────────────────────────────────────────────────┐
│  Alaya Plus 详情                                    [收起 ▲]      │
│                                                                  │
│  ┌─ Provider 信息 ─────────────────────────────────────────────┐ │
│  │  名称: Alaya                                                │ │
│  │  官网: https://alaya.ai [打开 →]                            │ │
│  │  文档: https://docs.alaya.ai [打开 →]                       │ │
│  │  客服: support@alaya.ai                                     │ │
│  └─────────────────────────────────────────────────────────────┘ │
│                                                                  │
│  ┌─ Coding Plan 信息 ──────────────────────────────────────────┐ │
│  │  方案: Plus（进阶版）                                        │ │
│  │  等级: Pro                                                   │ │
│  │  描述: 进阶版，适合专业开发者                                 │ │
│  │  订阅: ¥29/月 [管理订阅 →]                                  │ │
│  └─────────────────────────────────────────────────────────────┘ │
│                                                                  │
│  ┌─ 模型配置 ──────────────────────────────────────────────────┐ │
│  │  当前: GLM-5                                                 │ │
│  │  可选: [MiniMax-2.5] [MiniMax-2.5-Pro] [●GLM-5]             │ │
│  │  能力: 代码生成 ✓  推理 ✓  中文优化 ✓  长上下文 ✓            │ │
│  └─────────────────────────────────────────────────────────────┘ │
│                                                                  │
│  ┌─ Agent 工具绑定 ────────────────────────────────────────────┐ │
│  │  Claude Code  ✓ 已配置  [配置方法 →]  [测试连接]            │ │
│  │    · 环境变量: ANTHROPIC_BASE_URL=http://127.0.0.1:8080      │ │
│  │    · 状态: 🟢 最近一次连接成功                               │ │
│  │  Kimi CLI     ✓ 已配置  [配置方法 →]  [测试连接]            │ │
│  │    · 配置文件: ~/.kimi/config.yaml                            │ │
│  │    · 状态: 🟢 最近一次连接成功                               │ │
│  │  OpenCode     ✗ 未配置  [一键配置 →]                          │ │
│  │  Kilo CLI     ✗ 未绑定（Plus版不支持）                        │ │
│  └─────────────────────────────────────────────────────────────┘ │
│                                                                  │
│  ┌─ API Key ──────────────────────────────────────────────────┐ │
│  │  状态: ✓ 已配置                                               │ │
│  │  Key: sk-alya-****-****-xxxx (末尾4位)                      │ │
│  │  [更新 Key] [查看获取方法 →]                                  │ │
│  └─────────────────────────────────────────────────────────────┘ │
│                                                                  │
│  ┌─ 配额与限制 ────────────────────────────────────────────────┐ │
│  │  日配额: 400 / 500 (80%)  [调整 →]                          │ │
│  │  月配额: 8200 / 10000 (82%)                                  │ │
│  │  RPM: 45 / 60                                                │ │
│  │  Fallback优先级: 1 [调整 →]                                  │ │
│  └─────────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────────┘
```

---

## 六、未使用过某Provider时的引导

### 6.1 首次选择Provider的引导流程

```
用户点击"添加套餐" → 选择Provider列表
    │
    ├─ 用户选择 Alaya（未使用过）
    │
    ▼
┌─────────────────────────────────────────────────────────────────┐
│  🤔 您还没有配置过 Alaya                                        │
│                                                                  │
│  Alaya 是一款中国团队开发的AI编程平台，提供多种模型和套餐。      │
│                                                                  │
│  ┌─ 快速开始 ─────────────────────────────────────────────────┐ │
│  │ 1. [🌐 去 Alaya 官网注册账号 →]                             │ │
│  │ 2. [💳 选择适合您的 Coding Plan →]                           │ │
│  │ 3. [🔑 获取 API Key →]                                      │ │
│  │ 4. [📖 查看详细配置指南 →]                                   │ │
│  └─────────────────────────────────────────────────────────────┘ │
│                                                                  │
│  ┌─ 各 Plan 对比 ──────────────────────────────────────────────┐ │
│  │  Lite      Plus       Max                                    │ │
│  │  ¥0       ¥29/月    ¥99/月                                  │ │
│  │  100日    500日      2000日                                  │ │
│  │  MiniMax  MiniMax    全模型                                  │ │
│  │  Claude   Claude+Kimi 全工具                                 │ │
│  │  [详情 →]  [详情 →]   [详情 →]                               │ │
│  └─────────────────────────────────────────────────────────────┘ │
│                                                                  │
│  我已准备好，[开始配置 →]                                        │
└──────────────────────────────────────────────────────────────────┘
```

### 6.2 Agent工具配置指南（一键配置）

```txt
用户点击"一键配置 Claude Code"
    │
    ▼
┌─────────────────────────────────────────────────────────────────┐
│  🔧 配置 Claude Code 连接 agent-gateway                           │
│                                                                  │
│  我们将自动帮您配置 Claude Code，使其通过 agent-gateway 访问AI。 │
│                                                                  │
│  ┌─ 方式一：自动配置（推荐） ──────────────────────────────────┐ │
│  │  检测到 Claude Code 已安装在 /usr/local/bin/claude            │ │
│  │  [🤖 一键自动配置]                                           │ │
│  │  · 将设置环境变量 ANTHROPIC_BASE_URL=http://127.0.0.1:8080    │ │
│  │  · 将设置环境变量 ANTHROPIC_API_KEY=dummy-key               │ │
│  │  · 添加到您的 shell 配置文件 (~/.zshrc)                      │ │
│  └─────────────────────────────────────────────────────────────┘ │
│                                                                  │
│  ┌─ 方式二：手动配置 ──────────────────────────────────────────┐ │
│  │  在您的终端中执行以下命令：                                   │ │
│  │                                                              │ │
│  │  export ANTHROPIC_BASE_URL=http://127.0.0.1:8080             │ │
│  │  export ANTHROPIC_API_KEY="dummy-key"                       │ │
│  │  claude                                                      │ │
│  │                                                              │ │
│  │  [📋 复制命令]  [📖 详细文档 →]                               │ │
│  └─────────────────────────────────────────────────────────────┘ │
│                                                                  │
│  ┌─ 方式三：配置文件 ──────────────────────────────────────────┐ │
│  │  编辑 Claude Code 配置文件：                                  │ │
│  │  · macOS: ~/Library/Application\ Support/Claude/settings.json │ │
│  │  · Linux: ~/.config/claude/settings.json                      │ │
│  │                                                              │ │
│  │  ```json                                                      │ │
│  │  {                                                            │ │
│  │    "anthropicBaseUrl": "http://127.0.0.1:8080"               │ │
│  │  }                                                            │ │
│  │  ```                                                          │ │
│  └─────────────────────────────────────────────────────────────┘ │
│                                                                  │
│  [✓ 已完成配置] [测试连接]                                       │
└──────────────────────────────────────────────────────────────────┘
```

### 6.3 Provider内置引导内容

```rust
// 每个Provider模板中包含引导内容
pub struct ProviderTemplate {
    // ... 基础字段 ...
    
    /// 未使用过时的引导信息
    pub onboarding: ProviderOnboarding,
}

pub struct ProviderOnboarding {
    /// 简介（1-2句话）
    pub description: String,
    /// 注册页面URL
    pub signup_url: String,
    /// 套餐对比页面URL
    pub plans_comparison_url: String,
    /// 获取API Key的URL
    pub get_key_url: String,
    /// 详细配置指南URL
    pub setup_guide_url: String,
    /// 各Agent工具的配置方法
    pub agent_setup_guides: Vec<AgentSetupGuide>,
}

pub struct AgentSetupGuide {
    pub agent_id: String,
    pub agent_name: String,
    /// 是否支持自动配置
    pub auto_config_supported: bool,
    /// 自动配置脚本
    pub auto_config_script: Option<String>,
    /// 手动配置步骤
    pub manual_steps: Vec<SetupStep>,
    /// 配置文件路径（各平台）
    pub config_file_paths: PlatformPaths,
}

pub struct SetupStep {
    pub step_number: u32,
    pub description: String,
    pub command: Option<String>,
    pub note: Option<String>,
}
```

---

## 七、架构设计

### 7.1 四层联动数据模型

```txt
┌─────────────────────────────────────────────────────────────────────────┐
│  Provider（提供商模板）                                                    │
│  · 内置配置（provider_id, name, logo, URLs）                              │
│  · 可远程更新                                                            │
│  · 包含 Coding Plans、Models、Agents 定义                                │
├─────────────────────────────────────────────────────────────────────────┤
│  Coding Plan（套餐方案）                                                   │
│  · 属于特定 Provider                                                      │
│  · 用户实例化后成为 UserPlan                                              │
│  · 定义 supported_models、supported_agents、quotas、priority             │
├─────────────────────────────────────────────────────────────────────────┤
│  UserPlan（用户套餐实例）← 这是实际运行的配置单元                          │
│  · provider_id + plan_id + 用户自定义配置                                │
│  · api_key（加密存储）                                                    │
│  · selected_model（从Plan支持列表中选定）                                  │
│  · bound_agents（绑定的Agent工具列表，可多选）                             │
│  · custom_quota（覆盖Plan默认的配额）                                      │
│  · custom_priority（Fallback优先级）                                       │
├─────────────────────────────────────────────────────────────────────────┤
│  Agent Binding（Agent工具绑定）                                            │
│  · agent_id（claude-code / kimi-cli / opencode / kilo-cli）              │
│  · 配置状态（已配置/未配置/配置中）                                        │
│  · 连接状态（在线/离线/错误）                                              │
│  · 最后连接时间                                                            │
└─────────────────────────────────────────────────────────────────────────┘
```

### 7.2 分层架构（完整版）

```txt
┌─────────────────────────────────────────────────────────────────────────┐
│  交互层                                                                  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────────────┐  │
│  │ CLI (Clap)    │  │ GUI (Tauri)   │  │ REST API (Axum)              │  │
│  │ agw-cli      │  │ agw-gui      │  │ agw-api                      │  │
│  │              │  │              │  │                              │  │
│  │ · Commands   │  │ · Vue3 UI    │  │ · /api/v1/* endpoints       │  │
│  │ · Shell comp │  │ · IPC bridge │  │ · CORS                     │  │
│  │ · Plugin cmd │  │ · Tray icon  │  │ · Auth                     │  │
│  └──────┬───────┘  └──────┬───────┘  └──────────────┬───────────────┘  │
└─────────┼─────────────────┼─────────────────────────┼──────────────────┘
          │                 │                         │
          └─────────────────┼─────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  业务层 (Business Layer)                                                 │
│                                                                          │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────────────┐  │
│  │ PlanManager   │  │FallbackEngine│  │ QuotaTracker                 │  │
│  │              │  │              │  │                              │  │
│  │ · UserPlan   │  │ · Trigger    │  │ · Daily/Monthly counter     │  │
│  │   CRUD       │  │ · Priority   │  │ · RPM limit                 │  │
│  │ · Provider   │  │ · Cross-fmt  │  │ · Limit check               │  │
│  │   template   │  │ · Max retry  │  │ · Alert at 80%              │  │
│  │ · Model sel. │  │ · Health prb │  │ · Reset cycle               │  │
│  │ · Agent bind │  │              │  │                              │  │
│  │ · Auto-conf. │  │              │  │                              │  │
│  │ · API Key    │  │              │  │                              │  │
│  │   helper     │  │              │  │                              │  │
│  └──────┬───────┘  └──────┬───────┘  └──────────────┬───────────────┘  │
│         │                 │                         │                   │
│  ┌──────┴─────────────────┴─────────────────────────┴────────────────┐  │
│  │ ProviderEngine                                                        │  │
│  │ · Template registry (内置+远程)                                       │  │
│  │ · Remote update                                                      │  │
│  │ · Version merge                                                      │  │
│  └────────────────────────────────────────────────────────────────────┘  │
│         │                                                                │
│  ┌──────┴────────────────────────────────────────────────────────────┐  │
│  │ PluginEngine (WASM VM)                                               │  │
│  │ · Provider plugins (扩展新Provider)                                  │  │
│  │ · Transform plugins                                                   │  │
│  │ · Install/Enable/Disable/Update                                     │  │
│  └────────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  核心层 (Core Layer)                                                     │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────────────┐  │
│  │ HttpGateway  │  │ProtocolConv. │  │ GlobalState                  │  │
│  │ (Axum)       │  │              │  │                              │  │
│  │ · Listener   │  │ · A→O req    │  │ · active_user_plan_id       │  │
│  │ · Routing    │  │ · O→A resp   │  │ · fallback_config           │  │
│  │ · Handlers   │  │ · SSE conv.  │  │ · quota_cache               │  │
│  │ · SSE pass   │  │              │  │ · plugin_registry           │  │
│  └──────┬───────┘  └──────┬───────┘  │ · encryption_key            │  │
│         │                 │            └──────────────┬───────────────┘  │
│         │                 │                           │                   │
│         └─────────────────┼───────────────────────────┘                   │
│                           │                                               │
│  ┌────────────────────────┴───────────────────────────────────────────┐  │
│  │ Forwarder (Reqwest)                                                  │  │
│  │ · Connection pool · Timeout · Retry · Headers rewrite             │  │
│  └────────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  存储层 (Storage Layer)                                                  │
│  ┌──────────────────────────┐  ┌──────────────────────────────────────┐ │
│  │ ConfigStore (YAML)       │  │ SqliteStore (rusqlite)               │ │
│  │                          │  │                                      │ │
│  │ · gateway.yaml           │  │ · quota.db                           │ │
│  │ · user_plans.yaml        │  │ · fallback_logs                      │ │
│  │   (用户套餐实例，含      │  │ · request_logs                       │ │
│  │    api_key, model,        │  │ · health_checks                      │ │
│  │    agents, quotas)         │  │                                      │ │
│  │ · fallback.yaml          │  │                                      │ │
│  │ · plugin/manifest.yaml   │  │                                      │ │
│  │ · providers_builtin.yaml │  │                                      │ │
│  │   (内置Provider模板)      │  │                                      │ │
│  │ · providers_custom.yaml  │  │                                      │ │
│  │   (用户自定义Provider)     │  │                                      │ │
│  │ · encryption.key         │  │                                      │ │
│  └──────────────────────────┘  └──────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────────┘
```

### 7.3 核心数据结构（更新版）

```rust
// ==================== Provider 模板（内置，可远程更新） ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderTemplate {
    pub provider_id: String,              // "alaya", "anthropic", "kimi"
    pub name: String,
    pub description: String,
    pub logo_url: Option<String>,
    pub homepage: String,
    pub docs_url: String,
    pub get_api_key_url: Option<String>,     // 获取API Key的直达链接
    pub setup_guide_url: Option<String>,     // 配置指南链接
    pub api_format: ApiFormat,
    pub base_url: Option<String>,            // 直连型Provider的Base URL
    pub base_url_template: Option<String>,   // 模板型（如 https://api.{provider}.com）
    pub requires_api_key: bool,              // 是否需要用户自己提供Key
    pub onboarding: ProviderOnboarding,      // 新用户引导信息
    pub coding_plans: Vec<CodingPlanTemplate>, // 该Provider提供的套餐方案
    pub models: Vec<ModelTemplate>,          // 该Provider支持的所有模型
    pub supported_agents: Vec<AgentToolRef>,   // 该Provider支持的所有Agent工具
    pub version: String,                     // Provider配置版本（用于远程更新）
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderOnboarding {
    pub description: String,
    pub signup_url: String,
    pub plans_comparison_url: Option<String>,
    pub get_key_url: Option<String>,
    pub setup_guide_url: Option<String>,
    pub faq_url: Option<String>,
    pub agent_setup_guides: Vec<AgentSetupGuide>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSetupGuide {
    pub agent_id: String,
    pub agent_name: String,
    pub auto_config_supported: bool,
    pub auto_config_script: Option<String>,    // 自动配置脚本（各平台）
    pub manual_steps: Vec<SetupStep>,
    pub config_file_paths: PlatformPaths,
    pub env_vars: Vec<EnvVarConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupStep {
    pub step_number: u32,
    pub description: String,
    pub command: Option<String>,
    pub copyable_text: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvVarConfig {
    pub name: String,
    pub value: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformPaths {
    pub macos: Option<String>,
    pub linux: Option<String>,
    pub windows: Option<String>,
}

// ==================== Coding Plan 模板 ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodingPlanTemplate {
    pub plan_id: String,                    // "alaya-lite"
    pub name: String,                     // "Lite"
    pub description: String,
    pub tier: PlanTier,                   // Free / Pro / Enterprise / Custom
    pub supported_model_ids: Vec<String>,   // 该Plan支持的模型ID列表
    pub supported_agent_ids: Vec<String>,   // 该Plan支持的Agent工具ID列表
    pub default_model_id: String,           // Plan推荐默认模型
    pub default_agent_id: String,           // Plan推荐默认Agent
    pub quota_daily: Option<u64>,           // Plan默认日配额
    pub quota_monthly: Option<u64>,         // Plan默认月配额
    pub rpm_limit: Option<u32>,             // Plan默认RPM限制
    pub price: Option<String>,              // 价格信息（展示用）
    pub features: Vec<String>,              // 特性标签
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlanTier {
    Free,
    Pro,
    Enterprise,
    Custom,
}

// ==================== 模型模板 ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelTemplate {
    pub model_id: String,                 // API调用时使用的模型ID
    pub name: String,                     // 展示名称
    pub description: Option<String>,
    pub context_length: Option<u64>,      // 上下文窗口大小
    pub capabilities: Vec<ModelCapability>, // 能力标签
    pub provider_id: String,              // 所属Provider
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelCapability {
    Code,             // 代码生成
    Reasoning,        // 推理
    LongContext,      // 长上下文
    ChineseOptimized, // 中文优化
    Math,             // 数学
    Multimodal,       // 多模态
}

// ==================== Agent 工具定义 ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTool {
    pub agent_id: String,                 // "claude-code"
    pub name: String,                     // "Claude Code"
    pub description: String,
    pub logo_url: Option<String>,
    pub homepage: String,
    pub install_url: String,              // 安装该工具的链接
    pub supported_formats: Vec<ApiFormat>, // 该工具支持的API格式
    pub config_methods: Vec<AgentConfigMethod>, // 配置方式
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentConfigMethod {
    EnvVar { name: String, value_template: String },
    ConfigFile { path_template: String, content_template: String },
    CliFlag { flag: String },
}

// ==================== 用户套餐实例（运行时配置单元） ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPlan {
    pub id: String,                       // 实例唯一ID（如 "my-alaya-plus-01"）
    pub provider_id: String,              // 关联的Provider
    pub plan_id: String,                  // 关联的Coding Plan
    pub name: String,                     // 用户自定义名称
    pub api_key: String,                  // 加密存储
    pub selected_model_id: String,        // 用户选定的模型
    pub bound_agents: Vec<AgentBinding>,  // 绑定的Agent工具
    pub enabled: bool,
    pub priority: u32,                    // Fallback优先级
    pub custom_quota_daily: Option<u64>,   // 覆盖Plan默认
    pub custom_quota_monthly: Option<u64>,
    pub custom_rpm_limit: Option<u32>,
    pub alert_threshold: Option<f32>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_health_check: Option<DateTime<Utc>>,
    pub health_status: HealthStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentBinding {
    pub agent_id: String,
    pub configured: bool,                 // 是否已完成配置
    pub config_status: AgentConfigStatus,
    pub last_connected: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentConfigStatus {
    NotConfigured,    // 未配置
    AutoConfigured,   // 已自动配置
    ManuallyConfigured, // 已手动配置
    ConfigError,      // 配置出错
    NeedsUpdate,      // 配置需要更新
}

// ==================== 运行时请求路由上下文 ====================

#[derive(Debug, Clone)]
pub struct RequestContext {
    pub user_plan: UserPlan,                // 当前使用的用户套餐实例
    pub agent_tool: Option<String>,        // 发起请求的Agent工具ID
    pub endpoint_format: ApiFormat,        // 请求的端点格式
    pub needs_conversion: bool,            // 是否需要协议转换
    pub target_format: ApiFormat,        // 目标Provider需要的格式
}
```

### 7.4 配置文件设计（更新版）

**user_plans.yaml** — 用户套餐实例（替代原plans.yaml）：

```yaml
version: "2.0"
default_user_plan_id: "my-alaya-plus"

user_plans:
  - id: "my-alaya-plus"
    name: "我的 Alaya Plus"
    provider_id: "alaya"
    plan_id: "alaya-plus"
    api_key: "ENC:base64encrypted..."
    selected_model_id: "glm-5"
    bound_agents:
      - agent_id: "claude-code"
        configured: true
        config_status: "auto_configured"
        last_connected: "2025-01-15T10:30:00Z"
      - agent_id: "kimi-cli"
        configured: true
        config_status: "manually_configured"
        last_connected: "2025-01-15T09:15:00Z"
    enabled: true
    priority: 1
    custom_quota_daily: null          # 使用Plan默认值
    custom_quota_monthly: null
    custom_rpm_limit: null
    alert_threshold: 0.8
    notes: "工作主力账号"
    health_status: "healthy"
    last_health_check: "2025-01-15T10:30:00Z"

  - id: "my-anthropic-pro"
    name: "Anthropic Claude"
    provider_id: "anthropic"
    plan_id: "anthropic-default"
    api_key: "ENC:base64encrypted..."
    selected_model_id: "claude-sonnet-4-5"
    bound_agents:
      - agent_id: "claude-code"
        configured: true
        config_status: "auto_configured"
        last_connected: "2025-01-15T10:25:00Z"
    enabled: true
    priority: 2
    health_status: "healthy"
```

**providers_builtin.yaml** — 内置Provider模板（可远程更新）：

```yaml
version: "2025-01-15"

providers:
  - provider_id: "alaya"
    name: "Alaya"
    description: "Alaya AI Coding Platform"
    logo_url: "https://cdn.agent-gateway.dev/providers/alaya.svg"
    homepage: "https://alaya.ai"
    docs_url: "https://docs.alaya.ai"
    get_api_key_url: "https://console.alaya.ai/settings/api-keys"
    setup_guide_url: "https://docs.alaya.ai/getting-started"
    api_format: "anthropic"
    base_url_template: "https://api.alaya.com/coding/{plan_id}"
    requires_api_key: true
    onboarding:
      description: "Alaya 是中国团队开发的AI编程平台，提供多种模型和套餐方案。"
      signup_url: "https://alaya.ai/signup"
      plans_comparison_url: "https://alaya.ai/pricing"
      get_key_url: "https://console.alaya.ai/settings/api-keys"
      setup_guide_url: "https://docs.alaya.ai/getting-started"
      agent_setup_guides:
        - agent_id: "claude-code"
          agent_name: "Claude Code"
          auto_config_supported: true
          auto_config_script: |
            # macOS/Linux
            echo 'export ANTHROPIC_BASE_URL=http://127.0.0.1:8080' >> ~/.zshrc
            echo 'export ANTHROPIC_API_KEY=dummy' >> ~/.zshrc
          manual_steps:
            - step_number: 1
              description: "设置环境变量"
              command: "export ANTHROPIC_BASE_URL=http://127.0.0.1:8080"
            - step_number: 2
              description: "启动 Claude Code"
              command: "claude"
          env_vars:
            - name: "ANTHROPIC_BASE_URL"
              value: "http://127.0.0.1:8080"
              description: "Claude Code 网关地址"
            - name: "ANTHROPIC_API_KEY"
              value: "dummy"
              description: "任意值，网关会替换为实际Key"

    coding_plans:
      - plan_id: "alaya-lite"
        name: "Lite"
        description: "轻量版，适合个人日常开发"
        tier: "free"
        supported_model_ids: ["minimax-2.5", "minimax-2.5-pro"]
        supported_agent_ids: ["claude-code"]
        default_model_id: "minimax-2.5"
        default_agent_id: "claude-code"
        quota_daily: 100
        quota_monthly: 2000
        rpm_limit: 20
        price: "免费"
        features: ["基础代码生成", "日常开发辅助"]

      - plan_id: "alaya-plus"
        name: "Plus"
        description: "进阶版，适合专业开发者"
        tier: "pro"
        supported_model_ids: ["minimax-2.5", "minimax-2.5-pro", "glm-5"]
        supported_agent_ids: ["claude-code", "kimi-cli"]
        default_model_id: "glm-5"
        default_agent_id: "claude-code"
        quota_daily: 500
        quota_monthly: 10000
        rpm_limit: 60
        price: "¥29/月"
        features: ["多模型切换", "专业开发", "中文优化"]

    models:
      - model_id: "minimax-2.5"
        name: "MiniMax-2.5"
        context_length: 256000
        capabilities: ["code", "reasoning"]
      - model_id: "glm-5"
        name: "GLM-5"
        context_length: 128000
        capabilities: ["code", "reasoning", "chinese-optimized"]

    supported_agents:
      - agent_id: "claude-code"
        name: "Claude Code"
      - agent_id: "kimi-cli"
        name: "Kimi CLI"
```

### 7.5 请求转发流程（更新版）

```txt
1. Agent工具发送请求 → 网关:8080/v1/messages 或 /v1/chat/completions

2. 网关识别请求来源Agent（通过Header或配置映射）

3. 查询该Agent绑定的 UserPlan：
   - Agent: claude-code → 绑定到 UserPlan: "my-alaya-plus"
   - 读取 UserPlan 配置：provider_id="alaya", plan_id="alaya-plus"

4. 查询 ProviderTemplate：
   - Provider: alaya → api_format="anthropic"
   - base_url = "https://api.alaya.com/coding/alaya-plus"
   
5. 检查 UserPlan.selected_model_id = "glm-5"
   - 验证 glm-5 在 CodingPlan.supported_model_ids 中 ✓

6. 检查配额（QuotaTracker）

7. 确定协议转换：
   - 请求端点格式（Anthropic）== Provider格式（Anthropic）→ 无需转换
   - 若不同 → ProtocolConverter 转换

8. 构建转发请求：
   - URL: https://api.alaya.com/coding/alaya-plus/v1/messages
   - Authorization: Bearer {UserPlan.api_key}
   - Body: 原始请求（含 model: "glm-5"）

9. 转发 → 接收响应 → 回传给 Agent

10. 更新配额、记录日志
```

---

## 八、Provider-Plan-Model-Agent 联动 GUI 设计

### 8.1 主界面：套餐卡片（核心交互单元）

```txt
┌──────────────────────────────────────────────────────────────────────────┐
│  我的 AI 编码套餐                                             [+ 添加套餐] │
│                                                                          │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │  🔵 Alaya Plus（当前默认）                                        │   │
│  │  ┌────────────────┬──────────────────────────────────────────┐   │   │
│  │  │  🤖 Alaya       │  方案: Plus（进阶版）| ¥29/月           │   │   │
│  │  │  中国团队        │  模型: GLM-5                            │   │   │
│  │  │                 │  用量: ████████░░░ 80% (400/500 日)     │   │   │
│  │  └────────────────┴──────────────────────────────────────────┘   │   │
│  │                                                                  │   │
│  │  已绑定的 Agent 工具：                                            │   │
│  │  ┌─────────────┐  ┌─────────────┐                              │   │
│  │  │ 🤖 Claude   │  │ 🔵 Kimi     │                              │   │
│  │  │    Code     │  │    CLI      │                              │   │
│  │  │ 🟢 正常     │  │ 🟢 正常     │  [+ 添加更多工具]            │   │
│  │  └─────────────┘  └─────────────┘                              │   │
│  │                                                                  │   │
│  │  [切换模型 ▼] [更换方案 ▼] [管理API Key] [编辑] [设为默认]       │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│                                                                          │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │  ⚪ Anthropic Claude Pro                                         │   │
│  │  🤖 Anthropic | 模型: claude-sonnet-4-5                         │   │
│  │  Agent: Claude Code 🟢                                          │   │
│  │  用量: ████░░░░░░░░ 40% (120/300)  [使用此套餐]                  │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│                                                                          │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │  🔴 Alaya Lite（配额将尽）                                       │   │
│  │  🤖 Alaya | 模型: MiniMax-2.5                                   │   │
│  │  Agent: Claude Code 🟡（配置需更新）                            │   │
│  │  用量: ████████████░ 95% ⚠️ (95/100)  [升级方案 →]               │   │
│  └──────────────────────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────────────────────┘
```

### 8.2 添加套餐向导

```txt
┌─────────────────────────────────────────────────────────────────────────┐
│  添加 AI 编码套餐                                             [取消] [←] │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  Step 1/5: 选择 AI 服务商                                               │
│  ─────────────────────                                                  │
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │  推荐                                                             │   │
│  │                                                                  │   │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐                      │   │
│  │  │  🇨🇳      │  │  🇺🇸      │  │  🇨🇳      │                      │   │
│  │  │  Alaya   │  │ Anthropic│  │   Kimi   │                      │   │
│  │  │          │  │          │  │          │                      │   │
│  │  │ 中国团队  │  │ Claude   │  │ 月之暗面  │                      │   │
│  │  │ 多模型    │  │ 官方     │  │ 长文本   │                      │   │
│  │  │ 多方案    │  │ 高质量   │  │ 强能力   │                      │   │
│  │  │          │  │          │  │          │                      │   │
│  │  │ [选择 →] │  │ [选择 →] │  │ [选择 →] │                      │   │
│  │  └──────────┘  └──────────┘  └──────────┘                      │   │
│  │                                                                  │   │
│  │  其他                                                             │   │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐       │   │
│  │  │ OpenCode  │  │  Kilo    │  │  OpenAI  │  │ [更多 ▼] │       │   │
│  │  └──────────┘  └──────────┘  └──────────┘  └──────────┘       │   │
│  │                                                                  │   │
│  │  [🌐 查看所有服务商对比 →]                                       │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
│                              [取消]          [下一步 →]                 │
└─────────────────────────────────────────────────────────────────────────┘
```

### 8.3 选择 Coding Plan

```txt
┌─────────────────────────────────────────────────────────────────────────┐
│  添加 AI 编码套餐                                             [取消] [←] │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  Step 2/5: 选择 Alaya 的 Coding Plan                                    │
│  ─────────────────────────────────────                                  │
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                                                                  │   │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐           │   │
│  │  │   Lite      │  │   Plus   ●  │  │   Max       │           │   │
│  │  │   免费      │  │   ¥29/月   │  │   ¥99/月    │           │   │
│  │  │            │  │   推荐      │  │   团队      │           │   │
│  │  │            │  │            │  │            │           │   │
│  │  │  100次/日  │  │  500次/日  │  │  2000次/日 │           │   │
│  │  │  MiniMax   │  │  MiniMax   │  │  全模型    │           │   │
│  │  │  Claude    │  │  GLM-5     │  │  全工具    │           │   │
│  │  │            │  │  Claude    │  │            │           │   │
│  │  │            │  │  Kimi      │  │            │           │   │
│  │  │  [选择]    │  │  [已选]    │  │  [选择]    │           │   │
│  │  └─────────────┘  └─────────────┘  └─────────────┘           │   │
│  │                                                                  │   │
│  │  ┌─ 方案对比 ──────────────────────────────────────────────────┐ │   │
│  │  │  特性          Lite      Plus      Max                      │ │   │
│  │  │  ─────────────────────────────────────────                  │ │   │
│  │  │  日调用次数    100       500       2000                   │ │   │
│  │  │  月调用次数    2K        10K       50K                    │ │   │
│  │  │  RPM限制       20         60        200                   │ │   │
│  │  │  支持模型      2          3         5                     │ │   │
│  │  │  支持工具      Claude    Claude+Kimi 全工具                │ │   │
│  │  │  中文优化      ✓          ✓          ✓                      │ │   │
│  │  │  优先支持      -          ✓          ✓                      │ │   │
│  │  └─────────────────────────────────────────────────────────────┘ │   │
│  │                                                                  │   │
│  │  [💳 去 Alaya 官网查看详情并订阅 →]                              │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
│  [← 上一步]                          [下一步 →]                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### 8.4 选择 Agent 工具

```txt
┌─────────────────────────────────────────────────────────────────────────┐
│  添加 AI 编码套餐                                             [取消] [←] │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  Step 3/5: 选择要绑定的 Agent 工具（可多选）                            │
│  ───────────────────────────────────────────                            │
│                                                                         │
│  Alaya Plus 支持以下 Agent 工具：                                        │
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                                                                  │   │
│  │  [✓] 🤖 Claude Code                                              │   │
│  │      最知名的AI编码工具，由Anthropic开发                          │   │
│  │      支持：Anthropic Messages API                                 │   │
│  │      状态：已安装于 /usr/local/bin/claude                         │   │
│  │                                                                  │   │
│  │  [✓] 🔵 Kimi CLI                                                 │   │
│  │      月之暗面开发的AI编码工具，中文支持优秀                        │   │
│  │      支持：Anthropic + OpenAI API                                 │   │
│  │      状态：未安装 [去安装 →]                                       │   │
│  │                                                                  │   │
│  │  [ ] 🟢 OpenCode                                                 │   │
│  │      开源AI编码工具                                               │   │
│  │      ⚠️ Alaya Plus 不支持此工具（Max方案支持）                    │   │
│  │      [查看支持此工具的方案 →]                                       │   │
│  │                                                                  │   │
│  │  [ ] 🟣 Kilo CLI                                                 │   │
│  │      多模型网关型编码工具                                          │   │
│  │      ⚠️ Alaya Plus 不支持此工具                                   │   │
│  │                                                                  │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
│  提示：可以同时绑定多个工具，它们将共享此套餐的配额。                    │
│                                                                         │
│  [← 上一步]                          [下一步 →]                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### 8.5 获取并填入 API Key

```txt
┌─────────────────────────────────────────────────────────────────────────┐
│  添加 AI 编码套餐                                             [取消] [←] │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  Step 4/5: 配置 API Key                                                 │
│  ─────────────────────                                                    │
│                                                                         │
│  正在配置：Alaya Plus → Claude Code + Kimi CLI                          │
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │  API Key                                                          │   │
│  │                                                                  │   │
│  │  ┌────────────────────────────────────────┐                      │   │
│  │  │ sk-alaya-xxxxxxxxxxxxxxxxxxxxxxxx       │  [👁 显示] [📋 粘贴]│   │
│  │  └────────────────────────────────────────┘                      │   │
│  │                                                                  │   │
│  │  [🌐 去 Alaya 控制台获取 API Key →]                               │   │
│  │  点击后将打开浏览器，登录后创建 API Key 并复制                     │   │
│  │                                                                  │   │
│  │  ── 或 ──                                                         │   │
│  │                                                                  │   │
│  │  [📖 查看配置指南]  [❓ 我没有 Alaya 账号，去注册 →]                │   │
│  │                                                                  │   │
│  │  ⚡ 检测到剪贴板中有 API Key（sk-alaya-...），是否使用？            │   │
│  │     [使用] [忽略]                                                 │   │
│  │                                                                  │   │
│  │  ┌─ 自动验证 ──────────────────────────────────────────────────┐ │   │
│  │  │  [🔍 测试连接]  验证API Key是否有效...                         │ │   │
│  │  │  ✓ 连接成功！可以访问 Alaya Plus API                          │ │   │
│  │  │  ✓ 模型 GLM-5 可用                                             │ │   │
│  │  │  ✓ 配额: 500/500 日调用剩余                                    │ │   │
│  │  └─────────────────────────────────────────────────────────────┘ │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
│  [← 上一步]                          [下一步 →]                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### 8.6 高级设置与完成

```txt
┌─────────────────────────────────────────────────────────────────────────┐
│  添加 AI 编码套餐                                             [取消] [←] │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  Step 5/5: 高级设置（可选）                                               │
│  ─────────────────────────                                                │
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │  基础设置                                                         │   │
│  │                                                                  │   │
│  │  套餐名称: [ 我的 Alaya Plus                    ]                 │   │
│  │                                                                  │   │
│  │  选择模型: [ GLM-5 ▼] (推荐) 该方案支持: MiniMax-2.5, GLM-5    │   │
│  │                                                                  │   │
│  │  Fallback优先级: [ 1 ]（越小越优先，1为最高）                     │   │
│  │                                                                  │   │
│  │  ─────────────────────────────────────────────────────────────── │   │
│  │  配额设置（留空使用方案默认值：日500/月10000）                     │   │
│  │                                                                  │   │
│  │  日配额:  [      ] / 500（方案默认）                             │   │
│  │  月配额:  [      ] / 10000                                       │   │
│  │  RPM限制: [      ] / 60                                          │   │
│  │                                                                  │   │
│  │  ─────────────────────────────────────────────────────────────── │   │
│  │  Agent工具自动配置                                                 │   │
│  │                                                                  │   │
│  │  [✓] 自动配置 Claude Code（将写入环境变量到 ~/.zshrc）            │   │
│  │  [✓] 自动配置 Kimi CLI（将更新 ~/.kimi/config.yaml）            │   │
│  │                                                                  │   │
│  │  [📋 预览自动配置命令]                                            │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
│  [← 上一步]                          [✓ 完成配置]                     │
└─────────────────────────────────────────────────────────────────────────┘
```

### 8.7 添加完成后的自动配置

```txt
配置完成！

┌─────────────────────────────────────────────────────────────────────────┐
│  🎉 Alaya Plus 配置成功！                                               │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  您的套餐已就绪：                                                        │
│  · Provider: Alaya                                                     │
│  · 方案: Plus                                                          │
│  · 模型: GLM-5                                                         │
│  · Agent工具: Claude Code, Kimi CLI                                    │
│                                                                         │
│  ┌─ 自动配置结果 ──────────────────────────────────────────────────┐   │
│  │                                                                  │   │
│  │  🤖 Claude Code                                                  │   │
│  │     ✓ 已自动配置                                                 │   │
│  │     · 添加了 ANTHROPIC_BASE_URL 到 ~/.zshrc                      │   │
│  │     · 添加了 ANTHROPIC_API_KEY 到 ~/.zshrc                       │   │
│  │     · 请在终端执行: source ~/.zshrc                              │   │
│  │     [📋 复制命令] [测试连接]                                      │   │
│  │                                                                  │   │
│  │  🔵 Kimi CLI                                                     │   │
│  │     ✓ 已自动配置                                                 │   │
│  │     · 更新了 ~/.kimi/config.yaml                                  │   │
│  │     [📋 查看配置] [测试连接]                                      │   │
│  │                                                                  │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
│  [🚀 立即使用 Claude Code]  [🚀 立即使用 Kimi CLI]  [完成]              │
│                                                                         │
│  提示：在终端中运行 `claude` 或 `kimi` 即可开始使用。                   │
└─────────────────────────────────────────────────────────────────────────┘
```

### 8.8 未使用过Provider时的引导

```txt
用户点击 Alaya（未使用过）
    │
    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  🤔 您还没有配置过 Alaya                                                │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  Alaya 是中国团队开发的AI编程平台，提供多种模型和套餐方案。              │
│                                                                         │
│  ┌─ 快速开始（3步）───────────────────────────────────────────────┐   │
│  │                                                                  │   │
│  │  Step 1: [🌐 去 Alaya 注册账号 →]                                │   │
│  │           访问 alaya.ai，使用邮箱或手机号注册                      │   │
│  │                                                                  │   │
│  │  Step 2: [💳 选择 Coding Plan 并订阅 →]                          │   │
│  │           Lite（免费）、Plus（¥29/月）、Max（¥99/月）              │   │
│  │                                                                  │   │
│  │  Step 3: [🔑 获取 API Key →]                                      │   │
│  │           登录控制台，创建 API Key 并复制                          │   │
│  │                                                                  │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
│  ┌─ 各方案对比 ────────────────────────────────────────────────────┐   │
│  │  方案    价格      日调用    模型          工具                    │   │
│  │  ─────────────────────────────────────────────────────            │   │
│  │  Lite    免费      100       MiniMax        Claude                │   │
│  │  Plus    ¥29/月   500       MiniMax+GLM-5   Claude+Kimi         │   │
│  │  Max     ¥99/月   2000      全模型         全工具                │   │
│  │                                                                  │   │
│  │  [查看详细对比 →]                                                 │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
│  ┌─ 需要帮助？─────────────────────────────────────────────────────┐   │
│  │  [📖 阅读完整文档]  [🎬 观看视频教程]  [💬 加入社区群]           │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
│  [我已准备好，开始配置 →]                                               │
│  [稍后再说]                                                             │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 九、CLI设计（更新版）

### 9.1 新增命令

```bash
# Provider 管理
agw provider list                    # 列出所有内置Provider
agw provider info <provider_id>      # 查看Provider详情
agw provider update                  # 更新Provider配置（从远程）

# 向导式添加套餐（交互式）
agw plan add --wizard                # 启动交互式向导
# 或逐步指定
agw plan add \
  --provider alaya \
  --plan alaya-plus \
  --model glm-5 \
  --agents claude-code,kimi-cli \
  --api-key "sk-alya-xxx"

# Agent 工具管理
agw agent list                       # 列出支持的Agent工具
agw agent bind <plan_id> <agent_id>  # 绑定Agent工具到套餐
agw agent unbind <plan_id> <agent_id> # 解绑
agw agent config <agent_id>          # 查看Agent配置方法
agw agent auto-config <plan_id> <agent_id>  # 自动配置Agent

# API Key 助手
agw key open-page <provider_id>      # 打开获取API Key页面
agw key test <plan_id>               # 测试API Key是否有效
```

### 9.2 交互式向导（CLI）

```bash
$ agw plan add --wizard

🤖 添加新的 AI 编码套餐

Step 1: 选择 AI 服务商
─────────────────────
[1] 🇨🇳 Alaya (中国团队，多模型，多方案)
[2] 🇺🇸 Anthropic (Claude 官方)
[3] 🇨🇳 Kimi (月之暗面，长文本)
[4] 🌍 OpenCode
[5] 🌍 Kilo
[6] [查看更多信息...]

请选择 [1-6]: 1

Step 2: 选择 Alaya 的 Coding Plan
─────────────────────────────────
[1] Lite  - 免费    (100次/日, MiniMax, Claude Code)
[2] Plus  - ¥29/月  (500次/日, MiniMax+GLM-5, Claude+Kimi) ⭐ 推荐
[3] Max   - ¥99/月  (2000次/日, 全模型, 全工具)

请选择 [1-3]: 2

Step 3: 选择 Agent 工具（可多选，逗号分隔）
─────────────────────────────────────────
Alaya Plus 支持:
[1] Claude Code
[2] Kimi CLI

请选择 [1-2]: 1,2

Step 4: 配置 API Key
────────────────────
请访问 https://console.alaya.ai/settings/api-keys 获取 API Key
[已复制到剪贴板，去粘贴 → 按 Enter 继续]

API Key: sk-alaya-xxxxxxxxxxxxxxxx

🔍 正在测试连接...
✓ 连接成功！
✓ 模型 GLM-5 可用
✓ 日配额: 500/500 剩余

Step 5: 高级设置（可选）
───────────────────────
套餐名称 \\[Alaya Plus]: 我的工作账号
Fallback优先级 [1]: 
日配额 [500]: 

✓ 套餐 "我的工作账号" 配置完成！

已自动配置:
  ✓ Claude Code (环境变量已写入 ~/.zshrc)
  ✓ Kimi CLI (配置文件已更新)

请执行: source ~/.zshrc
然后运行: claude 或 kimi 开始使用！
```

---

## 十、技术栈

| 模块        | 选型                         | 说明                 |
|-------------|------------------------------|----------------------|
| 核心语言    | Rust 2021 Edition            | 内存安全、高性能      |
| 异步运行时  | Tokio                        | Rust异步标准         |
| HTTP服务端  | Axum 0.7                     | 基于Hyper的Web框架   |
| HTTP客户端  | Reqwest 0.12                 | 连接池、SSE流式       |
| 协议转换    | 自研转换层                   | Anthropic↔OpenAI     |
| CLI框架     | Clap 4.5                     | 子命令/参数解析      |
| GUI框架     | Tauri 2.0                    | 轻量跨平台桌面       |
| GUI前端     | Vue 3 + Element Plus         | 组件丰富             |
| 配置存储    | Serde + serde_yaml           | YAML格式             |
| 数据存储    | SQLite + rusqlite            | 本地嵌入式           |
| 日志        | tracing + tracing-subscriber | 结构化日志           |
| 加密        | aes-gcm                      | AES-256-GCM          |
| 全局状态    | Arc + RwLock + DashMap       | 线程安全             |
| 插件运行时  | wasmtime + WASI              | WASM沙箱             |
| Node.js绑定 | NAPI-RS                      | Rust ↔ Node.js       |
| 浏览器唤起  | open crate                   | 跨平台唤起系统浏览器 |
| 剪贴板      | Tauri clipboard API          | 检测剪贴板内容       |
| 打包        | Tauri CLI + Cargo            | 三平台打包           |

---

## 十一、完整项目结构

```txt
agent-gateway/
├── Cargo.toml
├── README.md
│
├── crates/
│   ├── agw-core/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── model.rs                    # 核心数据结构
│   │       ├── model_types.rs              # 枚举定义
│   │       ├── business/
│   │       │   ├── mod.rs
│   │       │   ├── plan.rs                 # 套餐管理（UserPlan CRUD）
│   │       │   ├── provider_engine.rs      # Provider模板管理+远程更新
│   │       │   ├── fallback.rs             # Fallback引擎
│   │       │   └── quota.rs                # 配额追踪
│   │       ├── core/
│   │       │   ├── mod.rs
│   │       │   ├── gateway.rs              # Axum网关
│   │       │   ├── handler_anthropic.rs
│   │       │   ├── handler_openai.rs
│   │       │   ├── forwarder.rs
│   │       │   ├── state.rs
│   │       │   └── converter/
│   │       ├── storage/
│   │       │   ├── mod.rs
│   │       │   ├── config.rs               # YAML配置读写
│   │       │   └── sqlite.rs
│   │       ├── security/
│   │       │   ├── mod.rs
│   │       │   ├── encryption.rs
│   │       │   └── api_key_helper.rs       # API Key获取助手
│   │       └── plugin/
│   │           ├── mod.rs
│   │           ├── engine.rs
│   │           ├── registry.rs
│   │           ├── lifecycle.rs
│   │           ├── installer.rs
│   │           └── manifest.rs
│   │
│   ├── agw-cli/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       └── commands/
│   │           ├── mod.rs
│   │           ├── serve.rs
│   │           ├── plan.rs
│   │           ├── provider.rs             # Provider管理命令
│   │           ├── agent.rs                # Agent工具管理命令
│   │           ├── fallback.rs
│   │           ├── quota.rs
│   │           ├── plugin.rs
│   │           ├── config.rs
│   │           ├── log.rs
│   │           └── completion.rs
│   │
│   ├── agw-gui/
│   │   ├── Cargo.toml
│   │   ├── tauri.conf.json
│   │   └── src/
│   │       ├── main.rs
│   │       └── clipboard.rs                # 剪贴板监听
│   │
│   └── agw-api/
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs
│           ├── router.rs
│           ├── handlers/
│           └── middleware/
│
├── web/
│   ├── package.json
│   ├── vite.config.ts
│   └── src/
│       ├── main.ts
│       ├── App.vue
│       ├── types.ts
│       ├── api.ts
│       ├── views/
│       │   ├── Dashboard.vue
│       │   ├── Plans.vue                   # 套餐列表（卡片式）
│       │   ├── PlanWizard.vue              # 添加套餐向导
│       │   ├── Fallback.vue
│       │   ├── Quota.vue
│       │   ├── Logs.vue
│       │   ├── Plugins.vue
│       │   └── Settings.vue
│       ├── components/
│       │   ├── PlanCard.vue                # 套餐卡片（核心组件）
│       │   ├── PlanForm.vue                # 套餐表单（向导步骤）
│       │   ├── ProviderGrid.vue            # Provider选择网格
│       │   ├── PlanSelector.vue            # Plan选择卡片组
│       │   ├── AgentSelector.vue           # Agent工具选择
│       │   ├── ApiKeyInput.vue             # API Key输入（含直达链接）
│       │   ├── ModelSelector.vue           # 模型选择下拉
│       │   ├── OnboardingGuide.vue         # 新用户引导
│       │   ├── AutoConfigResult.vue        # 自动配置结果展示
│       │   ├── QuotaProgress.vue
│       │   ├── LogTable.vue
│       │   ├── StatusBadge.vue
│       │   └── PluginCard.vue
│       └── composables/
│           ├── useGateway.ts
│           ├── usePlans.ts
│           ├── useLogs.ts
│           ├── usePlugins.ts
│           ├── useClipboardMonitor.ts      # 剪贴板监听
│           └── useProviders.ts           # Provider管理
│
├── packages/
│   └── @node/
│   └── core/
│   └── cli/
│   └── node-win32-x64/
│
└── scripts/
    ├── build.sh
    └── build-npm.sh
```

---

## 十二、关键模块实现要点

### 12.1 ProviderEngine（模板管理+远程更新）

```rust
pub struct ProviderEngine {
    /// 内置Provider模板（随应用发布）
    builtin: Arc<RwLock<HashMap<String, ProviderTemplate>>>,
    /// 用户自定义Provider（通过插件添加）
    custom: Arc<RwLock<HashMap<String, ProviderTemplate>>>,
    /// 远程registry客户端
    registry: ProviderRegistryClient,
    /// 本地版本
    local_version: String,
}

impl ProviderEngine {
    /// 获取所有可用Provider（内置+自定义）
    pub async fn list_providers(&self) -> Vec<ProviderTemplate> {
        let mut all = self.builtin.read().await.values().cloned().collect::<Vec<_>>();
        let custom = self.custom.read().await.values().cloned().collect::<Vec<_>>();
        all.extend(custom);
        all
    }
    
    /// 获取指定Provider
    pub async fn get_provider(&self, provider_id: &str) -> Option<ProviderTemplate> {
        if let Some(p) = self.builtin.read().await.get(provider_id) {
            return Some(p.clone());
        }
        self.custom.read().await.get(provider_id).cloned()
    }
    
    /// 获取Provider的Coding Plan
    pub async fn get_plan_template(
        &self,
        provider_id: &str,
        plan_id: &str,
    ) -> Option<CodingPlanTemplate> {
        let provider = self.get_provider(provider_id).await?;
        provider.coding_plans.into_iter().find(|p| p.plan_id == plan_id)
    }
    
    /// 检查并应用远程更新
    pub async fn check_and_update(&self) -> Result<Option<UpdateReport>> {
        let remote_index = self.registry.fetch_index().await?;
        
        if remote_index.version == self.local_version {
            return Ok(None);
        }
        
        // 下载更新
        let updates = self.registry.fetch_updates(&self.local_version).await?;
        
        // 应用更新（合并，不覆盖用户配置）
        let mut builtin = self.builtin.write().await;
        for provider_update in updates.providers {
            builtin.insert(
                provider_update.provider_id.clone(),
                provider_update,
            );
        }
        
        self.local_version = remote_index.version.clone();
        
        Ok(Some(UpdateReport {
            new_version: remote_index.version,
            added_providers: updates.added,
            updated_providers: updates.updated,
        }))
    }
}
```

### 12.2 自动配置Agent工具

```rust
pub struct AgentAutoConfig;

impl AgentAutoConfig {
    /// 自动配置Agent工具连接到网关
    pub async fn configure(
        &self,
        agent_id: &str,
        gateway_addr: &str,
    ) -> Result<ConfigReport> {
        match agent_id {
            "claude-code" => self.configure_claude_code(gateway_addr).await,
            "kimi-cli" => self.configure_kimi_cli(gateway_addr).await,
            "opencode" => self.configure_opencode(gateway_addr).await,
            "kilo-cli" => self.configure_kilo_cli(gateway_addr).await,
            _ => bail!("Unknown agent: {}", agent_id),
        }
    }
    
    async fn configure_claude_code(&self, gateway_addr: &str) -> Result<ConfigReport> {
        let shell = self.detect_shell()?;
        let rc_file = match shell.as_str() {
            "zsh" => "~/.zshrc",
            "bash" => "~/.bashrc",
            _ => bail!("Unsupported shell: {}", shell),
        };
        
        let rc_path = shellexpand::tilde(rc_file);
        
        // 添加环境变量
        let env_vars = format!(
            "\n# Added by agent-gateway\n\
             export ANTHROPIC_BASE_URL=http://{}\n\
             export ANTHROPIC_API_KEY=\"agent-gateway-managed\"\n",
            gateway_addr
        );
        
        let mut file = OpenOptions::new()
            .append(true)
            .open(rc_path.as_ref())?;
        file.write_all(env_vars.as_bytes())?;
        
        Ok(ConfigReport {
            agent: "Claude Code".to_string(),
            method: "Env vars in rc file".to_string(),
            paths: vec![rc_path.to_string()],
            requires_reload: true,
            reload_command: Some(format!("source {}", rc_file)),
        })
    }
    
    async fn configure_kimi_cli(&self, gateway_addr: &str) -> Result<ConfigReport> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow!("Cannot find config dir"))?
            .join("kimi");
        fs::create_dir_all(&config_dir).await?;
        
        let config_path = config_dir.join("config.yaml");
        let config = format!(
            "api: anthropic-messages\n\
             baseUrl: http://{}/v1\n\
             apiKey: \"agent-gateway-managed\"\n",
            gateway_addr
        );
        
        fs::write(&config_path, config).await?;
        
        Ok(ConfigReport {
            agent: "Kimi CLI".to_string(),
            method: "Config file".to_string(),
            paths: vec![config_path.to_string_lossy().to_string()],
            requires_reload: false,
            reload_command: None,
        })
    }
}
```

### 12.3 API Key助手（唤起浏览器+剪贴板监听）

```rust
// crates/agw-core/src/security/api_key_helper.rs

pub struct ApiKeyHelper;

impl ApiKeyHelper {
    /// 打开Provider的API Key获取页面
    pub fn open_get_key_page(provider: &ProviderTemplate) -> Result<()> {
        let url = provider.get_api_key_url
            .as_ref()
            .ok_or_else(|| anyhow!("Provider does not have a key page URL"))?;
        open::that(url)?;
        Ok(())
    }
    
    /// 打开配置指南
    pub fn open_setup_guide(provider: &ProviderTemplate) -> Result<()> {
        if let Some(url) = &provider.setup_guide_url {
            open::that(url)?;
        }
        Ok(())
    }
    
    /// 验证API Key格式
    pub fn validate_key_format(provider: &ProviderTemplate, key: &str) -> Result<()> {
        match provider.provider_id.as_str() {
            "anthropic" | "alaya" => {
                if !key.starts_with("sk-") {
                    bail!("API Key should start with 'sk-'");
                }
            }
            _ => {}
        }
        if key.len() < 20 {
            bail!("API Key seems too short");
        }
        Ok(())
    }
}

// crates/agw-gui/src/clipboard.rs
#[tauri::command]
async fn check_clipboard_for_key(
    app: tauri::AppHandle,
    expected_prefix: Option<String>,
) -> Result<Option<String>, String> {
    let clipboard = app.clipboard();
    let content = clipboard.read_text().map_err(|e| e.to_string())?;
    
    let trimmed = content.trim();
    
    // 检测是否是API Key
    let is_key = matches_api_key_pattern(trimmed);
    
    if is_key {
        if let Some(prefix) = expected_prefix {
            if trimmed.starts_with(&prefix) {
                return Ok(Some(trimmed.to_string()));
            }
        } else {
            return Ok(Some(trimmed.to_string()));
        }
    }
    
    Ok(None)
}

fn matches_api_key_pattern(content: &str) -> bool {
    let prefixes = ["sk-", "sk-ant-", "sk-proj-", "AIza", "gsk_", "kilo_"];
    prefixes.iter().any(|p| content.starts_with(p)) && content.len() > 20
}
```

---

## 十三、部署形态矩阵

| 形态              | 说明                  | 编译                     | 安装                   |
|-------------------|-----------------------|--------------------------|------------------------|
| **Desktop**（默认） | GUI+CLI+网关+系统托盘 | `cargo build -p agw-gui` | .dmg/.exe/.deb         |
| CLI Only          | 命令行，无GUI          | `cargo build -p agw-cli` | `cargo install` / brew |
| API Server        | 纯网关+REST API       | `cargo build -p agw-api` | 二进制 / Docker        |
| Library           | Rust库                | `cargo add agw-core`     | crates.io              |
| npm包             | Node.js绑定           | `npm install`            | npm registry           |

---

## 十四、开发里程碑（更新）

| 阶段     | 内容          | 工期               | 关键交付                                          |
|----------|---------------|--------------------|---------------------------------------------------|
| M1       | 骨架+数据结构 | 4天                | Provider/Plan/Model/Agent四层模型定义完成         |
| M2       | Provider引擎  | 3天                | 内置Provider模板+远程更新机制                     |
| M3       | 存储+业务层   | 5天                | UserPlan CRUD+配额+Fallback                       |
| M4       | 协议转换      | 5天                | A↔O转换+SSE                                       |
| M5       | 网关+转发     | 4天                | 双端点+Agent路由                                  |
| M6       | 插件系统      | 5天                | WASM运行时+安装/启用                              |
| M7       | CLI           | 3天                | 含provider/agent/key命令                          |
| M8       | API服务器     | 3天                | REST API+Auth                                     |
| M9       | GUI后端       | 3天                | IPC+剪贴板+浏览器唤起                             |
| M10      | GUI前端       | 8天                | 向导式配置+Provider网格+Plan卡片+Agent选择+AK输入 |
| M11      | Node.js绑定   | 3天                | NAPI+预编译                                       |
| M12      | E2E测试       | 5天                | 向导流程+自动配置+Fallback                        |
| M13      | 打包+文档     | 3天                | 三平台+Docker+npm                                 |
| 缓冲     | 修复          | 5天                |                                                   |
| **合计** |               | **约59天 (~12周)** |                                                   |

**团队**：3人（2 Rust后端 + 1 Vue前端），最后2周共同集成。

---

## 十五、代码量估计

| 模块          | 预估行数     |
|---------------|--------------|
| 数据结构+类型 | 500          |
| Provider引擎  | 600          |
| 套餐管理      | 500          |
| Fallback      | 400          |
| 配额          | 450          |
| 网关+转发     | 550          |
| 协议转换      | 1400         |
| 插件系统      | 1300         |
| API Key助手   | 300          |
| Agent自动配置 | 400          |
| CLI           | 1000         |
| API服务器     | 800          |
| Tauri后端     | 400          |
| Vue前端       | 3500         |
| Node.js绑定   | 400          |
| **合计**      | **~13500行** |

我已仔细阅读了你的 `design.md` 文档。这是一份非常详实的设计方案，四层联动体系（Provider-Plan-Model-Agent）的设计思路很清晰。我来帮你补全和完善，重点补充以下内容：

1. **运行时请求路由的完整实现细节**
2. **Fallback 引擎的详细设计**
3. **配额追踪的完整实现**
4. **协议转换的边界情况处理**
5. **插件系统的 Provider 扩展规范**
6. **安全与加密的完整方案**
7. **API Server 的完整端点设计**

---

## 补全内容

### 十六、运行时请求路由详细设计

#### 16.1 Agent 识别与路由映射

```rust
// crates/agw-core/src/core/routing.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRouter {
    /// Agent标识 → UserPlan绑定关系
    agent_bindings: HashMap<String, AgentRouteConfig>,
    /// 默认UserPlan（未匹配到Agent时使用）
    default_user_plan_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRouteConfig {
    pub user_plan_id: String,
    pub agent_id: String,
    pub detection_method: AgentDetectionMethod,
    pub auto_configured: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentDetectionMethod {
    /// 通过请求Header中的 User-Agent
    UserAgent(String),
    /// 通过请求Header中的自定义 Header
    CustomHeader { name: String, value: String },
    /// 通过请求来源IP（本地不同端口）
    SourcePort { port: u16 },
    /// 通过入站端点路径区分
    EndpointPath { path_prefix: String },
}

impl AgentRouter {
    /// 从请求中识别Agent
    pub fn detect_agent(&self, req: &Request<Body>) -> Option<&AgentRouteConfig> {
        // 1. 检查自定义Header
        if let Some(agent_id) = req.headers()
            .get("X-Agent-Gateway-Agent-Id")
            .and_then(|v| v.to_str().ok()) 
        {
            return self.agent_bindings.get(agent_id);
        }
        
        // 2. 检查User-Agent
        if let Some(ua) = req.headers().get(hyper::header::USER_AGENT) {
            let ua_str = ua.to_str().unwrap_or("");
            for binding in self.agent_bindings.values() {
                if let AgentDetectionMethod::UserAgent(pattern) = &binding.detection_method {
                    if ua_str.contains(pattern.as_str()) {
                        return Some(binding);
                    }
                }
            }
        }
        
        // 3. 通过端点路径推断（如 /claude-code/*, /kimi-cli/*）
        let path = req.uri().path();
        for binding in self.agent_bindings.values() {
            if let AgentDetectionMethod::EndpointPath { path_prefix } = &binding.detection_method {
                if path.starts_with(path_prefix) {
                    return Some(binding);
                }
            }
        }
        
        None
    }
    
    /// 获取Agent绑定的UserPlan
    pub fn resolve_user_plan(&self, agent_id: Option<&str>) -> Result<UserPlan> {
        match agent_id {
            Some(id) => {
                let binding = self.agent_bindings.get(id)
                    .ok_or_else(|| anyhow!("No binding found for agent: {}", id))?;
                UserPlan::load(&binding.user_plan_id)
            }
            None => {
                // 使用默认套餐
                UserPlan::load(&self.default_user_plan_id)
            }
        }
    }
}
```

#### 16.2 完整的请求转发流程

```rust
// crates/agw-core/src/core/gateway.rs

pub struct AgentGateway {
    config: Arc<RwLock<GatewayConfig>>,
    router: Arc<RwLock<AgentRouter>>,
    forwarder: Forwarder,
    quota_tracker: Arc<QuotaTracker>,
    fallback_engine: Arc<FallbackEngine>,
    converter: ProtocolConverter,
    state: Arc<GlobalState>,
}

impl AgentGateway {
    /// 处理传入的API请求
    pub async fn handle_request(&self, req: Request<Body>) -> Result<Response<Body>> {
        let request_id = Uuid::new_v4();
        let start_time = Instant::now();
        
        // Step 1: 识别Agent
        let agent_binding = self.router.read().await.detect_agent(&req);
        let agent_id = agent_binding.map(|b| b.agent_id.clone());
        
        tracing::info!(
            request_id = %request_id,
            agent_id = ?agent_id,
            "Incoming request"
        );
        
        // Step 2: 解析请求格式（Anthropic / OpenAI / Custom）
        let request_format = self.detect_request_format(&req);
        
        // Step 3: 加载UserPlan（通过Agent绑定或默认）
        let user_plan = match agent_binding {
            Some(binding) => UserPlan::load(&binding.user_plan_id)?,
            None => UserPlan::load(&self.config.read().await.default_user_plan_id)?,
        };
        
        if !user_plan.enabled {
            return Err(anyhow!("UserPlan {} is disabled", user_plan.id));
        }
        
        // Step 4: 加载Provider模板
        let provider = ProviderEngine::global()
            .get_provider(&user_plan.provider_id)
            .await
            .ok_or_else(|| anyhow!("Provider not found: {}", user_plan.provider_id))?;
        
        // Step 5: 确定目标格式
        let target_format = provider.api_format;
        
        // Step 6: 检查配额
        if !self.quota_tracker.check_and_consume(&user_plan.id).await? {
            // 配额不足，尝试Fallback
            match self.fallback_engine.find_alternative(&user_plan.id).await? {
                Some(fallback_plan) => {
                    tracing::warn!(
                        request_id = %request_id,
                        original_plan = %user_plan.id,
                        fallback_plan = %fallback_plan.id,
                        "Quota exhausted, falling back"
                    );
                    return self.forward_with_plan(
                        &fallback_plan,
                        &provider,
                        req,
                        request_format,
                        target_format,
                    ).await;
                }
                None => {
                    return Ok(self.build_quota_exceeded_response(&user_plan));
                }
            }
        }
        
        // Step 7: 转发请求
        self.forward_with_plan(&user_plan, &provider, req, request_format, target_format).await
    }
    
    async fn forward_with_plan(
        &self,
        user_plan: &UserPlan,
        provider: &ProviderTemplate,
        req: Request<Body>,
        request_format: ApiFormat,
        target_format: ApiFormat,
    ) -> Result<Response<Body>> {
        // 构建目标URL
        let base_url = self.build_base_url(provider, user_plan);
        let target_url = format!("{}{}", base_url, req.uri().path());
        
        // 协议转换（如果需要）
        let (body, headers) = if request_format != target_format {
            self.converter.convert_request(
                req,
                request_format,
                target_format,
                &user_plan.selected_model_id,
            )?
        } else {
            self.passthrough_request(req, &user_plan.selected_model_id)?
        };
        
        // 注入API Key
        let headers = self.inject_auth(headers, &user_plan.api_key, target_format);
        
        // 转发
        let response = self.forwarder.send(
            &target_url,
            body,
            headers,
            provider.api_format,
        ).await?;
        
        // 转回响应格式
        let response = if request_format != target_format {
            self.converter.convert_response(response, target_format, request_format)?
        } else {
            response
        };
        
        Ok(response)
    }
    
    fn build_base_url(&self, provider: &ProviderTemplate, user_plan: &UserPlan) -> String {
        if let Some(template) = &provider.base_url_template {
            // 模板型：https://api.{provider}.com/coding/{plan_id}
            template
                .replace("{plan_id}", &user_plan.plan_id)
                .replace("{provider_id}", &provider.provider_id)
        } else if let Some(base_url) = &provider.base_url {
            base_url.clone()
        } else {
            panic!("Provider has no base_url or base_url_template");
        }
    }
    
    fn inject_auth(
        &self,
        mut headers: HeaderMap,
        api_key: &str,
        format: ApiFormat,
    ) -> HeaderMap {
        match format {
            ApiFormat::Anthropic => {
                headers.insert(
                    "x-api-key",
                    HeaderValue::from_str(api_key).unwrap(),
                );
            }
            ApiFormat::OpenAi => {
                headers.insert(
                    hyper::header::AUTHORIZATION,
                    HeaderValue::from_str(&format!("Bearer {}", api_key)).unwrap(),
                );
            }
        }
        headers
    }
}
```

---

### 十七、Fallback 引擎完整设计

#### 17.1 Fallback 策略与触发条件

```rust
// crates/agw-core/src/business/fallback.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackEngine {
    /// 所有可用的UserPlan（按优先级排序）
    plans: Vec<UserPlan>,
    /// Fallback配置
    config: FallbackConfig,
    /// 健康检查状态缓存
    health_cache: Arc<DashMap<String, HealthStatus>>,
    /// 近期Fallback日志
    recent_fallbacks: Arc<RwLock<VecDeque<FallbackEvent>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackConfig {
    /// 是否启用Fallback
    pub enabled: bool,
    /// 最大重试次数（一个请求链中）
    pub max_retries: u32,
    /// Fallback触发条件
    pub triggers: FallbackTriggers,
    /// 冷却时间（同一Plan被降级后，多少秒内不再尝试）
    pub cooldown_seconds: u64,
    /// 自动恢复检测间隔（秒）
    pub recovery_check_interval: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackTriggers {
    /// HTTP 429 (Rate Limit)
    pub on_rate_limited: bool,
    /// HTTP 5xx (Server Error)
    pub on_server_error: bool,
    /// 连接超时
    pub on_timeout: bool,
    /// 配额耗尽（预先检测，不等返回429）
    pub on_quota_exhausted: bool,
    /// 连续失败次数
    pub consecutive_failures: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackEvent {
    pub timestamp: DateTime<Utc>,
    pub from_plan_id: String,
    pub to_plan_id: String,
    pub reason: FallbackReason,
    pub request_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FallbackReason {
    QuotaExhausted,
    RateLimited,
    ServerError { status_code: u16 },
    Timeout,
    ConnectionFailed,
    ConsecutiveFailures { count: u32 },
    ManualDisable,
}

impl FallbackEngine {
    /// 查找可用的替代套餐
    pub async fn find_alternative(&self, failed_plan_id: &str) -> Result<Option<UserPlan>> {
        if !self.config.enabled {
            return Ok(None);
        }
        
        // 获取所有启用的套餐，按优先级排序
        let candidates: Vec<&UserPlan> = self.plans.iter()
            .filter(|p| p.enabled && p.id != failed_plan_id)
            .sorted_by_key(|p| p.priority)
            .collect();
        
        for candidate in candidates {
            // 检查冷却时间
            if self.is_in_cooldown(&candidate.id) {
                tracing::debug!(plan_id = %candidate.id, "Plan in cooldown, skipping");
                continue;
            }
            
            // 检查健康状态
            let health = self.health_cache.get(&candidate.id);
            if let Some(h) = health.as_deref() {
                if *h == HealthStatus::Unhealthy {
                    tracing::debug!(plan_id = %candidate.id, "Plan unhealthy, skipping");
                    continue;
                }
            }
            
            // 检查配额
            if let Ok(quota) = self.check_quota_available(&candidate.id).await {
                if quota.has_capacity() {
                    return Ok(Some(candidate.clone()));
                }
            }
        }
        
        Ok(None)
    }
    
    /// 记录Fallback事件
    pub async fn record_fallback(&self, event: FallbackEvent) {
        tracing::warn!(
            from = %event.from_plan_id,
            to = %event.to_plan_id,
            reason = ?event.reason,
            "Fallback triggered"
        );
        
        // 设置为冷却状态
        self.set_cooldown(&event.from_plan_id, self.config.cooldown_seconds);
        
        // 记录日志
        let mut recent = self.recent_fallbacks.write().await;
        recent.push_front(event);
        if recent.len() > 100 {
            recent.truncate(100);
        }
        
        // 持久化到SQLite
        self.persist_fallback_event(&event).await.ok();
    }
    
    /// 健康检查：主动探测
    pub async fn health_check(&self, plan: &UserPlan) -> HealthStatus {
        let provider = ProviderEngine::global()
            .get_provider(&plan.provider_id)
            .await;
        
        match provider {
            Some(p) => {
                // 发送一个轻量探测请求（如列出模型）
                let result = self.probe_endpoint(&p, plan).await;
                match result {
                    Ok(()) => HealthStatus::Healthy,
                    Err(_) => HealthStatus::Unhealthy,
                }
            }
            None => HealthStatus::Unhealthy,
        }
    }
    
    /// 定期健康检查和自动恢复
    pub async fn start_health_monitor(self: Arc<Self>) {
        let interval = self.config.recovery_check_interval;
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(interval)).await;
                
                for plan in &self.plans {
                    if plan.health_status == HealthStatus::Unhealthy {
                        let new_status = self.health_check(plan).await;
                        if new_status == HealthStatus::Healthy {
                            tracing::info!(
                                plan_id = %plan.id,
                                "Plan recovered, available for fallback"
                            );
                            self.clear_cooldown(&plan.id);
                        }
                        self.health_cache.insert(plan.id.clone(), new_status);
                    }
                }
            }
        });
    }
}
```

#### 17.2 Fallback 执行流程

```txt
请求到达 → 主Plan处理
      │
      ├─ 成功 → 返回响应
      │
      └─ 失败 ──┬── 配额耗尽 → 查找Fallback Plan
                │        │
                │        ├─ 找到 → 记录Fallback事件 → 转发到Fallback Plan
                │        │        │
                │        │        ├─ 成功 → 返回响应
                │        │        └─ 失败 → 继续查找下一个（最多max_retries次）
                │        │
                │        └─ 未找到 → 返回配额耗尽/服务不可用
                │
                ├── 429限流 → 同上查找Fallback
                │
                ├── 5xx错误 → 同上查找Fallback（可选）
                │
                └── 超时 → 重试N次后，同上查找Fallback
```

---

### 十八、配额追踪完整实现

```rust
// crates/agw-core/src/business/quota.rs

#[derive(Debug, Clone)]
pub struct QuotaTracker {
    /// 内存缓存（高性能读写）
    counters: Arc<DashMap<String, PlanQuotaCounter>>,
    /// 持久化存储（SQLite）
    storage: Arc<SqliteStore>,
    /// 配置
    config: QuotaConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanQuotaCounter {
    pub plan_id: String,
    /// 当日计数
    pub daily_count: AtomicU64,
    pub daily_reset_at: DateTime<Utc>,
    /// 当月计数
    pub monthly_count: AtomicU64,
    pub monthly_reset_at: DateTime<Utc>,
    /// RPM：滑动窗口
    pub rpm_window: Arc<Mutex<VecDeque<Instant>>>,
    /// 上次检查时间
    pub last_checked: AtomicI64,  // Unix timestamp
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaConfig {
    /// 配额检查模式
    pub mode: QuotaMode,
    /// 告警阈值（如0.8表示80%时告警）
    pub alert_threshold: f32,
    /// RPM滑动窗口大小（秒）
    pub rpm_window_seconds: u64,
    /// 持久化间隔（秒）
    pub persist_interval_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QuotaMode {
    /// 严格模式：超配额立即拒绝
    Strict,
    /// 宽松模式：超配额仅告警
    Lenient,
    /// 软限制：允许超出一定比例（如10%）
    SoftLimit { overage_percent: f32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaCheckResult {
    pub allowed: bool,
    pub reason: Option<QuotaExceedReason>,
    pub remaining_daily: Option<u64>,
    pub remaining_monthly: Option<u64>,
    pub current_rpm: Option<u32>,
    pub rpm_limit: Option<u32>,
    pub usage_percent: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QuotaExceedReason {
    DailyLimitReached { limit: u64, current: u64 },
    MonthlyLimitReached { limit: u64, current: u64 },
    RpmLimitReached { limit: u32, current: u32 },
}

impl QuotaTracker {
    /// 检查并扣减配额
    pub async fn check_and_consume(&self, plan_id: &str) -> Result<QuotaCheckResult> {
        // 获取UserPlan配置
        let user_plan = UserPlan::load(plan_id)?;
        let plan_template = ProviderEngine::global()
            .get_plan_template(&user_plan.provider_id, &user_plan.plan_id)
            .await
            .ok_or_else(|| anyhow!("Plan template not found"))?;
        
        // 获取限制（用户自定义优先，否则用Plan默认）
        let daily_limit = user_plan.custom_quota_daily
            .or(plan_template.quota_daily)
            .unwrap_or(u64::MAX);
        let monthly_limit = user_plan.custom_quota_monthly
            .or(plan_template.quota_monthly)
            .unwrap_or(u64::MAX);
        let rpm_limit = user_plan.custom_rpm_limit
            .or(plan_template.rpm_limit)
            .unwrap_or(u32::MAX);
        
        // 获取或初始化计数器
        let counter = self.get_or_init_counter(plan_id);
        
        // 检查日重置
        counter.check_and_reset_daily().await;
        
        // 检查月重置
        counter.check_and_reset_monthly().await;
        
        // 检查RPM
        let current_rpm = counter.get_current_rpm(self.config.rpm_window_seconds);
        
        // 检查各项限制
        let daily_current = counter.daily_count.load(Ordering::Relaxed);
        let monthly_current = counter.monthly_count.load(Ordering::Relaxed);
        
        let exceeded = if daily_current >= daily_limit {
            Some(QuotaExceedReason::DailyLimitReached {
                limit: daily_limit,
                current: daily_current,
            })
        } else if monthly_current >= monthly_limit {
            Some(QuotaExceedReason::MonthlyLimitReached {
                limit: monthly_limit,
                current: monthly_current,
            })
        } else if current_rpm >= rpm_limit {
            Some(QuotaExceedReason::RpmLimitReached {
                limit: rpm_limit,
                current: current_rpm,
            })
        } else {
            None
        };
        
        let usage_percent = (daily_current as f32 / daily_limit as f32).max(
            monthly_current as f32 / monthly_limit as f32
        );
        
        match (&self.config.mode, exceeded) {
            (QuotaMode::Strict, Some(reason)) => {
                Ok(QuotaCheckResult {
                    allowed: false,
                    reason: Some(reason),
                    remaining_daily: Some(daily_limit.saturating_sub(daily_current)),
                    remaining_monthly: Some(monthly_limit.saturating_sub(monthly_current)),
                    current_rpm: Some(current_rpm),
                    rpm_limit: Some(rpm_limit),
                    usage_percent,
                })
            }
            (QuotaMode::SoftLimit { overage_percent }, Some(reason)) => {
                let overage = (daily_limit as f32 * overage_percent) as u64;
                if daily_current < daily_limit + overage {
                    // 在软限制范围内，允许
                    self.consume_quota(plan_id, &counter).await;
                    Ok(QuotaCheckResult {
                        allowed: true,
                        reason: None,
                        remaining_daily: Some((daily_limit + overage).saturating_sub(daily_current)),
                        remaining_monthly: Some(monthly_limit.saturating_sub(monthly_current)),
                        current_rpm: Some(current_rpm),
                        rpm_limit: Some(rpm_limit),
                        usage_percent,
                    })
                } else {
                    Ok(QuotaCheckResult {
                        allowed: false,
                        reason: Some(reason),
                        remaining_daily: Some(0),
                        remaining_monthly: Some(monthly_limit.saturating_sub(monthly_current)),
                        current_rpm: Some(current_rpm),
                        rpm_limit: Some(rpm_limit),
                        usage_percent,
                    })
                }
            }
            _ => {
                // 允许通过
                self.consume_quota(plan_id, &counter).await;
                Ok(QuotaCheckResult {
                    allowed: true,
                    reason: None,
                    remaining_daily: Some(daily_limit.saturating_sub(daily_current + 1)),
                    remaining_monthly: Some(monthly_limit.saturating_sub(monthly_current + 1)),
                    current_rpm: Some(current_rpm + 1),
                    rpm_limit: Some(rpm_limit),
                    usage_percent,
                })
            }
        }
    }
}
```

---

### 十九、协议转换完整实现

#### 19.1 Anthropic ↔ OpenAI 双向转换

```rust
// crates/agw-core/src/core/converter/mod.rs

pub struct ProtocolConverter;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApiFormat {
    Anthropic,   // Messages API
    OpenAi,      // Chat Completions API
    Custom(String), // 其他格式
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionContext {
    pub model_id: String,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub system_prompt: Option<String>,
}

// ==================== Anthropic → OpenAI ====================

impl ProtocolConverter {
    pub fn anthropic_to_openai_request(
        anthropic_req: &AnthropicRequest,
        ctx: &ConversionContext,
    ) -> Result<OpenAiRequest> {
        let mut messages = Vec::new();
        
        // 系统提示
        if let Some(system) = &anthropic_req.system {
            messages.push(OpenAiMessage {
                role: "system".to_string(),
                content: OpenAiContent::Text(system.clone()),
                name: None,
                tool_calls: None,
                tool_call_id: None,
            });
        }
        
        // 转换消息
        for msg in &anthropic_req.messages {
            let openai_msg = Self::convert_anthropic_message_to_openai(msg)?;
            messages.push(openai_msg);
        }
        
        // 转换工具定义
        let tools = anthropic_req.tools.as_ref().map(|tools| {
            tools.iter().map(|tool| OpenAiTool {
                r#type: "function".to_string(),
                function: OpenAiFunctionDef {
                    name: tool.name.clone(),
                    description: tool.description.clone(),
                    parameters: tool.input_schema.clone(),
                },
            }).collect()
        });
        
        Ok(OpenAiRequest {
            model: ctx.model_id.clone(),
            messages,
            max_tokens: ctx.max_tokens,
            temperature: ctx.temperature,
            tools,
            tool_choice: anthropic_req.tool_choice.clone().map(|tc| {
                match tc {
                    AnthropicToolChoice::Auto => "auto".to_string(),
                    AnthropicToolChoice::Any => "required".to_string(),
                    AnthropicToolChoice::Tool { name } => {
                        serde_json::json!({"type": "function", "function": {"name": name}}).to_string()
                    }
                }
            }),
            stream: anthropic_req.stream,
            ..Default::default()
        })
    }
    
    fn convert_anthropic_message_to_openai(
        msg: &AnthropicMessage,
    ) -> Result<OpenAiMessage> {
        let role = match msg.role.as_str() {
            "user" => "user",
            "assistant" => "assistant",
            _ => bail!("Unknown role: {}", msg.role),
        };
        
        let content = match &msg.content {
            AnthropicContent::Text(text) => OpenAiContent::Text(text.clone()),
            AnthropicContent::ToolUse { id, name, input } => {
                OpenAiContent::ToolCalls(vec![OpenAiToolCall {
                    id: id.clone(),
                    r#type: "function".to_string(),
                    function: OpenAiFunctionCall {
                        name: name.clone(),
                        arguments: serde_json::to_string(input)?,
                    },
                }])
            }
            AnthropicContent::ToolResult { tool_use_id, content } => {
                OpenAiContent::Text(serde_json::to_string(&serde_json::json!({
                    "tool_use_id": tool_use_id,
                    "content": content,
                }))?)
            }
            AnthropicContent::Image { source } => {
                // 图片转换：Anthropic source -> OpenAI image_url
                OpenAiContent::ImageUrl(vec![OpenAiImageUrl {
                    url: format!("data:{};base64,{}", source.media_type, source.data),
                    detail: "auto".to_string(),
                }])
            }
        };
        
        Ok(OpenAiMessage {
            role: role.to_string(),
            content,
            name: None,
            tool_calls: None,
            tool_call_id: None,
        })
    }
    
    // ==================== OpenAI → Anthropic ====================
    
    pub fn openai_to_anthropic_request(
        openai_req: &OpenAiRequest,
        ctx: &ConversionContext,
    ) -> Result<AnthropicRequest> {
        let mut messages = Vec::new();
        let mut system = None;
        
        // 提取系统消息
        for msg in &openai_req.messages {
            if msg.role == "system" {
                system = match &msg.content {
                    OpenAiContent::Text(text) => Some(text.clone()),
                    _ => Some(serde_json::to_string(&msg.content)?),
                };
            } else {
                let anthropic_msg = Self::convert_openai_message_to_anthropic(msg)?;
                messages.push(anthropic_msg);
            }
        }
        
        // 转换工具定义
        let tools = openai_req.tools.as_ref().map(|tools| {
            tools.iter().map(|tool| AnthropicTool {
                name: tool.function.name.clone(),
                description: tool.function.description.clone(),
                input_schema: tool.function.parameters.clone(),
            }).collect()
        });
        
        Ok(AnthropicRequest {
            model: ctx.model_id.clone(),
            messages,
            system,
            max_tokens: ctx.max_tokens.unwrap_or(4096),
            temperature: ctx.temperature,
            tools,
            tool_choice: openai_req.tool_choice.as_ref().map(|tc| {
                match tc.as_str() {
                    "auto" => AnthropicToolChoice::Auto,
                    "required" => AnthropicToolChoice::Any,
                    other => AnthropicToolChoice::Tool { name: other.to_string() },
                }
            }),
            stream: openai_req.stream.unwrap_or(false),
            ..Default::default()
        })
    }
    
    fn convert_openai_message_to_anthropic(
        msg: &OpenAiMessage,
    ) -> Result<AnthropicMessage> {
        let role = match msg.role.as_str() {
            "user" => "user".to_string(),
            "assistant" => "assistant".to_string(),
            "tool" => "user".to_string(), // OpenAI tool role → Anthropic user
            _ => bail!("Unknown role: {}", msg.role),
        };
        
        let content = match &msg.content {
            OpenAiContent::Text(text) => AnthropicContent::Text(text.clone()),
            OpenAiContent::ImageUrl(urls) => {
                if let Some(url) = urls.first() {
                    // 解析 data: URL 获取图片数据
                    let (media_type, data) = Self::parse_data_url(&url.url)?;
                    AnthropicContent::Image {
                        source: AnthropicImageSource {
                            media_type,
                            data,
                        },
                    }
                } else {
                    AnthropicContent::Text(String::new())
                }
            }
            OpenAiContent::ToolCalls(tool_calls) => {
                if let Some(tc) = tool_calls.first() {
                    AnthropicContent::ToolUse {
                        id: tc.id.clone(),
                        name: tc.function.name.clone(),
                        input: serde_json::from_str(&tc.function.arguments)?,
                    }
                } else {
                    AnthropicContent::Text(String::new())
                }
            }
        };
        
        Ok(AnthropicMessage {
            role,
            content,
        })
    }
}
```

#### 19.2 SSE 流式响应的双向转换

```rust
impl ProtocolConverter {
    /// 将 Anthropic SSE 流转换为 OpenAI SSE 流
    pub fn convert_anthropic_sse_to_openai(
        anthropic_sse: impl Stream<Item = Result<AnthropicSseEvent>>,
        model_id: &str,
    ) -> impl Stream<Item = Result<OpenAiSseChunk>> {
        anthropic_sse.map(move |event| {
            let event = event?;
            match event {
                AnthropicSseEvent::MessageStart { message } => {
                    Ok(OpenAiSseChunk {
                        id: message.id.clone(),
                        object: "chat.completion.chunk".to_string(),
                        created: Utc::now().timestamp(),
                        model: model_id.to_string(),
                        choices: vec![OpenAiChoiceDelta {
                            index: 0,
                            delta: OpenAiDelta {
                                role: Some("assistant".to_string()),
                                content: None,
                                tool_calls: None,
                            },
                            finish_reason: None,
                        }],
                    })
                }
                AnthropicSseEvent::ContentBlockDelta { index, delta } => {
                    match delta {
                        AnthropicDelta::TextDelta { text } => {
                            Ok(OpenAiSseChunk {
                                id: Uuid::new_v4().to_string(),
                                object: "chat.completion.chunk".to_string(),
                                created: Utc::now().timestamp(),
                                model: model_id.to_string(),
                                choices: vec![OpenAiChoiceDelta {
                                    index: index as u32,
                                    delta: OpenAiDelta {
                                        role: None,
                                        content: Some(text),
                                        tool_calls: None,
                                    },
                                    finish_reason: None,
                                }],
                            })
                        }
                        AnthropicDelta::InputJsonDelta { partial_json } => {
                            Ok(OpenAiSseChunk {
                                id: Uuid::new_v4().to_string(),
                                object: "chat.completion.chunk".to_string(),
                                created: Utc::now().timestamp(),
                                model: model_id.to_string(),
                                choices: vec![OpenAiChoiceDelta {
                                    index: index as u32,
                                    delta: OpenAiDelta {
                                        role: None,
                                        content: None,
                                        tool_calls: Some(vec![OpenAiToolCallDelta {
                                            index: 0,
                                            id: None,
                                            function: Some(OpenAiFunctionDelta {
                                                name: None,
                                                arguments: Some(partial_json),
                                            }),
                                        }]),
                                    },
                                    finish_reason: None,
                                }],
                            })
                        }
                    }
                }
                AnthropicSseEvent::MessageDelta { delta, usage } => {
                    Ok(OpenAiSseChunk {
                        id: Uuid::new_v4().to_string(),
                        object: "chat.completion.chunk".to_string(),
                        created: Utc::now().timestamp(),
                        model: model_id.to_string(),
                        choices: vec![OpenAiChoiceDelta {
                            index: 0,
                            delta: OpenAiDelta {
                                role: None,
                                content: None,
                                tool_calls: None,
                            },
                            finish_reason: Some(delta.stop_reason.unwrap_or("stop".to_string())),
                        }],
                    })
                }
                AnthropicSseEvent::MessageStop => {
                    // [DONE] 信号，调用方需要特殊处理
                    Ok(OpenAiSseChunk {
                        id: String::new(),
                        object: "chat.completion.chunk".to_string(),
                        created: Utc::now().timestamp(),
                        model: model_id.to_string(),
                        choices: vec![],
                    })
                }
                _ => {
                    Err(anyhow!("Unhandled SSE event type"))
                }
            }
        })
    }
}
```

---

### 二十、插件系统之 Provider 扩展规范

```rust
// crates/agw-core/src/plugin/manifest.rs

/// 插件清单（manifest.yaml）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub plugin_type: PluginType,
    pub wasm_module: String,
    pub api_version: String,  // 插件API版本
    pub permissions: Vec<PluginPermission>,
    pub dependencies: Vec<PluginDependency>,
    pub min_gateway_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginType {
    /// Provider扩展：接入新的AI服务商
    Provider {
        provider_id: String,
        provider_name: String,
        api_format: ApiFormat,
        base_url_template: String,
    },
    /// 协议转换扩展：支持新的API格式
    Transform {
        from_format: String,
        to_format: String,
    },
    /// 中间件扩展：请求/响应处理
    Middleware {
        stage: MiddlewareStage,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginPermission {
    HttpRequest,
    FileRead { path: String },
    FileWrite { path: String },
    EnvironmentAccess,
    Logging,
    MetricsAccess,
}

/// WASM插件接口定义（WASI）
#[link(wasm_import_module = "gateway")]
extern "C" {
    // 日志
    fn gw_log(level: i32, msg_ptr: *const u8, msg_len: u32);
    
    // HTTP请求（插件不能直接访问网络，需通过网关）
    fn gw_http_request(
        method_ptr: *const u8, method_len: u32,
        url_ptr: *const u8, url_len: u32,
        headers_ptr: *const u8, headers_len: u32,
        body_ptr: *const u8, body_len: u32,
        result_ptr: *mut u8, result_max_len: u32,
    ) -> i32;
    
    // 获取配置
    fn gw_get_config(key_ptr: *const u8, key_len: u32) -> *const u8;
    
    // 获取当前请求上下文
    fn gw_get_request_context() -> *const u8;
}

/// 插件必须导出的函数
#[link(wasm_import_module = "plugin")]
extern "C
```

## 附录：Provider内置配置速查

### Alaya

```yaml
provider_id: "alaya"
name: "Alaya"
homepage: "https://alaya.ai"
get_api_key_url: "https://console.alaya.ai/settings/api-keys"
api_format: "anthropic"

plans:
  Lite:   MiniMax-2.5, Claude Code, 免费, 100日
  Plus:   MiniMax-2.5+GLM-5, Claude+Kimi, ¥29/月, 500日
  Max:    全模型, 全工具, ¥99/月, 2000日
```

### Anthropic

```yaml
provider_id: "anthropic"
name: "Anthropic"
homepage: "https://anthropic.com"
get_api_key_url: "https://console.anthropic.com/settings/keys"
api_format: "anthropic"

plans:
  Default: Claude官方API, 自定义配额
```

### Kimi

```yaml
provider_id: "kimi"
name: "Kimi"
homepage: "https://kimi.moonshot.cn"
get_api_key_url: "https://platform.moonshot.cn/console/api-keys"
api_format: "anthropic"  # 或 openai（可配置）

plans:
  个人版: 免费额度
  企业版: 自定义
```

### OpenCode

```yaml
provider_id: "opencode"
name: "OpenCode"
homepage: "https://opencode.ai"
get_api_key_url: "https://opencode.ai/settings/api"
api_format: "openai"
```

### Kilo

```yaml
provider_id: "kilo"
name: "Kilo"
homepage: "https://kilo.ai"
get_api_key_url: "https://kilo.ai/dashboard/api-keys"
api_format: "openai"
```
