# Agent Gateway

统一网关，用于管理多个 AI 编程工具，支持 Provider-Plan-Model-Agent 四层架构、自动故障转移、配额控制和协议转换。

## 功能特性

- **多 Provider 支持**: Claude Code, Kimi Code, OpenCode, Kilo CLI
- **协议转换**: Anthropic ↔ OpenAI API 双向转换
- **自动回退**: 速率限制、服务器错误或配额耗尽时的智能故障转移
- **配额管理**: 按计划跟踪使用量和限制
- **多种接口**: CLI 界面、桌面 GUI (Tauri)、REST API 和库模式
- **安全存储**: AES-256-GCM 加密存储 API 密钥
- **插件系统**: 基于 wasmtime + WASI 的 WASM 可扩展性

## 架构图

```txt
┌─────────────────────────────────────────────────┐
│  CLI (agw-cli) │ GUI (agw-gui) │ API (agw-api) │
└──────────────────────┬──────────────────────────┘
                       ↓
┌─────────────────────────────────────────────────┐
│               agw-core (共享核心库)              │
│  ┌──────────────────────────────────────────┐  │
│  │  Provider → Plan → Model → Agent 四层    │  │
│  │  层级架构                                │  │
│  └──────────────────────────────────────────┘  │
│  ┌─────────────┬─────────────┬─────────────┐  │
│  │ 业务逻辑    │   代理      │   存储      │  │
│  │ (回退/配额) │   (HTTP     │   (SQLite   │  │
│  │             │   网关)     │   + YAML)   │  │
│  └─────────────┴─────────────┴─────────────┘  │
└─────────────────────────────────────────────────┘
```

## 安装

### 前置要求

- Rust 1.75+
- Node.js 18+ (用于 GUI 前端)

### 从源码构建

```bash
# 克隆仓库
git clone https://github.com/veaba/agent-gateway.git
cd agent-gateway

# 构建所有组件
cargo build

# 或者构建特定二进制文件
cargo build -p agw-cli --release     # 仅 CLI
cargo build -p agw-gui --release     # 桌面 GUI
cargo build -p agw-api --release     # REST API 服务器
```

## 快速开始

### CLI 使用

```bash
# 启动网关
agw serve

# 列出 Provider
agw provider list

# 添加计划
agw plan add --wizard

# 测试连接
agw plan test <plan_id>

# 启用回退
agw fallback on

# 查看配额状态
agw quota status
```

### REST API 服务器

```bash
# 启动 API 服务器（默认端口：8081）
cargo run -p agw-api

# 或者运行编译好的二进制文件
./target/release/agw-api.exe
```

#### API 端点

| 方法   | 端点                    | 描述             |
|--------|-------------------------|------------------|
| GET    | `/health`               | 健康检查         |
| GET    | `/api/v1/plans`         | 列出所有计划     |
| POST   | `/api/v1/plans`         | 创建计划         |
| GET    | `/api/v1/providers`     | 列出 Provider    |
| GET    | `/api/v1/quota`         | 获取配额状态     |
| GET    | `/api/v1/fallback`      | 获取回退配置     |
| POST   | `/api/v1/fallback`      | 更新回退配置     |

### 桌面 GUI

```bash
# 构建并运行 Tauri 桌面应用
cargo build -p agw-gui --release
./target/release/agw-gui.exe
```

## 配置

配置文件存储位置：

- **Linux/macOS**: `~/.config/agent-gateway/`
- **Windows**: `%APPDATA%\agent-gateway\`

### 关键文件

- `config.yaml` - 主配置文件
- `plans.db` - SQLite 数据库（计划和配额）
- `keys.enc` - 加密的 API 密钥

## 技术栈

| 组件         | 技术                |
|--------------|---------------------|
| 核心语言     | Rust (2021 edition) |
| 异步运行时   | Tokio               |
| HTTP 服务器  | Axum 0.7            |
| HTTP 客户端  | Reqwest 0.12        |
| GUI 框架     | Tauri v2            |
| CLI 框架     | Clap v4.5           |
| 数据库       | SQLite (rusqlite)   |
| 日志         | tracing             |

## License
