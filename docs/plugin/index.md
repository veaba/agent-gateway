# agent-gateway 插件系统

## 概述

agent-gateway 的插件系统基于 **WebAssembly (WASM)** 沙箱运行时，使用 wasmtime 20 + WASI preview1 实现。插件在隔离的沙箱环境中执行，通过宿主函数（Host Functions）与网关核心交互。

### 插件类型

| 类型 | 说明 | 用途 |
|------|------|------|
| `provider` | Provider 插件 | 扩展新的 AI 服务商，自定义 API 格式 |
| `transform` | 转换插件 | 自定义协议转换，修改请求/响应 |
| `tool` | 工具插件 | 扩展功能，通用工具 |

### 插件状态

| 状态 | 说明 |
|------|------|
| `installed` | 已安装，未启用 |
| `enabled` | 已启用，可执行 |
| `disabled` | 已禁用 |
| `error` | 出错 |

---

## 生命周期

插件从安装到卸载的完整生命周期：

```
安装 (install) → 注册 (register) → 启用 (enable) → 执行 (execute) → 禁用 (disable) → 卸载 (uninstall)
                                                     ↑                    |
                                                     └────────────────────┘
```

### 1. 安装 (Install)

支持三种安装来源：

```bash
# 本地文件
agw plugin install /path/to/plugin.wasm
agw plugin install file:///path/to/plugin.wasm

# GitHub 仓库
agw plugin install github://owner/repo@v1.0.0
agw plugin install github://owner/repo          # 默认 latest

# 远程 URL
agw plugin install https://example.com/plugin.wasm
```

安装过程：
1. 下载/读取 WASM 二进制文件
2. 验证 WASM magic number (`\0asm`)
3. 尝试从 WASM 自定义段提取清单（`manifest`、`plugin_manifest`、`agw_manifest`）
4. 如果没有内嵌清单，尝试从同目录加载 `manifest.yaml`
5. 验证模块可加载
6. 保存 WASM 文件到 `~/.local/share/agent-gateway/plugins/{id}.wasm`
7. 保存清单文件到 `~/.local/share/agent-gateway/plugins/{id}.yaml`
8. 注册到内存注册表

### 2. 启用/禁用 (Enable/Disable)

```bash
agw plugin enable my-plugin
agw plugin disable my-plugin
```

只有 `enabled` 状态的插件才能被执行。

### 3. 执行 (Execute)

插件执行流程：
1. 从注册表获取插件信息
2. 检查插件状态是否为 `enabled`
3. 从磁盘读取 WASM 二进制
4. 创建 WASI 上下文（继承 stdio，预打开插件目录）
5. 添加 Gateway 宿主函数到 Linker
6. 实例化模块
7. 调用 `_initialize` 或 `init` 初始化函数
8. 调用目标函数，传递输入数据

### 4. 卸载 (Uninstall)

```bash
agw plugin uninstall my-plugin
```

删除 WASM 文件、清单文件，从注册表移除。

---

## 清单格式 (manifest.yaml)

每个插件可以包含一个 YAML 清单文件，描述插件的元数据：

```yaml
id: my-transform-plugin
name: My Transform Plugin
version: 1.0.0
description: 一个自定义协议转换插件
author: Author Name
plugin_type: transform
entry_point: transform
permissions:
  - http
  - log
  - config
dependencies:
  - id: core-utils
    version_range: ">=1.0.0"
wasm_target: wasm32-wasi
```

### 字段说明

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `id` | string | ✅ | 插件唯一标识符，建议使用反向域名格式 |
| `name` | string | ✅ | 插件显示名称 |
| `version` | string | ✅ | 语义化版本号 |
| `description` | string | ✅ | 插件描述 |
| `author` | string | ✅ | 作者 |
| `plugin_type` | enum | ✅ | `provider` / `transform` / `tool` |
| `entry_point` | string | ✅ | WASM 入口函数名 |
| `permissions` | string[] | ✅ | 所需权限列表：`http`、`log`、`config` |
| `dependencies` | Dependency[] | ✅ | 依赖的其他插件 |
| `wasm_target` | string | ✅ | WASM 编译目标，通常为 `wasm32-wasi` |

