# 第 1 章：简介

## 什么是 agent-gateway

agent-gateway 是一个统一网关，用于管理多个 AI 编码工具。它在单个应用程序中整合了多个 AI 服务商（Provider），支持自动故障转移（Fallback）、配额控制和协议转换。

## 核心特性

- **多 Provider 支持**：Anthropic、OpenAI、Google、通义千问、Kilo Code 等
- **自动 Fallback**：主 Provider 失败时自动切换到备用 Provider
- **配额控制**：每日/每月 token 限额、请求速率限制
- **协议转换**：Anthropic Messages API ↔ OpenAI Chat Completions
- **多接入方式**：CLI、REST API、桌面 GUI、Node.js 绑定

## 架构概览

```
用户请求
    │
    ▼
┌──────────────────────────────────────┐
│           agent-gateway               │
│  ┌────────────────────────────────┐ │
│  │    请求路由与协议转换           │ │
│  └────────────────────────────────┘ │
│  ┌────────┬────────┬────────┐    │
│  │ Fallback│ Quota  │ Proxy  │    │
│  │ Manager│ Manager│       │    │
│  └────────┴────────┴────────┘    │
└──────────────────────────────────────┘
    │
    ▼
AI Provider (Primary)
    │
    ▼ (Fallback)
AI Provider (Backup)
```

## 数据模型

agent-gateway 使用四层体系结构：

1. **Provider** - AI 服务商（如 Anthropic、OpenAI）
2. **Plan** - 订阅套餐（如 Sonnet 4、GPT-4o）
3. **Model** - 具体模型（如 claude-sonnet-4-20250514）
4. **Agent** - 客户端工具（如 Claude Code）

## 使用方式

| 场景 | 推荐方式 |
|------|---------|
| 本地开发 | CLI 或 GUI |
| 服务器部署 | REST API |
| Node.js 集成 | @agent-gateway/node |
| 自定义集成 | agw-core 库 |

## 版本信息

当前版本：0.1.0