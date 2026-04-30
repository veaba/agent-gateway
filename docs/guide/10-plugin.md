# 第 10 章：插件系统

## 概述

agent-gateway 支持通过插件扩展功能。插件用 WebAssembly (WASM) 编写，运行在安全的沙箱环境中。

## 插件类型

| 类型 | 说明 |
|------|------|
| Provider | 扩展新的 AI 服务商 |
| Transform | 自定义协议转换 |
| Tool | 扩展其他功能 |

## 安装插件

### 本地插件

```bash
agw plugin install ./my-plugin.wasm
```

### 远程插件

```bash
agw plugin install https://example.com/plugins/my-plugin.wasm
```

### GitHub 插件

```bash
agw plugin install gh:username/repo
```

## 插件管理

### 列表

```bash
agw plugin list
```

输出：
```
ID          Name        Type      Status
my-provider Provider    Provider  Enabled
transform  Transform   Transform  Enabled
```

### 启用/禁用

```bash
agw plugin enable <id>
agw plugin disable <id>
```

### 卸载

```bash
agw plugin uninstall <id>
```

## 开发插件

### 插件清单

每个插件需要一个 `plugin.json` 清单：

```json
{
  "id": "my-provider",
  "name": "My Provider",
  "version": "1.0.0",
  "type": "provider",
  "description": "My custom AI provider",
  "entry": "plugin.wasm",
  "api_version": "0.1.0"
}
```

### WASI 接口

插件需要实现以下函数：

```rust
// 初始化
fn init() -> Result<()>;

// 处理请求
fn handle_request(request: Request) -> Result<Response>;

// 健康检查
fn health() -> Result<Health>;
```

### 构建插件

```rust
use wasmtime::*;

// 创建引擎
let engine = Engine::default();

// 加载插件
let module = Module::from_file(&engine, "plugin.wasm")?;
let instance = Instance::new(&module, &[])?;

// 调用
let init = instance.get_typed_func::<(), ()>("init")?;
init.call(())?;
```

## 插件 API

### 请求格式

```json
{
  "model": "my-model",
  "messages": [...],
  "options": {
    "temperature": 0.7,
    "max_tokens": 1024
  }
}
```

### 响应格式

```json
{
  "content": "...",
  "usage": {
    "input_tokens": 10,
    "output_tokens": 100
  },
  "finish_reason": "stop"
}
```

## 示例：自定义 Provider

```rust
use wasmtime::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Request {
    model: String,
    messages: Vec<Message>,
}

#[derive(Serialize, Deserialize)]
struct Response {
    content: String,
    usage: Usage,
}

#[no_mangle]
pub extern "C" fn handle_request(req: Request) -> Response {
    // 调用自定义 API
    let client = reqwest::blocking::Client::new();
    let response = client.post("https://my-api.example.com/v1/chat")
        .json(&req)
        .send()
        .unwrap()
        .json()
        .unwrap();
    
    Response {
        content: response.content,
        usage: response.usage,
    }
}
```

## 安全

插件运行在受限的沙箱中：

- **无文件系统访问**：插件无法直接读写文件
- **无网络监听**：只能发送 HTTP 请求
- **受限内存**：内存使用受限
- **无子进程**：无法启动其他进程

## 下一步

- [第 11 章：故障排查](11-troubleshooting.md) - 常见问题