---

## 宿主函数 (Host Functions)

插件通过 WASM 导入函数调用网关提供的宿主函数。所有宿主函数在 `gateway` 命名空间下。

### gw_log — 日志输出

```c
// 函数签名
void gw_log(int32_t level, int32_t ptr, int32_t len);

// 日志级别
#define GW_LOG_TRACE 0
#define GW_LOG_DEBUG 1
#define GW_LOG_INFO  2
#define GW_LOG_WARN  3
#define GW_LOG_ERROR 4
```

**用途**: 从插件输出日志到网关的 tracing 系统。

**示例 (Rust)**:
```rust
fn gw_log(level: i32, msg: &str) {
    let msg_ptr = /* allocate in wasm memory */;
    let msg_len = msg.len() as i32;
    unsafe { gateway::gw_log(level, msg_ptr, msg_len); }
}
```

### gw_http_request — HTTP 请求代理

```c
// 函数签名
int32_t gw_http_request(
    int32_t method_ptr, int32_t method_len,  // HTTP 方法
    int32_t url_ptr, int32_t url_len,          // 请求 URL
    int32_t body_ptr, int32_t body_len,        // 请求体（可为空）
    int32_t response_buf_ptr,                   // 响应写入缓冲区
    int32_t response_buf_len                    // 响应缓冲区大小
);
// 返回: 响应长度（正数），或错误码（负数）
```

**用途**: 在插件内发起 HTTP 请求。目前同步实现有限制，建议仅用于简单场景。

**返回值**: 成功时返回响应体长度（正数），响应体写入 `response_buf_ptr` 指向的 WASM 内存；失败返回 `-1`。

### gw_get_config — 获取配置

```c
// 函数签名
int32_t gw_get_config(
    int32_t key_ptr, int32_t key_len,        // 配置键名
    int32_t value_ptr, int32_t value_buf_len // 值写入缓冲区及大小
);
// 返回: 值的实际长度（正数），或 -1 表示键不存在
```

**用途**: 从网关的全局配置中读取键值对。

**示例**: 获取网关监听地址：
```c
char buf[256];
int32_t len = gw_get_config("gateway.listen_addr", buf, sizeof(buf));
if (len > 0) {
    // buf 中包含 "127.0.0.1:8080"
}
```

### gw_set_config — 设置配置

```c
// 函数签名
int32_t gw_set_config(
    int32_t key_ptr, int32_t key_len,      // 配置键名
    int32_t value_ptr, int32_t value_len   // 配置值
);
// 返回: 0 表示成功
```

**用途**: 向网关的全局配置写入键值对，持久化到内存。

### gw_get_request_context — 获取请求上下文

```c
// 函数签名
int32_t gw_get_request_context(
    int32_t field_ptr, int32_t field_len,      // 字段名（如 "plan_id"）
    int32_t value_ptr, int32_t value_buf_len   // 值写入缓冲区
);
// 返回: 值的实际长度（正数），或 -1 表示字段不存在
```

**用途**: 在请求处理过程中获取当前请求的上下文信息，如 plan_id、agent_id、model 等。

**可用字段**（由网关在请求处理期间设置）：
- `plan_id` - 当前套餐 ID
- `agent_id` - 当前 Agent ID
- `model_id` - 当前模型 ID
- `request_id` - 请求唯一 ID

---

## 插件开发指南

### 环境要求

- Rust nightly (wasm32-wasi target)
- wasm-pack 或 cargo build --target wasm32-wasi

### Cargo.toml 配置

```toml
[package]
name = "my-plugin"
version = "1.0.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
wit-bindgen = "0.24"

[profile.release]
opt-level = "s"
lto = true
```

### 示例：简单的日志插件 (Tool 类型)

