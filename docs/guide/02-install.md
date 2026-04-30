# 第 2 章：安装与配置

## 环境要求

- **Rust**: 1.75+
- **Node.js**: 18+ (用于 @agent-gateway/node)
- **操作系统**: Windows 10+, macOS 11+, Linux (Ubuntu 20.04+)

## 安装方式

### 方式一：CLI 工具

```bash
# 克隆项目
git clone https://github.com/veaba/agent-gateway.git
cd agent-gateway

# 构建 CLI
cargo build -p agw-cli --release

# 可执行文件位置
# Windows: target/release/agw.exe
# Linux/macOS: target/release/agw

# 添加到 PATH (可选)
# Windows:
# copy target/release/agw.exe %LOCALAPPDATA%\agent-gateway\agw.exe
# macOS/Linux:
# cp target/release/agw /usr/local/bin/agw
```

### 方式二：REST API 服务器

```bash
# 开发模式运行
cargo run -p agw-api

# 生产构建
cargo build -p agw-api --release
```

服务启动在 `http://127.0.0.1:8081`

### 方式三：桌面应用

```bash
# 构建 GUI
cargo build -p agw-gui --release

# 运行
# Windows: target/release/agw-gui.exe
# macOS: target/release/agw-gui
# Linux: target/release/agw-gui
```

### 方式四：Node.js 绑定

```bash
# 安装 npm 包
npm install @agent-gateway/node @agent-gateway/core

# 或使用 yarn
yarn add @agent-gateway/node @agent-gateway/core

# 或使用 bun
bun add @agent-gateway/node @agent-gateway/core
```

## 初始化配置

### 首次运行

首次运行时，系统会自动创建配置目录和默认配置：

**Windows**: `%APPDATA%\agent-gateway\`
**macOS**: `~/Library/Application Support/agent-gateway`
**Linux**: `~/.config/agent-gateway/`

### 配置文件

```yaml
# config.yaml
version: "2.0"
default_user_plan_id: null
user_plans: []
```

### 配置目录结构

```
agent-gateway/
├── config.yaml          # 用户配置
├── providers.yaml      # Provider 定义 (自动更新)
├── gateway.db         # SQLite 数据库
└── logs/
    └── gateway.log   # 运行日志
```

## 验证安装

### CLI

```bash
agw --version
# 输出: agw 0.1.0
```

### API 服务器

```bash
curl http://127.0.0.1:8081/health
# 输出: {"status":"ok","version":"0.1.0"}
```

### Node.js

```javascript
import { getGateway } from '@agent-gateway/node';

const health = getGateway().health();
console.log(health);
// { status: 'ok', version: '0.1.0' }
```

## 下一步

- [第 3 章：添加套餐](03-plan.md) - 配置你的第一个套餐
- [第 4 章：使用 CLI](04-cli.md) - 命令行操作指南