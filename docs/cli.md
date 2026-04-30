# agent-gateway CLI 使用文档

## 概述

agw-cli 是 agent-gateway 的命令行工具，用于管理多 AI 编码工具网关。

## 安装

```bash
cargo build -p agw-cli --release
# 可执行文件位于 target/release/agw.exe (Windows) 或 target/release/agw (Linux/macOS)
```

## 命令

### 网关控制

```bash
agw serve              # 启动网关服务
agw serve --daemon    # 后台模式启动
agw stop              # 停止网关服务
```

### 套餐管理 (Plan)

```bash
# 交互式添加套餐
agw plan add --wizard

# 非交互式添加套餐
agw plan add --provider <provider_id> --plan <plan_id>

# 列出所有套餐
agw plan list

# 设置默认套餐
agw plan use <plan_id>

# 测试套餐连通性
agw plan test <plan_id>
```

### Provider 管理

```bash
# 列出所有 Provider
agw provider list

# 显示 Provider 详情
agw provider info <provider>

# 更新 Provider 定义
agw provider update
```

### Agent 工具管理

```bash
# 列出 Agent
agw agent list

# 绑定 Agent 到套餐
agw agent bind <plan_id> <agent_id>

# 解除绑定
agw agent unbind <plan_id> <agent_id>

# 自动配置 Agent
agw agent auto-config <plan_id> <agent_id>
```

### API Key 助手

```bash
# 打开 Provider 的 API Key 页面
agw key open-page <provider>

# 测试 API Key
agw key test <plan_id>
```

### Fallback 控制

```bash
agw fallback on                 # 启用自动 fallback
agw fallback off              # 禁用
agw fallback set <plan1,plan2> # 设置优先级顺序
```

### 配额管理

```bash
agw quota status               # 显示配额使用情况
agw quota set <plan> <limit>  # 设置配额
```

### 配置管理

```bash
agw config edit                # 在编辑器中打开配置
agw config show                # 显示当前配置
```

### Shell 补全

```bash
agw completion bash            # 生成 bash 补全
agw completion zsh            # 生成 zsh 补全
```

### 插件管理

```bash
agw plugin list                # 列出插件
agw plugin install <source>   # 安装插件
agw plugin uninstall <id>    # 卸载插件
```

### 日志管理

```bash
agw log show                  # 显示最近日志
agw log clear                # 清除日志
```

## 示例

```bash
# 完整配置流程
agw plan add --wizard                    # 交互式添加套餐
agw plan use my-plan                    # 设置为默认
agw agent bind my-plan claude-code         # 绑定 Agent
agw key open-page anthropic             # 打开 API Key 页面
agw key test my-plan                   # 测试连接

# 启动网关
agw serve
```