```rust
// src/lib.rs

/// 插件入口函数
#[no_mangle]
pub extern "C" fn main(input_ptr: i32, input_len: i32) -> (i32, i32) {
    // 通过 gw_log 输出日志
    let msg = "Hello from my-plugin!";
    let msg_bytes = msg.as_bytes();

    // 在实际实现中需要：
    // 1. 通过 malloc 分配 WASM 内存
    // 2. 将消息写入内存
    // 3. 调用 gw_log(2, msg_ptr, msg_len)

    // 返回结果
    let result = format!(r#"{{"status": "ok", "message": "processed"}}"#);
    let result_bytes = result.as_bytes();
    // 返回 (result_ptr, result_len)
    (0, result_bytes.len() as i32)
}

/// 初始化函数（可选）
#[no_mangle]
pub extern "C" fn _initialize() {
    // 初始化逻辑
}
```

### 示例：Transform 插件

Transform 插件需要实现两个函数：

```rust
// src/lib.rs

use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct TransformInput {
    #[serde(rename = "type")]
    input_type: String,
    method: Option<String>,
    url: Option<String>,
    headers: Option<std::collections::HashMap<String, String>>,
    body: Option<String>,
    status_code: Option<u16>,
}

#[derive(Serialize)]
struct TransformOutput {
    headers: std::collections::HashMap<String, String>,
    body: String,
    url: Option<String>,
    method: Option<String>,
}

/// 请求转换入口
#[no_mangle]
pub extern "C" fn transform_request(input_ptr: i32, input_len: i32) -> (i32, i32) {
    // 读取输入 JSON
    let input: TransformInput = /* parse from wasm memory */;

    // 构建转换后的请求
    let mut output = TransformOutput {
        headers: input.headers.unwrap_or_default(),
        body: input.body.unwrap_or_default(),
        url: input.url,
        method: input.method,
    };

    // 示例：添加自定义请求头
    output.headers.insert(
        "X-Plugin-Transformed".to_string(),
        "true".to_string()
    );

    // 返回结果 JSON
    let result = serde_json::to_string(&output).unwrap();
    let result_bytes = result.as_bytes();
    (0, result_bytes.len() as i32)
}

/// 响应转换入口
#[no_mangle]
pub extern "C" fn transform_response(input_ptr: i32, input_len: i32) -> (i32, i32) {
    // 类似 transform_request，但处理响应
    let input: TransformInput = /* parse from wasm memory */;

    let mut output = TransformOutput {
        headers: input.headers.unwrap_or_default(),
        body: input.body.unwrap_or_default(),
        url: None,
        method: None,
    };

    // 示例：添加响应头
    output.headers.insert(
        "X-Plugin-Response-Processed".to_string(),
        "true".to_string()
    );

    let result = serde_json::to_string(&output).unwrap();
    let result_bytes = result.as_bytes();
    (0, result_bytes.len() as i32)
}
```

对应的清单文件 `manifest.yaml`：

```yaml
id: my-transform-plugin
name: My Transform Plugin
version: 1.0.0
description: 自定义协议转换插件示例
author: Example Author
plugin_type: transform
entry_point: transform_request
permissions:
  - log
  - config
dependencies: []
wasm_target: wasm32-wasi
```

### 示例：Provider 插件

Provider 插件用于扩展新的 AI 服务商：

```yaml
id: my-provider-plugin
name: My Provider Plugin
version: 1.0.0
description: 自定义 AI 服务商插件
author: Example Author
plugin_type: provider
entry_point: handle_request
permissions:
  - http
  - log
  - config
dependencies: []
wasm_target: wasm32-wasi
```

---

## 内存传递协议

插件与网关之间通过 WASM 线性内存传递数据。字符串传递遵循以下协议：

### 字符串传入（宿主 → 插件）

1. 宿主获取 WASM 模块的 `memory` 导出
2. 调用 `malloc` 函数分配内存（需导出 `malloc` 函数）
3. 将数据写入分配的内存地址
4. 调用目标函数，传入 `(ptr, len)` 参数对
5. 读取返回值 `(result_ptr, result_len)`

```rust
// call_string 的内存传递流程（engine.rs 中的实现）
pub fn call_string(&mut self, name: &str, input: &str) -> Result<Vec<u8>> {
    // 1. 分配内存
    let (input_ptr,) = malloc_func.call(input_len)?;
    // 2. 写入数据
    memory.data_mut().copy_from_slice(input_bytes);
    // 3. 调用函数
    let (result_ptr, result_len) = func.call((input_ptr, input_len))?;
    // 4. 读取结果
    let result = memory.data()[result_ptr..result_ptr+result_len].to_vec();
    Ok(result)
}
```

