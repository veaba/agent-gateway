# 第 4 章：CLI 命令参考

## 命令总览

```
agw serve              启动网关服务
agw stop              停止网关服务
agw plan              套餐管理
agw provider          Provider 管理
agw agent             Agent 管理
agw fallback          Fallback 控制
agw quota             配额管理
agw config            配置管理
agw key               API Key 助手
agw completion         Shell 补全
agw plugin             插件管理
```

## 网关控制

### 启动网关

```bash
agw serve                    # 前台运行
agw serve --port 8081         # 指定端口
agw serve --daemon           # 后台运行
agw serve --config custom.yaml  # 指定配置
```

### 停止网关

```bash
agw stop
```

## 套餐命令

### 列表

```bash
agw plan list
```

### 添加

```bash
agw plan add --wizard              # 交互式
agw plan add --provider anthropic --plan sonnet-4  # 直接添加
```

### 使用

```bash
agw plan use <plan_id>           # 设置默认
agw plan test <plan_id>          # 测试连接
```

### 更新

```bash
agw plan update <plan_id> --name "新名称"
agw plan update <plan_id> --model <model_id>
agw plan update <plan_id> --enabled true
```

### 删除

```bash
agw plan delete <plan_id>
```

## Provider 命令

### 列表

```bash
agw provider list
```

### 详情

```bash
agw provider info <provider_id>
```

### 更新定义

```bash
agw provider update
```

## Agent 命令

### 列表

```bash
agw agent list
```

### 绑定

```bash
agw agent bind <plan_id> <agent_id>
agw agent unbind <plan_id> <agent_id>
```

### 自动配置

```bash
agw agent auto-config <plan_id> <agent_id>
```

## Fallback 命令

```bash
agw fallback on               # 启用
agw fallback off            # 禁用
agw fallback set plan1,plan2 # 设置优先级
agw fallback status         # 查看状态
```

## 配额命令

```bash
agw quota status                  # 查看状态
agw quota set <plan> --daily 100000
agw quota set <plan> --monthly 1000000
agw quota set <plan> --rpm 60
agw quota alert <plan> 80       # 设置警告阈值
```

## API Key 命令

```bash
agw key open-page <provider>  # 打开 Key 页面
agw key test <plan>        # 测试 Key
```

## 配置命令

```bash
agw config edit     # 编辑器打开
agw config show    # 显示配置
agw config path    # 显示配置路径
```

## Shell 补全

```bash
agw completion bash > /etc/bash_completion.d/agw
agw completion zsh > ~/.zsh/completions/_agw
```

## 日志命令

```bash
agw log show              # 显示最近日志
agw log show --error      # 仅错误
agw log show -n 50      # 最近 50 条
agw log clear            # 清除日志
```

## 插件命令

```bash
agw plugin list
agw plugin install <source>
agw plugin uninstall <id>
agw plugin enable <id>
agw plugin disable <id>
```

## 全局选项

```
--help, -h          帮助
--version, -V       版本
--config PATH       配置文件路径
--verbose, -v      详细输出
--quiet, -q        安静模式
```