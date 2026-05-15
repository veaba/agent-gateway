# agent-gateway GUI 使用文档

## 概述

agw-gui 是 agent-gateway 的桌面应用程序，基于 Tauri v2 构建，提供图形界面管理多 AI 编码工具网关。

## 服务器模式

GUI 支持两种服务器模式：

### 嵌入式模式 (Embedded)

默认模式。GUI 内置完整服务器，同时提供：
- **Proxy 路由**：`/v1/messages`, `/v1/chat/completions` 等 AI 请求转发
- **管理 API**：`/api/v1/*` 端点用于配置管理

嵌入式服务器默认监听 `127.0.0.1:8080`，GUI 启动时自动联动启动。

### 外部模式 (External)

连接到外部独立部署的 API 服务器（如 `agw-api`）。适用于：
- 团队共享服务器
- 服务器远程部署
- 多客户端共享配置

### 配置文件

服务器配置位于 `~/.agent-gateway/agw-gui/server.yaml`：

```yaml
mode: Embedded                 # Embedded 或 External
embedded_listen: "127.0.0.1:8080"  # 嵌入式服务器监听地址
external_endpoint: null        # 外部服务器地址 (仅 External 模式)
auto_start: true               # GUI 启动时自动启动服务器
```

## 安装

```bash
cargo build -p agw-gui --release
# Windows: target/release/agw-gui.exe
# macOS: target/release/agw-gui
# Linux: target/release/agw-gui
```

## 功能

### 系统托盘

- 最小化到系统托盘
- 托盘图标右键菜单：
  - 显示/隐藏窗口
  - 启动/停止网关
  - 退出

### 核心功能

1. **套餐管理**
   - 添加/编辑/删除套餐
   - 设置默认套餐
   - 测试连接

2. **Agent 配置**
   - 自动配置 Agent
   - 绑定/解绑 Agent

3. **配额监控**
   - 实时配额使用显示
   - 超额预警

4. **Fallback 配置**
   - 启用/禁用自动降级
   - 设置优先级

5. **剪贴板监控**
   - 自动检测 API Key
   - 支持格式：`sk-`, `sk-ant-`, `sk-proj-`, `AIza`, `gsk_`, `kilo_`

### 窗口行为

- 关闭按钮 → 最小化到托盘（不退出）
- 彻底退出：托盘菜单 → 退出

## 界面预览

### 仪表盘 (Dashboard)

- 网关运行状态
- 服务器模式显示
- 当前使用套餐
- 配额使用情况
- 最近请求统计

### 套餐 (Plans)

- 套餐列表
- Provider 信息
- 模型选择
- Agent 绑定状态

### Providers

- 可用 Provider 列表
- 套餐模板
- 模型能力对比

### Fallback

- 启用状态
- 优先级顺序
- 触发条件统计

### 日志

- 请求日志
- 错误日志
- 统计图表

## 配置目录

统一存储在用户主目录：

| 平台 | 路径 |
|------|------|
| Windows | `C:\Users\<用户名>\.agent-gateway\` |
| macOS | `/Users/<用户名>/.agent-gateway/` |
| Linux | `/home/<用户名>/.agent-gateway/` |

可通过 `AGENT_GATEWAY_HOME` 环境变量覆盖。

## Tauri Commands

GUI 提供以下 Tauri invoke 命令：

### 服务器控制

| 命令 | 功能 |
|------|------|
| `start_full_server` | 启动完整服务器 (proxy + 管理 API) |
| `start_gateway` | 启动 Proxy 服务器 (legacy) |
| `stop_gateway` | 停止服务器 |
| `get_gateway_status` | 获取服务器状态 |
| `get_server_config` | 获取服务器配置 |
| `set_server_config` | 设置服务器配置 |

### 数据访问 (嵌入式模式)

| 命令 | 功能 |
|------|------|
| `fetch_plans` | 获取所有套餐 |
| `fetch_plan` | 获取单个套餐 |
| `fetch_providers` | 获取所有 Provider |
| `fetch_provider` | 获取单个 Provider |
| `fetch_fallback_config` | 获取 Fallback 配置 |
| `fetch_quota_status` | 获取配额状态 |
| `test_external_connection` | 测试外部服务器连接 |

## API 端点 (嵌入式模式)

嵌入式服务器提供完整端点：

### Proxy 端点

- `GET /health` - 健康检查
- `POST /v1/messages` - Anthropic Messages API
- `POST /v1/chat/completions` - OpenAI Chat Completions

### 管理 API 端点

- `GET /api/v1/plans` - 套餐管理
- `GET /api/v1/providers` - Provider 管理
- `GET /api/v1/quota` - 配额状态
- `GET /api/v1/fallback` - Fallback 配置
- 等 30+ 端点 (详见 API 文档)