### 宿主函数内存传递

宿主函数（如 `gw_get_config`）直接操作 WASM 内存：

1. 从 WASM 内存读取输入参数（key/field 字符串）
2. 在 WASM 内存中写入输出值
3. 返回写入的字节数（正数）或错误码（负数）

---

## Transform 管道

多个 Transform 插件可以组成管道，按优先级顺序依次处理：

```
请求 → Transform1 → Transform2 → Transform3 → ... → Provider
                                          ↑
响应 ← Transform3 ← Transform2 ← Transform1 ← ... ← Provider
```

### 管道使用示例 (Rust)

```rust
use agw_core::plugin::ProviderPluginManager;

async fn apply_transforms(
    manager: &ProviderPluginManager,
    headers: &HashMap<String, String>,
    body: &[u8],
    url: &str,
    method: &str,
) -> Result<TransformResult> {
    // 获取所有 Transform 插件
    let transform_plugins = manager.get_transform_plugins();

    let mut current_headers = headers.clone();
    let mut current_body = body.to_vec();
    let mut current_url = url.to_string();
    let mut current_method = method.to_string();

    // 按优先级顺序执行
    for plugin in transform_plugins {
        if plugin.status == PluginStatus::Enabled {
            let result = manager.transform_request(
                &plugin.id,
                &current_headers,
                &current_body,
                &current_url,
                &current_method,
            ).await?;

            current_headers = result.headers;
            current_body = result.body;
            if let Some(url) = result.url { current_url = url; }
            if let Some(method) = result.method { current_method = method; }
        }
    }

    Ok(TransformResult {
        headers: current_headers,
        body: current_body,
        url: Some(current_url),
        method: Some(current_method),
    })
}
```

### ProviderPluginTransform 结构

```rust
pub struct ProviderPluginTransform {
    pub plugin_id: String,
    pub provider_id: Option<String>,   // None = 所有 Provider
    pub supported_formats: Vec<String>, // 如 ["anthropic", "openai"]
    pub priority: u32,                   // 数字越小优先级越高
}
```

---

## 宿主上下文 (HostContext)

网关提供全局的 `HostContext`，用于在宿主函数之间共享数据：

```rust
use agw_core::plugin::global_host_context;

// 设置配置（可在网关初始化时或请求处理前）
let ctx = global_host_context();
ctx.set_config("gateway.listen_addr".to_string(), "127.0.0.1:8080".to_string());
ctx.set_config("gateway.version".to_string(), "0.1.0".to_string());

// 设置请求上下文（在请求处理期间）
ctx.set_request_context("plan_id".to_string(), "plan-123".to_string());
ctx.set_request_context("agent_id".to_string(), "claude-code".to_string());
```

### 配置键约定

| 键 | 说明 | 示例值 |
|-----|------|--------|
| `gateway.listen_addr` | 网关监听地址 | `127.0.0.1:8080` |
| `gateway.version` | 网关版本 | `0.1.0` |
| `plugin.{id}.enabled` | 插件启用状态 | `true`/`false` |
| 自定义键 | 插件自定义配置 | 任意值 |

---

## API 端点

插件管理通过 REST API 操作：

| 端点 | 方法 | 说明 |
|------|------|------|
| `/api/v1/plugins` | GET | 列出所有插件 |
| `/api/v1/plugins/:id` | GET | 获取单个插件详情 |
| `/api/v1/plugins/install` | POST | 安装插件 |
| `/api/v1/plugins/:id` | DELETE | 卸载插件 |
| `/api/v1/plugins/:id/enable` | POST | 启用插件 |
| `/api/v1/plugins/:id/disable` | POST | 禁用插件 |

### 安装插件

```bash
# 从本地文件安装
curl -X POST http://127.0.0.1:8081/api/v1/plugins/install \
  -H "Content-Type: application/json" \
  -d '{"source": "/path/to/plugin.wasm"}'

# 从 GitHub 安装
curl -X POST http://127.0.0.1:8081/api/v1/plugins/install \
  -H "Content-Type: application/json" \
  -d '{"source": "github://owner/repo@v1.0.0"}'

# 从 URL 安装
curl -X POST http://127.0.0.1:8081/api/v1/plugins/install \
  -H "Content-Type: application/json" \
  -d '{"source": "https://example.com/plugin.wasm"}'
```

