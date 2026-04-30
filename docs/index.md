---
pageType: home
title: Agent Gateway
titleSuffix: '统一网关 - 管理多个 AI 编码工具'

hero:
  name: Agent Gateway
  text: |
    统一网关
    管理多个 AI 编码工具
  tagline: Provider-Plan-Model-Agent 四层体系 | 自动降级 | 协议转换
  actions:
    - theme: brand
      text: 快速开始
      link: ./guide/02-install
    - theme: alt
      text: API 文档
      link: ./guide/07-api

features:
  - title: 多提供商支持
    details: 支持 Claude Code, Kimi Code, OpenCode, Kilo CLI 等主流 AI 编码工具
    link: ./guide/01-intro
  - title: 四层体系架构
    details: Provider → Plan → Model → Agent 清晰的分层设计
    link: ./guide/03-plan
  - title: 自动降级
    details: 速率限制、服务器错误、连接超时自动切换到备选方案
    link: ./guide/05-fallback
  - title: 配额控制
    details: 按套餐限制使用量，防止超额使用
    link: ./guide/06-quota
  - title: 协议转换
    details: Anthropic Messages API ↔ OpenAI Chat Completions 双向转换
    link: ./guide/08-converter
  - title: 插件扩展
    details: WASM 插件系统，支持自定义扩展
    link: ./guide/10-plugin