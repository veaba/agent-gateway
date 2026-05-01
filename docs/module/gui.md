# agent-gateway GUI 使用文档

## 概述

agw-gui 是 agent-gateway 的桌面应用程序，基于 Tauri v2 构建，提供图形界面管理多 AI 编码工具网关。

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

- Windows: `%APPDATA%\agent-gateway`
- macOS: `~/Library/Application Support/agent-gateway`
- Linux: `~/.config/agent-gateway`