---

## 文件存储

插件相关文件存储位置：

```
~/.local/share/agent-gateway/plugins/
├── my-plugin.wasm          # WASM 二进制文件
├── my-plugin.yaml         # 插件清单文件
├── another-plugin.wasm
└── another-plugin.yaml
```

| 文件 | 说明 |
|------|------|
| `{id}.wasm` | 编译后的 WASM 模块 |
| `{id}.yaml` | 插件清单（YAML 格式） |

---

## 架构图

```
┌───────────────────────────────────────────────────────────┐
│                    agent-gateway                          │
│                                                           │
│  ┌─────────┐    ┌──────────────┐    ┌────────────────┐  │
│  │ HTTP    │    │  Plugin       │    │  Host          │  │
│  │ Gateway │───▶│  Lifecycle    │───▶│  Functions     │  │
│  │ (Axum)  │    │  (install/    │    │  (gw_log/      │  │
│  │         │    │   enable/     │    │   gw_http_req/ │  │
│  │         │    │   execute)    │    │   gw_get_cfg/  │  │
│  └─────────┘    └──────────────┘    │   gw_set_cfg/  │  │
│                                       │   gw_get_ctx)  │  │
│  ┌─────────┐    ┌──────────────┐     └───────┬────────┘  │
│  │Plugin   │    │  Plugin      │             │             │
│  │Registry │    │  Engine      │             │             │
│  │(DashMap)│    │  (wasmtime)  │             │             │
│  └─────────┘    └──────────────┘             │             │
│                                       ┌───────┴────────┐  │
│  ┌──────────────────────────┐         │  WASM Instance  │  │
│  │  PluginManifest (YAML)  │         │                  │  │
│  │  PluginManifest (嵌入式)│         │  ┌────────────┐  │  │
│  │  PluginInstaller        │         │  │ WASI       │  │  │
│  │  PluginSource (Local/   │         │  │ Preview1   │  │  │
│  │   GitHub/URL)           │         │  └────────────┘  │  │
│  └──────────────────────────┘         └─────────────────┘  │
│                                                           │
│  ┌──────────────────────────┐    ┌──────────────────────┐ │
│  │  ProviderPluginManager  │    │  HostContext          │ │
│  │  (transform_request/    │    │  (DashMap config +   │ │
│  │   transform_response)  │    │   request_context)   │ │
│  └──────────────────────────┘    └──────────────────────┘ │
│                                                           │
│  ┌──────────────────────────┐                             │
│  │  TransformPipeline       │                             │
│  │  (多插件按优先级管道)    │                             │
│  └──────────────────────────┘                             │
└───────────────────────────────────────────────────────────┘
```

---

## 注意事项

1. **WASM 沙箱隔离**: 插件在 wasmtime 沙箱中运行，无法直接访问宿主文件系统或网络，只能通过宿主函数交互

2. **内存传递**: 插件与宿主之间通过 WASM 线性内存传递数据，传递字符串需要先 `malloc` 分配内存

3. **导出函数**: WASM 模块必须导出 `malloc` 函数用于内存分配，以及 `entry_point` 指定的入口函数

4. **初始化函数**: 如果模块导出 `_initialize` 或 `init` 函数，引擎会在执行前自动调用

5. **HTTP 限制**: `gw_http_request` 当前在同步 WASM 上下文中执行，完整异步支持需要 wasmtime async feature，暂时返回错误 JSON

6. **插件目录**: 默认位于 `~/.local/share/agent-gateway/plugins/`（Linux/macOS）或 `%LOCALAPPDATA%/agent-gateway/plugins/`（Windows）

7. **并发安全**: `PluginRegistry` 和 `HostContext` 使用 `DashMap` 实现线程安全，可在多线程环境中安全使用

8. **执行流程**: 每次执行插件时会重新加载 WASM 模块（当前实现），未来可考虑模块缓存优化性能