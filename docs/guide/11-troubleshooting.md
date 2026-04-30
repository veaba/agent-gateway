# 第 11 章：故障排查

## 常见问题

### 无法启动

**问题**：端口被占用

```bash
# Windows
netstat -ano | findstr :8081
taskkill /PID <PID> /F

# Linux/macOS
lsof -i :8081
kill <PID>
```

**问题**：配置文件损坏

```bash
# 备份并重置
mv config.yaml config.yaml.bak
agw config reset
```

### 连接失败

**问题**：API Key 无效

```bash
# 测试连接
agw plan test my-plan

# 重新获取 API Key
agw key open-page <provider>
```

**问题**：网络问题

```bash
# 测试网络
ping api.anthropic.com
curl -v https://api.anthropic.com
```

### Fallback 循环

**问题**：无限 Fallback

检查配置：
- 确保套餐之间不互相 Fallback
- 设置 `max_attempts` 为合理值
- 检查套餐优先级

### 配额问题

**问题**：配额计算不准

```bash
# 重置配额统计
agw quota reset my-plan

# 查看详细统计
agw quota status --verbose
```

### 性能问题

**问题**：响应慢

1. 检查网络延迟
2. 检查模型大小
3. 检查 Fallback 次数

```bash
# 查看延迟统计
agw stats --plan my-plan
```

### 插件问题

**问题**：插件加载失败

```bash
# 查看插件错误
agw plugin list --verbose

# 重新加载
agw plugin reload <id>
```

## 日志分析

### 查看错误日志

```bash
agw log show --error -n 50
```

### 查看 Fallback 日志

```bash
agw log show --fallback -n 50
```

### 实时日志

```bash
agw log tail -f
```

## 调试模式

### 启用调试日志

```bash
agw serve --verbose
# 或
RUST_LOG=debug agw serve
```

### 调试 API 请求

```bash
curl -v http://127.0.0.1:8081/api/v1/plans
```

## 恢复步骤

### 重置所有配置

```bash
agw config reset --all
```

### 清除数据

```bash
# 备份
cp gateway.db gateway.db.backup

# 清除
rm gateway.db
agw serve  # 会重新创建
```

### 重新同步 Provider

```bash
agw provider update
```

## 性能调优

### 增加连接池

```yaml
# config.yaml
connection_pool:
  max_connections: 100
  idle_timeout: 300
```

### 启用缓存

```yaml
# config.yaml
cache:
  enabled: true
  ttl: 3600
```

## 获取帮助

### 查看版本

```bash
agw --version
```

### 查看帮助

```bash
agw --help
agw plan --help
```

### 报告问题

请在 GitHub 上报告问题：https://github.com/veaba/agent-gateway/issues