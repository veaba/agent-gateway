# agent-gateway 使用文档

## 简介

agent-gateway 是一个统一网关，用于管理多个 AI 编码工具（Claude Code, Kimi Code, OpenCode, Kilo CLI），支持 Provider-Plan-Model-Agent 四层体系、自动降级、配额控制和协议转换。

## 架构

```
┌──────────────────────┬──────────────────────────┐
│  CLI (agw-cli) │ GUI (agw-gui/Tauri) │ API (agw-api) │
└──────────────────────┬──────────────────────────┘
                       ↓
┌─────────────────────────────────────────────────┐
│               agw-core (共享库)               │
└─────────────────────────────────────────────────┘
```

## 模块文档

| 模块 | 说明 | 文档 |
|------|------|------|
| agw-core | 核心库 | [core.md](core.md) |
| agw-cli | 命令行工具 | [cli.md](cli.md) |
| agw-api | REST API 服务器 | [api.md](api.md) |
| agw-gui | 桌面应用程序 | [gui.md](gui.md) |
| @agent-gateway/core | TypeScript 类型 | [core-ts.md](core-ts.md) |
| @agent-gateway/node | Node.js 绑定 | [node-ts.md](node-ts.md) |
| 插件系统 | [plugin/README.md](plugin/README.md) |

## 其他 TypeScript 文档

| 文档 | 说明 |
|------|------|
| [core-ts.md](core-ts.md) | @agent-gateway/core 类型定义 |
| [node-ts.md](node-ts.md) | @agent-gateway/node 使用指南 |

## 其他文档

| 文档 | 说明 |
|------|------|
| [README.md](../README.md) | 项目简介 |
| [process.md](process.md) | 开发流程 |
| [design.md](design.md) | 设计文档 |
| [dev.md](dev.md) | 开发指南 |

## 快速开始

### 1. 安装 CLI

```bash
cargo build -p agw-cli --release
```

### 2. 添加套餐

```bash
agw plan add --wizard
```

### 3. 启动 API 服务器

```bash
cargo run -p agw-api
```

### 4. 测试

```bash
curl http://127.0.0.1:8081/health
```

## 技术栈

| 组件 | 技术 |
|------|------|
| 核心语言 | Rust (2021) |
| 异步运行时 | Tokio |
| HTTP 服务器 | Axum 0.7 |
| HTTP 客户端 | Reqwest 0.12 |
| GUI 框架 | Tauri v2 |
| CLI 框架 | Clap v4.5 |
| 数据存储 | SQLite (rusqlite) |
| 加密 | AES-GCM |
| 插件 | WASM (wasmtime + WASI) |