# agent-gateway 开发指南

## 环境要求

- **Rust**: 1.75+ (2021 edition)
- **Node.js**: 18+
- **npm**: 9+
- **Tauri CLI**: `cargo install tauri-cli` (仅 GUI 构建需要)

## 项目结构

```
agent-gateway/
├── Cargo.toml                    # Workspace 配置
├── crates/
│   ├── agw-core/                 # 核心库
│   ├── agw-cli/                  # CLI 工具
│   ├── agw-gui/                  # Tauri 桌面应用
│   └── agw-api/                  # REST API 服务器
├── web/                          # Vue3 前端
└── docs/                         # 文档
```

---

## 运行方式

### 方式一：API 服务器 + 前端（推荐开发模式）

**1. 启动后端 API 服务：**

```bash
# 方式 A：直接运行（调试用）
cargo run -p agw-api
# 默认监听 http://127.0.0.1:8081

# 方式 B：后台长期运行（生产用）
cargo build -p agw-api --release

# Windows 后台运行
start "agw-api" ./target/release/agw-api.exe

# 或使用 nohup（Git Bash / Linux / macOS）
nohup ./target/release/agw-api.exe > agw-api.log 2>&1 &

# 指定端口
RUST_LOG=info cargo run -p agw-api
```

**验证服务：**
```bash
curl http://127.0.0.1:8081/health
# 返回: {"status":"ok","version":"0.1.0"}

# 查看日志
tail -f agw-api.log
```

**停止服务：**
```bash
# 查找进程
netstat -ano | grep 8081

# 停止（替换 PID 为实际进程 ID）
taskkill //PID <PID> //F   # Windows
kill <PID>                 # Linux/macOS
```

**2. 启动前端开发服务器：**

```bash
cd web
npm run dev
# 默认访问 http://localhost:5173
```

**3. 访问应用：**
- 打开浏览器访问 http://localhost:5173

---

### 方式二：纯 CLI 网关服务

```bash
# 开发模式运行
cargo run -p agw-cli -- serve

# 指定监听地址
cargo run -p agw-cli -- serve --listen 127.0.0.1:8080

# 后台运行
cargo run -p agw-cli -- serve --daemon

# 停止服务
cargo run -p agw-cli -- stop
```

**CLI 命令：**

```bash
# 套餐管理
cargo run -p agw-cli -- plan list
cargo run -p agw-cli -- plan add --wizard
cargo run -p agw-cli -- plan test <plan_id>

# Provider 管理
cargo run -p agw-cli -- provider list
cargo run -p agw-cli -- provider info <provider_id>
cargo run -p agw-cli -- provider update

# Agent 工具管理
cargo run -p agw-cli -- agent list
cargo run -p agw-cli -- agent bind <plan_id> <agent_id>
cargo run -p agw-cli -- agent auto-config <plan_id> <agent_id>

# Fallback 控制
cargo run -p agw-cli -- fallback on
cargo run -p agw-cli -- fallback set plan1,plan2

# 配额管理
cargo run -p agw-cli -- quota status
cargo run -p agw-cli -- quota set <plan_id> --daily 500

# API Key 助手
cargo run -p agw-cli -- key open-page <provider>
cargo run -p agw-cli -- key test <plan_id>

# 配置管理
cargo run -p agw-cli -- config edit
cargo run -p agw-cli -- config show

# Shell 补全
cargo run -p agw-cli -- completion bash
cargo run -p agw-cli -- completion zsh
```

---

### 方式三：Tauri 桌面应用

**1. 安装依赖：**

```bash
# 安装 Tauri CLI
cargo install tauri-cli

# 安装前端依赖
cd web
npm install
```

**2. 开发模式（热重载）：**

```bash
cd crates/agw-gui
cargo tauri dev
```

**3. 构建发布版本：**

```bash
cargo tauri build
# 输出在 crates/agw-gui/target/release/bundle/
```

---

### 方式四：独立 API 服务器

```bash
# 运行 REST API 服务器
cargo run -p agw-api

# API 端点
# GET  /health           - 健康检查
# GET  /api/v1/plans     - 获取套餐列表
# POST /api/v1/plans     - 创建套餐
# GET  /api/v1/providers - 获取 Provider 列表
# GET  /api/v1/quota     - 获取配额状态
# GET  /api/v1/fallback  - 获取 Fallback 配置
```

---

## 构建命令

```bash
# 检查项目能否编译
cargo check

# 构建所有 crate
cargo build

# 构建特定 crate
cargo build -p agw-core
cargo build -p agw-cli --release
cargo build -p agw-gui --release
cargo build -p agw-api --release

# 运行测试
cargo test
cargo test -p agw-core

# 运行单个测试
cargo test test_name_here

# 查看测试输出
cargo test -- --nocapture

# 格式化代码
cargo fmt

# 代码检查
cargo clippy --fix
```

---

## 开发工作流

### 1. 前端开发

```bash
cd web

# 安装依赖
npm install

# 开发服务器（热重载）
npm run dev

# 类型检查
npm run build
npx vue-tsc --noEmit

# 构建生产版本
npm run build
```

### 2. 后端开发

```bash
# 监听代码变化自动重新编译（需要 cargo-watch）
cargo install cargo-watch
cargo watch -x check -x test

# 调试
RUST_LOG=debug cargo run -p agw-cli -- serve
```

### 3. 全栈开发

终端 1 - 后端：
```bash
cargo run -p agw-api
```

终端 2 - 前端：
```bash
cd web && npm run dev
```

---

## 配置目录

项目配置存储在：

| 平台 | 路径 |
|------|------|
| Linux | `~/.config/agent-gateway/` |
| macOS | `~/Library/Application Support/agent-gateway/` |
| Windows | `%APPDATA%/agent-gateway/` |

**配置文件：**

- `user_plans.yaml` - 用户套餐配置
- `providers_builtin.yaml` - 内置 Provider 配置
- `fallback.yaml` - Fallback 配置
- `encryption.key` - 加密密钥

**数据目录：**

| 平台 | 路径 |
|------|------|
| Linux | `~/.local/share/agent-gateway/` |
| macOS | `~/Library/Application Support/agent-gateway/` |
| Windows | `%LOCALAPPDATA%/agent-gateway/` |

**数据文件：**

- `logs/` - 日志文件
- `plugins/` - 插件目录
- `quota.db` - 配额数据库
- `logs.db` - 请求日志数据库

---

## 常见问题

### 1. 编译失败

```bash
# 更新依赖
cargo update

# 清理并重新编译
cargo clean
cargo build
```

### 2. 前端依赖问题

```bash
cd web
rm -rf node_modules
npm install
```

### 3. Tauri 构建失败

```bash
# 安装 Tauri 依赖（Linux）
curl -fsSL https://tauri.app/install.sh | bash

# 或手动安装 WebView2 (Windows)
# 下载: https://developer.microsoft.com/en-us/microsoft-edge/webview2/
```

---

## 环境变量

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `RUST_LOG` | `info` | 日志级别 (trace, debug, info, warn, error) |
| `AGW_CONFIG_DIR` | - | 自定义配置目录 |
| `AGW_DATA_DIR` | - | 自定义数据目录 |

---

## 下一步

- 阅读 [design.md](./design.md) 了解项目设计
- 查看 [process.md](./process.md) 了解开发进度
- 参考 CLAUDE.md 了解代码规范
