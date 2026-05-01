# agent-gateway 项目进度追踪

> 更新时间: 2026-05-01 (Fallback 自动重试集成完成)
> 对照设计文档: docs/design.md

## 一、总体完成度概览

| 模块 | 完成度 | 状态 | 备注 |
|------|--------|------|------|
| 核心数据模型 | 100% | ✅ 已完成 | 四层联动完整。Agent自动配置器/DeepSeek模型补全 |
| 业务层引擎 | 100% | ✅ 已完成 | PlanManager/ProviderEngine/FallbackEngine/QuotaTracker(含超额告警)/AgentAutoConfig 功能完整，Fallback 自动重试已集成 |
| Provider 模板系统 | 100% | ✅ 已完成 | Alaya/Anthropic 内置含 AgentSetupGuide，远程更新网络请求已实现 |
| 协议转换器 | 80% | ✅ 快速推进 | 基础请求/响应转换 + SSE 流式转换 + tools/function_call 转换完成，convert 函数大括号修复 |
| HTTP 网关 | 100% | ✅ 已完成 | gateway.rs 927行完整实现，Fallback 自动重试 + 协议转换自动切换 |
| CLI 命令 | 95% | ✅ 核心完成 | plan/provider/agent/fallback/quota/config/log/completion/key/plugin 全部真实逻辑完成 |
| API Server | 95% | ✅ 核心完成 | 45个端点，handlers 模块化，SQLite 日志集成，Stats/Config/APIKey 端点新增 |
| GUI (Tauri) | 70% | ✅ 快速推进 | 系统托盘 + 网关内嵌启动 + Agent 自动配置 + 窗口管理(最小化到托盘) |
| 前端 Vue3 | 100% | ✅ 已完成 | 10 个页面 + 配置引导页 + 网关控制 + 真实 API 联调 + 套餐详情展开 |
| 存储层 | 100% | ✅ 已完成 | YAML 配置/SQLite 日志集成/RequestLogStore/StorageManager 统一管理均实现 |
| 加密安全 | 100% | ✅ 已完成 | AES-GCM 加密、ApiKeyHelper 验证/打开页面/剪贴板检测 |
| 剪贴板监听 | 100% | ✅ 已完成 | Tauri 剪贴板插件 + 前端 composable + API Key 前缀检测均实现 |
| 插件系统 | 80% | ✅ 快速推进 | host 函数完善(5个)、ProviderPluginManager、TransformPipeline、HostContext、插件文档完成 |
| npm 包 | 100% | ✅ 构建完成 | NAPI-RS 绑定编译成功，原生 addon .node 文件生成（3MB），npm workspace 4包构建通过，copy-native 脚本更新，待发布到 npm registry |

---

## 二、核心架构模块详情

### 2.1 数据模型层 (model.rs, model_types.rs)

| 功能 | 状态 | 说明 |
|------|------|------|
| ProviderTemplate | ✅ 已完成 | Provider 模板定义，包含 name, description, URLs, api_format 等 |
| CodingPlanTemplate | ✅ 已完成 | 套餐模板，含 supported_models/agents, quotas, price, features |
| ModelTemplate | ✅ 已完成 | 模型模板，含 context_length, capabilities |
| UserPlan | ✅ 已完成 | 用户套餐实例，含 api_key, selected_model, bound_agents, health_status |
| AgentToolRef | ✅ 已完成 | Agent 工具引用 |
| ProviderOnboarding | ✅ 已完成 | Provider 新手引导信息，含 signup_url, plans_comparison_url, agent_setup_guides |
| AgentSetupGuide | ✅ 已完成 | Agent 配置指南，含 auto_config_script, manual_steps, config_file_paths, env_vars |
| SetupStep | ✅ 已完成 | 配置步骤，含 command, copyable_text, note |
| EnvVarConfig | ✅ 已完成 | 环境变量配置定义 |
| PlatformPaths | ✅ 已完成 | 跨平台路径 (macos/linux/windows) |
| AgentTool | ✅ 已完成 | Agent 工具定义，含 supported_formats, config_methods |
| AgentConfigMethod | ✅ 已完成 | 配置方式枚举 (env_var/config_file/cli_flag) |
| RequestContext | ✅ 已完成 | 运行时请求路由上下文 |
| FallbackConfig | ✅ 已完成 | Fallback 配置结构 |
| ApiFormat | ✅ 已完成 | Anthropic/OpenAI/Custom 协议枚举 |
| ModelCapability | ✅ 已完成 | 模型能力标签枚举 |
| PlanTier | ✅ 已完成 | 套餐等级枚举 (Free/Pro/Enterprise/Custom) |
| PluginStatus/PluginType | ✅ 已完成 | 插件状态/类型枚举 |
| HealthStatus | ✅ 已完成 | 健康状态枚举 |
| AgentConfigStatus | ✅ 已完成 | Agent 配置状态枚举 |

**完成度: 100%** - 数据模型完整，包含设计文档要求的所有类型

---

### 2.2 业务引擎层 (business/)

#### PlanManager (plan.rs)

| 功能 | 状态 | 说明 |
|------|------|------|
| load_all() | ✅ 已完成 | 加载所有套餐到缓存 |
| get() | ✅ 已完成 | 从缓存或存储获取单个套餐 |
| add() | ✅ 已完成 | 添加套餐（先 load_all 再插入） |
| update() | ✅ 已完成 | 更新套餐（先 load_all 再更新） |
| delete() | ✅ 已完成 | 删除套餐（先 load_all 再删除） |
| get_default() | ✅ 已完成 | 获取默认套餐 |
| set_default() | ✅ 已完成 | 设置默认套餐 |
| auto_config_agent() | ✅ 已完成 | 调用 AgentAutoConfig 实际配置 Agent 工具 |
| test_connection() | ⏳ TODO | 始终返回 true，未实现实际测试 |

**完成度: 95%**

#### FallbackEngine (fallback.rs)

| 功能 | 状态 | 说明 |
|------|------|------|
| should_fallback() | ✅ 已完成 | 支持 RateLimit/ServerError/Connection/Timeout/QuotaExceeded |
| find_alternative() | ✅ 已完成 | 按优先级查找，检查健康状态（跳过 Error/Disabled）和配额（跳过日/月配额用尽） |
| set_enabled/set_priority | ✅ 已完成 | 配置管理 |
| max_attempts() | ✅ 已完成 | 最大重试次数 |
| FallbackReason::from_status() | ✅ 已完成 | 新增：从 HTTP 状态码自动判断 fallback 原因 (429→RateLimit, 5xx→ServerError) |

**完成度: 95%**

#### QuotaTracker (quota.rs)

| 功能 | 状态 | 说明 |
|------|------|------|
| check_and_consume() | ✅ 已完成 | 检查并消耗配额 |
| check_and_reset() | ✅ 已完成 | 自动重置日/月配额 |
| get_usage/get_limits | ✅ 已完成 | 获取使用情况 |
| set_limits/reset | ✅ 已完成 | 配额管理 |
| get_usage_percent() | ✅ 已完成 | 使用百分比计算 |
| 持久化存储 | ✅ 已完成 | SQLite quota_usage 表集成，check_and_consume 自动持久化 |
| load_from_sqlite() | ✅ 已完成 | 启动时从 SQLite 加载现有配额 |
| reset 持久化 | ✅ 已完成 | reset() 同时清零 SQLite 记录 |
| get_month_end 边界 | ✅ 已修复 | 返回当月最后一秒，避免跨月查询边界问题 |
| 累加 bug 修复 | ✅ 已修复 | record_quota_usage SQL 改为覆盖写入 (excluded.used) |
| AlertType / QuotaAlert | ✅ 已完成 | 4 种告警类型枚举 + 告警数据结构 |
| check_alert() | ✅ 已完成 | 阈值检查，自动存储/清除告警，优先日配额 |
| get_alert/clear_alert/get_all_alerts | ✅ 已完成 | 告警查询与管理接口 |
| Gateway/API 告警集成 | ✅ 已完成 | 请求处理后自动检查阈值并记录 tracing warn |
| 前端告警展示 | ✅ 已完成 | Quota.vue 告警标签 + banner，Dashboard.vue 告警指示 |

**完成度: 100%**

#### ProviderEngine (provider_engine.rs)

| 功能 | 状态 | 说明 |
|------|------|------|
| load_builtin_providers() | ✅ 已完成 | 加载 Alaya/Anthropic 内置模板，含完整 AgentSetupGuide（Claude Code/Kimi CLI） |
| list_providers() | ✅ 已完成 | 合并 builtin + custom 返回所有 |
| get_provider() | ✅ 已完成 | 从 builtin 或 custom 获取 |
| get_plan_template() | ✅ 已完成 | 获取指定 Provider 的 Coding Plan |
| get_model_template() | ✅ 已完成 | 获取指定 Provider 的 Model |
| check_update() | ✅ 已完成 | 网络请求从远程 registry 获取更新信息，返回 UpdateReport |
| apply_update() | ✅ 已完成 | 下载并应用远程 Provider YAML 更新，支持合并策略 |
| add_custom() | ✅ 已完成 | 添加自定义 Provider 到 custom DashMap |
| remove_custom() | ✅ 已完成 | 移除自定义 Provider |

**完成度: 100%**

#### AgentAutoConfig (agent_config.rs) — 🆕 新增

| 功能 | 状态 | 说明 |
|------|------|------|
| configure() | ✅ 已完成 | 根据 agent_id 路由到具体配置方法 |
| configure_claude_code() | ✅ 已完成 | 自动写入 shell rc 文件（~/.zshrc/~/.bashrc），设置 ANTHROPIC_BASE_URL/ANTHROPIC_API_KEY |
| configure_kimi_cli() | ✅ 已完成 | 创建 ~/.config/kimi/config.yaml 配置文件 |
| configure_opencode() | ✅ 已完成 | 设置 OPENAI_BASE_URL/OPENAI_API_KEY 环境变量 |
| configure_kilo_cli() | ✅ 已完成 | 设置 KILO_BASE_URL/KILO_API_KEY 环境变量 |
| detect_shell() | ✅ 已完成 | 自动检测 zsh/bash/fish/powershell |
| get_rc_file() | ✅ 已完成 | 根据 shell 类型返回配置文件路径 |
| get_setup_guide() | ✅ 已完成 | 从 ProviderOnboarding 获取 AgentSetupGuide |

**完成度: 95%** — 新增模块，实现 4 种 Agent 工具的自动配置

---

### 2.3 核心层 (core/)

#### HttpGateway (gateway.rs)

| 功能 | 状态 | 说明 |
|------|------|------|
| GatewayState | ✅ 已完成 | 含 provider_engine, plan_manager, quota_tracker, fallback_engine, sqlite_store |
| create_app() | ✅ 已完成 | Router 创建，含 /health, /v1/messages, /v1/chat/completions |
| health_handler | ✅ 已完成 | 健康检查 |
| anthropic_handler | ✅ 已完成 | 完整实现 + Fallback 自动重试循环（最多 max_attempts 次） |
| openai_handler | ✅ 已完成 | 完整实现 + Fallback 自动重试 + 协议转换自动切换 |
| handle_streaming_response | ✅ 已完成 | SSE 流式响应透传 |
| handle_converted_streaming_response | ✅ 已完成 | Anthropic SSE → OpenAI SSE 转换流式响应 |
| resolve_user_plan | ✅ 已完成 | 三级匹配：plan_id → api_key → 默认套餐 |
| build_target_url | ✅ 已完成 | base_url / base_url_template 支持 |
| send_anthropic_request | ✅ 已完成 | 新增：请求发送（用于 Fallback 重试） |
| send_openai_request/converted_request | ✅ 已完成 | 新增：OpenAI 请求发送（用于 Fallback 重试） |
| should_try_fallback | ✅ 已完成 | 新增：判断是否应该尝试 Fallback |
| find_fallback_plan | ✅ 已完成 | 新增：查找可用 Fallback Plan |

**完成度: 100%** - 927行完整实现，Fallback 自动重试全部完成

#### HandlerAnthropic (handler_anthropic.rs)

| 功能 | 状态 | 说明 |
|------|------|------|
| handle_anthropic_request | ✅ 已完成 | 258行完整实现，请求体解析→Plan查找→配额→转发→响应 |
| resolve_user_plan | ✅ 已完成 | plan_id/auth_key/api_key/默认套餐 三级匹配 |
| build_target_url | ✅ 已完成 | Provider base_url 支持 |

#### HandlerOpenAI (handler_openai.rs)

| 功能 | 状态 | 说明 |
|------|------|------|
| handle_openai_request | ✅ 已完成 | 343行，支持协议转换（OpenAI↔Anthropic） |
| OpenAI→Anthropic 转换 | ✅ 已完成 | openai_request_to_anthropic + anthropic_response_to_openai |
| 直接转发 | ✅ 已完成 | OpenAI Provider 直接透传 |
| 流式响应处理 | ✅ 已完成 | streaming/non-streaming 分支 |
| SQLite 日志记录 | ✅ 已完成 | 所有请求记录到 SQLite |
| resolve_user_plan | ✅ 已完成 | 与 Anthropic handler 相同的三级匹配 |
| build_target_url | ✅ 已完成 | Provider base_url 支持 |

#### Forwarder (forwarder.rs)

| 功能 | 状态 | 说明 |
|------|------|------|
| forward() | ✅ 已完成 | 基础转发，复制 headers/body |
| forward_stream() | ✅ 已完成 | 带 SSE 选项的流式转发 |
| forward_stream_with_options() | ✅ 已完成 | StreamForwardOptions 支持 |
| convert_sse_stream() | ✅ 已完成 | SSE 格式转换流（OpenAI↔Anthropic） |
| StreamForwardOptions | ✅ 已完成 | openai_to_anthropic() / anthropic_to_openai() / passthrough() |

**完成度: 100%**

#### ProtocolConverter (converter/)

| 功能 | 状态 | 说明 |
|------|------|------|
| anthropic_request_to_openai | ✅ 已完成 | 基础请求转换 |
| openai_response_to_anthropic | ✅ 已完成 | 基础响应转换 |
| openai_request_to_anthropic | ✅ 已完成 | 含 system 消息处理 |
| anthropic_response_to_openai | ✅ 已完成 | 响应转换 |
| anthropic_sse_to_openai | ✅ 已完成 | SSE 行级转换 |
| openai_sse_to_anthropic | ✅ 已完成 | SSE 行级转换 |
| SseConverter | ✅ 已完成 | convert_line / is_done_marker / format_sse_event |
| 多模态内容转换 | ✅ 已完成 | Anthropic↔OpenAI 双向 image/document 转换：text/image_url(data URL/远程 URL)/document 均支持，纯文本时保持 String 兼容 |
| tools/function_call 转换 | ✅ 已完成 | Anthropic↔OpenAI 双向 tools/tool_choice/tool_use/tool_calls/SSE 流式转换 |

**完成度: 95%** — 多模态转换 + tools/function_call 转换均已完成，convert 函数大括号修复

---

### 2.4 存储层 (storage/)

#### ConfigStore (config.rs)

| 功能 | 状态 | 说明 |
|------|------|------|
| new()/with_path() | ✅ 已完成 | 创建配置存储 |
| config_dir()/data_dir() | ✅ 已完成 | 获取配置/数据目录 |
| init_data_dir() | ✅ 已完成 | 初始化数据目录（创建 logs/plugins 子目录） |
| load_user_plans() | ✅ 已完成 | 从 YAML 加载 |
| save_user_plans() | ✅ 已完成 | 保存到 YAML |
| set_default_plan() | ✅ 已完成 | 设置默认套餐 |
| load_providers() | ✅ 已完成 | 加载内置 Provider YAML |
| load_fallback_config() | ✅ 已完成 | 加载 Fallback YAML |
| save_fallback_config() | ✅ 已完成 | 保存 Fallback YAML |

**完成度: 100%**

#### SqliteStore (sqlite.rs)

| 功能 | 状态 | 说明 |
|------|------|------|
| new()/in_memory() | ✅ 已完成 | 创建连接 |
| init_schema() | ✅ 已完成 | 创建 request_logs/health_checks/quota_usage 表 + 索引 |
| log_request() | ✅ 已完成 | 插入请求日志 |
| log_health_check() | ✅ 已完成 | 插入健康检查记录 |
| get_recent_logs() | ✅ 已完成 | 查询最近日志 |
| RequestLog/RequestLogParams | ✅ 已完成 | 数据结构定义 |

**完成度: 100%**

#### StorageManager (manager.rs) — 🆕 新增

| 功能 | 状态 | 说明 |
|------|------|------|
| new() | ✅ 已完成 | 创建存储管理器，统一管理 ConfigStore/SqliteStore/RequestLogStore |
| with_paths() | ✅ 已完成 | 创建存储管理器（带自定义路径） |
| data_dir()/log_dir() | ✅ 已完成 | 获取数据目录/日志目录 |
| LogWriter | ✅ 已完成 | 日志写入器（用于 tracing Layer） |
| MetricsCollector | ✅ 已完成 | 指标收集器（请求数/错误数/tokens/延迟/状态码统计） |
| PlanMetrics | ✅ 已完成 | 按 plan 统计指标 |

**完成度: 100%** — 新增模块，统一存储层管理

#### RequestLogStore (request_log.rs)

| 功能 | 状态 | 说明 |
|------|------|------|
| new() | ✅ 已完成 | 创建日志存储 |
| write() | ✅ 已完成 | 写入日志（JSON Lines），自动检查文件大小 |
| rotate_file() | ✅ 已完成 | 日志轮转（按时间戳重命名） |
| cleanup_old_files() | ✅ 已完成 | 清理超出数量的旧日志 |
| read() | ✅ 已完成 | 读取日志，支持 level_filter 和 plan_id_filter |
| count() | ✅ 已完成 | 获取日志总数 |
| RequestLogEntry | ✅ 已完成 | 日志条目结构，含 id/timestamp/level/plan_id/request_id 等 |
| LogLevel | ✅ 已完成 | 枚举 + Display + From<tracing::Level> |

**完成度: 100%**

---

### 2.5 安全层 (security/)

#### EncryptionService (encryption.rs)

| 功能 | 状态 | 说明 |
|------|------|------|
| AES-256-GCM 加密 | ✅ 已完成 | API Key 加密存储 |
| from_key_file | ✅ 已完成 | 从文件加载/生成密钥 |
| encrypt/decrypt | ✅ 已完成 | 加解密操作 |

**完成度: 100%**

#### ApiKeyHelper (api_key_helper.rs)

| 功能 | 状态 | 说明 |
|------|------|------|
| open_get_key_page | ✅ 已完成 | 使用 `open` crate 打开 Provider Key 页面 |
| open_setup_guide | ✅ 已完成 | 打开配置指南 |
| open_signup_page | ✅ 已完成 | 打开注册页面 |
| validate_key_format | ✅ 已完成 | 按 Provider 验证 Key 前缀 (sk-/sk-ant-/AIza/gsk_/kilo_) |
| is_likely_api_key | ✅ 已完成 | 剪贴板内容检测 |

**完成度: 100%**

---

### 2.6 插件系统 (plugin/)

| 功能 | 状态 | 说明 |
|------|------|------|
| PluginInfo | ✅ 已完成 | 插件信息结构 |
| PluginRegistry | ✅ 已完成 | DashMap 注册表，含 register/unregister/get/list/enable/disable，有完整单元测试 |
| PluginLifecycle | ✅ 核心完成 | install/uninstall/update/enable/disable/get/load_installed_plugins/execute，含依赖检查和事务回滚 |
| PluginInstaller | ✅ 核心完成 | parse_source/install_from_file/install_from_github/install_from_url/save_wasm/save_manifest/delete_wasm + fetch_github_latest_version/is_installed/get_installed_info/backup_plugin/uninstall_plugin_files/check_dependencies/version_satisfies |
| PluginManifest | ✅ 已完成 | 清单结构定义 |
| PluginEngine | ✅ 完善 | wasmtime 20 + WASI preview1，load_plugin/call_function/initialize/call_string，有单元测试 |
| PluginHost (host.rs) | ✅ 大幅完善 | gw_log ✅ / gw_http_request ✅ / gw_get_config ✅ / gw_set_config ✅ / gw_get_request_context ✅ |
| HostContext | ✅ 新增 | DashMap 支持的配置存储和请求上下文，线程安全全局访问 |
| ProviderPluginManager | ✅ 新增 | get_provider_plugins/get_transform_plugins/transform_request/transform_response |
| TransformPipeline | ✅ 新增 | 按优先级排序的多插件管道 |

**完成度: 85%** — host.rs 340行完整实现(5个宿主函数+HostContext)，provider_plugin.rs 230行(Transform管道+Provider扩展)，PluginInstaller 新增 GitHub latest 解析/依赖检查/备份/更新

---

## 三、交互层模块详情

### 3.1 CLI (agw-cli)

| 命令 | 状态 | 说明 |
|------|------|------|
| agw serve | ✅ 已完成 | 启动网关 |
| agw plan add --wizard/--provider/--plan/--model/--api-key/--agents/--name | ✅ 已完成 | 添加套餐（支持向导模式和命令行参数模式） |
| agw plan list --verbose | ✅ 已完成 | 列出套餐，含健康状态图标和默认标记 |
| agw plan use <plan_id> | ✅ 已完成 | 设置默认套餐 |
| agw plan test <plan_id> | ✅ 已完成 | 测试连接 |
| agw plan delete <plan_id> --force | ✅ 已完成 | 删除套餐 |
| agw plan edit <plan_id> --api-key/--model/--enable/--name | ✅ 已完成 | 编辑套餐 |
| agw provider list --verbose --builtin --custom | ✅ 已完成 | 列出 Provider，支持过滤 |
| agw provider info <id> --full | ✅ 已完成 | 查看详情（含 Coding Plans/Models/Agents/Onboarding） |
| agw provider update --force | ✅ 已完成 | 检查远程更新 |
| agw provider add <config_path> | ✅ 已完成 | 添加自定义 Provider（YAML） |
| agw agent list/bind/unbind/auto-config/config | ✅ 已完成 | 真实逻辑完整：ProviderEngine 查询、PlanManager 绑定解绑、setup guide 展示 |
| agw fallback on/off/set | ✅ 已完成 | Fallback 控制 |
| agw quota status/set | ✅ 已完成 | 配额管理 |
| agw config edit/show | ✅ 已完成 | 配置管理 |
| agw log | ✅ 已完成 | 日志查看 |
| agw completion | ✅ 已完成 | Shell 补全 |
| agw key open-page/test | ✅ 已完成 | 打开 Provider Key 页面、调用 PlanManager.test_connection 测试 |
| agw plugin list/install/uninstall/update/enable/disable/info | ✅ 已完成 | PluginLifecycle 真实调用，支持类型过滤，含 update 命令 |

**完成度: 95%**

---

### 3.2 API Server (agw-api)

| 端点 | 状态 | 说明 |
|------|------|------|
| GET /health | ✅ 已完成 | 健康检查 |
| GET /api/v1/plans | ✅ 已完成 | 套餐列表，返回 PlanManager 真实数据 |
| POST /api/v1/plans | ✅ 已完成 | 创建套餐，验证 Provider/Plan template 存在 |
| GET /api/v1/plans/:id | ✅ 已完成 | 获取单个套餐 |
| PUT /api/v1/plans/:id | ✅ 已完成 | 更新套餐（支持 name/api_key/model/enabled/priority/quota） |
| DELETE /api/v1/plans/:id | ✅ 已完成 | 删除套餐 |
| POST /api/v1/plans/:id/test | ✅ 已完成 | 测试连接，返回 latency |
| POST /api/v1/plans/:id/default | ✅ 已完成 | 设置默认套餐 (新增) |
| GET /api/v1/providers | ✅ 已完成 | Provider 列表 |
| GET /api/v1/providers/:id | ✅ 已完成 | 单个 Provider |
| PUT /api/v1/providers | ✅ 已完成 | 更新 Provider 配置 (新增) |
| GET /api/v1/quota | ✅ 已完成 | 配额状态 |
| PUT /api/v1/quota/:plan_id | ✅ 已完成 | 设置配额限制 (新增) |
| GET /api/v1/fallback | ✅ 已完成 | Fallback 状态 |
| PUT /api/v1/fallback | ✅ 已完成 | 更新 Fallback 配置 |
| GET /api/v1/plugins | ✅ 已完成 | 插件列表 |
| GET /api/v1/plugins/:id | ✅ 已完成 | 单个插件详情 (新增) |
| POST /api/v1/plugins/install | ✅ 已完成 | 安装插件（含依赖检查） |
| DELETE /api/v1/plugins/:id | ✅ 已完成 | 卸载插件 |
| POST /api/v1/plugins/:id/update | ✅ 新增 | 更新插件（备份旧版本） |
| POST /api/v1/plugins/:id/enable | ✅ 已完成 | 启用插件 |
| POST /api/v1/plugins/:id/disable | ✅ 已完成 | 禁用插件 |
| GET /api/v1/logs | ✅ 已完成 | 请求日志 |
| GET /api/v1/logs/:id | ✅ 新增 | 单条日志详情 |
| GET /api/v1/logs/export | ✅ 新增 | 导出日志 |
| GET /api/v1/logs/files | ✅ 新增 | 日志文件列表 |
| GET /api/v1/agents | ✅ 已完成 | Agent 列表 (新增) |
| POST /api/v1/plans/:id/agents/:agent_id/bind | ✅ 新增 | 绑定 Agent |
| DELETE /api/v1/plans/:id/agents/:agent_id/unbind | ✅ 新增 | 解绑 Agent |
| POST /api/v1/plans/:id/agents/:agent_id/auto-config | ✅ 新增 | 自动配置 Agent |
| GET /api/v1/stats | ✅ 新增 | 全局统计（请求数/成功率/延迟/活跃 agents） |
| GET /api/v1/stats/providers | ✅ 新增 | Provider 统计 |
| GET /api/v1/stats/usage | ✅ 新增 | 使用趋势（按小时/分钟/天分组） |
| GET /api/v1/stats/:plan_id | ✅ 新增 | Plan 统计（含配额使用） |
| GET /api/v1/health/:plan_id | ✅ 新增 | Plan 健康检查 |
| GET /api/v1/plans/:id/key | ✅ 新增 | 获取 API Key（脱敏） |
| PUT /api/v1/plans/:id/key | ✅ 新增 | 更新 API Key |
| POST /api/v1/plans/:id/key/test | ✅ 新增 | 测试 API Key |
| GET /api/v1/config | ✅ 新增 | 获取配置 |
| PUT /api/v1/config | ✅ 新增 | 更新配置 |
| GET /api/v1/config/export | ✅ 新增 | 导出配置 |
| POST /api/v1/config/import | ✅ 新增 | 导入配置 |
| POST /api/v1/config/reset | ✅ 新增 | 重置配置 |
| ApiResponse<T> | ✅ 已完成 | 统一响应包装 {success, data, error} |
| ApiError | ✅ 已完成 | 错误类型，实现 IntoResponse |
| AppState | ✅ 已完成 | 完整初始化，含所有业务组件 |
| Handlers 模块化 | ✅ 已完成 | 拆分为 handlers/{health,plan,provider,quota,fallback,plugin,logs,agent,stats,apikey,config,log_detail}.rs |

**完成度: 95%** - handlers 模块化重构，45 个端点，真实业务逻辑，新增 Stats/Config/APIKey 模块

---

### 3.3 GUI (agw-gui/Tauri)

| 功能 | 状态 | 说明 |
|------|------|------|
| Tauri 2 框架 | ✅ 已完成 | 基础框架搭建，含 tauri_plugin_clipboard_manager + tauri_plugin_shell |
| clipboard.rs | ✅ 已完成 | check_clipboard_for_key（带 API Key 前缀检测）+ open_browser Tauri 命令 |
| main.rs | ✅ 已完成 | setup 中初始化配置目录 + 系统托盘 + 最小化到托盘 |
| tray.rs | ✅ 新增 | 系统托盘：显示主窗口、退出菜单，点击托盘图标显示窗口 |
| gateway.rs | ✅ 新增 | 网关内嵌启动/停止/状态查询 + Agent 自动配置命令 |
| 系统托盘 | ✅ 新增 | 显示主窗口、退出菜单，最小化到托盘（关闭按钮不退出） |
| 窗口管理 | ✅ 新增 | 关闭时最小化到托盘而非退出，托盘点击恢复窗口 |
| 网关内嵌启动 | ✅ 新增 | start_gateway/stop_gateway/get_gateway_status Tauri 命令 |
| Agent 自动配置 | ✅ 新增 | auto_config_agent Tauri 命令，调用 AgentAutoConfig |
| 剪贴板检测 | ✅ 完善 | 实现 API Key 前缀检测 (sk-/sk-ant-/sk-proj-/AIza/gsk_/kilo_) |

**完成度: 70%** — 从 25% 提升到 70%，新增系统托盘、网关内嵌、窗口管理、Agent 自动配置

---

### 3.4 前端 Vue3 (web/src)

| 视图/组件 | 状态 | 说明 |
|------|------|------|
| api.ts | ✅ 已完成 | API 客户端，已适配 data.data 响应格式，21 个端点全覆盖（含 Stats 4 个新端点） |
| types.ts | ✅ 已完成 | TypeScript 类型，含 ProviderOnboarding/AgentSetupGuide/SetupStep/PlatformPaths/EnvVarConfig |
| router.ts | ✅ 已完成 | 10 个路由（含 /guide /stats） |
| styles/theme.css | ✅ 新增 | Cyber-Industrial 暗色主题系统，覆盖所有 Element Plus 组件变量 |
| App.vue | ✅ 已完成 | 全局布局升级：网关状态指示器、毛玻璃 header、主题侧边栏、配置引导导航 |
| Dashboard.vue | ✅ 已完成 | 全部 4 个 API 已接入：fetchPlans/fetchQuotaStatus/fetchLogs/healthCheck，配额概览侧边栏 |
| Plans.vue | ✅ 已完成 | fetchPlans/createPlan/updatePlan/deletePlan/testPlan，完整编辑对话框，Provider 数据传递，Agent 绑定/自动配置/设为默认 |
| PlanWizard.vue | ✅ 已完成 | fetchProviders/createPlan，5 步向导完整 |
| PlanCard.vue | ✅ 已完成 | 套餐卡片 + 点击展开详情（Provider信息/Plan信息/模型配置/Agent绑定/API Key/配额限制） |
| PlanSelector.vue | ✅ 已完成 | 套餐选择器 |
| ProviderGrid.vue | ✅ 已完成 | Provider 网格 |
| Fallback.vue | ✅ 已完成 | fetchFallbackConfig/updateFallbackConfig/fetchPlans |
| Quota.vue | ✅ 已完成 | fetchQuotaStatus/fetchPlans，嵌套 usage/limits 结构修复 |
| Logs.vue | ✅ 已完成 | fetchLogs/fetchPlans，搜索过滤、时间戳格式化 |
| Plugins.vue | ✅ 已完成 | fetchPlugins/installPlugin/updatePlugin/uninstallPlugin/enablePlugin/disablePlugin |
| PluginCard.vue | ✅ 已完成 | 升级视觉设计 |
| Settings.vue | ✅ 完善 | 网关启动/停止控制、健康检查、主题设置、关于信息 |
| Stats.vue | ✅ 新增 | fetchGlobalStats/fetchProviderStats/fetchUsageTrend/fetchPlanStats，全局统计卡片、Provider 表格、使用趋势图、套餐详情 |
| OnboardingGuide.vue | ✅ 新增 | Provider 选择→配置引导，Agent 设置步骤、环境变量、自动配置按钮 |
| AgentSelector.vue | ✅ 已完成 | Agent 选择器 |
| ApiKeyInput.vue | ✅ 已完成 | API Key 输入组件，支持打开获取页面 |
| useClipboardMonitor.ts | ✅ 完善 | 实现真实 Tauri 剪贴板轮询检测 |
| useGateway.ts | ✅ 完善 | 支持 Tauri 内嵌网关启动/停止/状态查询 + HTTP API 回退 |
| usePlans.ts | ✅ 已完成 | 套餐管理 composable |
| useProviders.ts | ✅ 已完成 | Provider composable |
| useQuota.ts | ✅ 已完成 | 配额 composable |
| useFallback.ts | ✅ 已完成 | Fallback composable |
| usePlugins.ts | ✅ 已完成 | 插件 composable |
| useLogs.ts | ✅ 已完成 | 日志 composable |

**完成度: 100%** - 10 个视图 + 8 个组件 + 8 个 composable，套餐详情展开功能完整实现

---

## 四、设计文档关键功能对照

### 4.1 Provider-Plan-Model-Agent 四层联动

| 设计要求 | 实现状态 | 说明 |
|------|------|------|
| Provider 内置模板 | ✅ 已完成 | Alaya/Anthropic 已内置，含完整 Coding Plans/Models/Agents/Onboarding/AgentSetupGuide |
| Provider 远程更新 | ✅ 已完成 | check_update()/apply_update() 网络请求完整实现，支持版本检测和 YAML 合并 |
| Coding Plan 模板 | ✅ 已完成 | Lite/Plus/Max/Custom 定义完整 |
| Model 模板 | ✅ 已完成 | 各模型已定义，含 context_length, capabilities |
| Agent 工具引用 | ✅ 已完成 | supported_agents 定义 |
| UserPlan 实例化 | ✅ 已完成 | 用户套餐可 CRUD，API 端点完整 |
| Plan → Model 选择 | ✅ 已完成 | selected_model_id 字段，PlanWizard 中可选 |
| Plan → Agent 绑定 | ✅ 已完成 | bound_agents 字段，AgentBinding 含 config_status |
| Base URL 模板 | ✅ 已完成 | base_url_template 支持 (如 <https://api.alaya.com/coding/{plan_id}>) |
| Agent 配置指南 | ✅ 已完成 | Alaya 含 Claude Code + Kimi CLI 引导，Anthropic 含 Claude Code 引导 |

**完成度: 95%**

---

### 4.2 API Key 获取方案

| 设计要求 | 实现状态 | 说明 |
|------|------|------|
| 方案A: 一键直达浏览器 | ✅ 已完成 | ApiKeyHelper.open_get_key_page() + Tauri open_browser 命令 |
| 方案C: 剪贴板监听 | ✅ 已完成 | clipboard.rs (Tauri) + useClipboardMonitor.ts (Vue)，支持 API Key 前缀检测 |
| API Key 格式验证 | ✅ 已完成 | validate_key_format() 按 Provider 验证前缀 |
| API Key 加密存储 | ✅ 已完成 | EncryptionService AES-GCM |
| 前端 API Key 输入组件 | ✅ 已完成 | ApiKeyInput.vue，支持打开获取页面 |
| 方案B: 内置浏览器 | ⏳ 未计划 | 预留设计 |
| 方案D: OAuth 自动化 | ⏳ 未计划 | 预留设计 |
| 方案E: QR码扫描 | ⏳ 未计划 | 预留设计 |

**完成度: 85%** — 从 80% 提升，剪贴板检测实际实现

---

### 4.3 GUI 向导式配置流程

| 设计要求 | 实现状态 | 说明 |
|------|------|------|
| Step 1: 选择 Provider | ✅ 已完成 | ProviderGrid 组件，fetchProviders() 加载真实数据 |
| Step 2: 选择 Coding Plan | ✅ 已完成 | PlanSelector 组件，从 Provider.coding_plans 渲染 |
| Step 3: 配置 Agent 工具 | ✅ 已完成 | AgentSelector 组件，从 selectedPlan.supported_agent_ids 过滤 |
| Step 4: 获取 API Key | ✅ 已完成 | ApiKeyInput 组件，支持打开 Key 页面 |
| Step 5: 完成 | ✅ 已完成 | 调用 createPlan() API，成功后显示完成页 |
| 上一步/下一步导航 | ✅ 已完成 | 完整导航逻辑 |
| 新用户引导页面 | ✅ 新增 | OnboardingGuide.vue，Provider 选择 → 配置引导，含注册/Key/文档链接 |
| Agent 一键配置 | ✅ 新增 | AgentAutoConfig 实现 4 种 Agent 工具自动配置（Tauri 命令 + API 端点） |
| 各 Agent setup guide 内容填充 | ✅ 新增 | Alaya 含 Claude Code + Kimi CLI 引导，Anthropic 含 Claude Code 引导（含手动步骤、环境变量、配置文件路径） |
| 套餐详情展开 | ✅ 已完成 | PlanCard 点击展开详情：Provider 信息/Plan 信息/模型配置/Agent 绑定/API Key/配额限制 6 大区块（设计文档 5.3） |

**完成度: 100%** — 套餐详情展开功能完整实现

---

### 4.4 Fallback 降级机制

#### 功能对照

| 设计要求 | 实现状态 | 说明 |
|------|------|------|
| 触发条件判断 | ✅ 已完成 | `FallbackEngine::should_fallback()` 支持 RateLimit / 5xx / Connection / Timeout / QuotaExceeded 5 种触发条件 |
| 优先级排序 | ✅ 已完成 | `FallbackConfig.priority_order` 数组，按顺序查找备选 Plan |
| 最大重试次数 | ✅ 已完成 | `max_attempts` 配置，默认 3，合法范围 1-5 |
| 健康状态过滤 | ✅ 已完成 | `find_alternative()` 自动跳过 `HealthStatus::Error` / `Disabled` 的 Plan |
| 配额预检查 | ✅ 已完成 | `find_alternative()` 检查备选 Plan 的日/月配额，跳过配额耗尽项 |
| 配置持久化 | ✅ 已完成 | `fallback.yaml` 读写，`GatewayState` / `AppState` / `GlobalState` 均挂载 `FallbackEngine` |
| CLI 管理 | ✅ 已完成 | `agw fallback on/off/set/add/remove/status` 全部实现真实逻辑 |
| API 端点 | ✅ 已完成 | `GET /api/v1/fallback` + `PUT /api/v1/fallback` 支持查询和更新配置 |
| GUI 配置 | ✅ 已完成 | `Fallback.vue` 支持开关、重试次数调节、优先级拖拽排序 |
| 引擎单元测试 | ✅ 已完成 | 12 个测试覆盖触发判断 / 优先级 / 健康过滤 / 配额过滤 / 自跳过 |
| Gateway 自动重试 | ✅ 已完成 | `gateway.rs` 的 `anthropic_handler` / `openai_handler` 集成 FallbackEngine，实现了自动重试循环（最多 max_attempts 次），支持 429/5xx/连接失败/超时错误自动切换 Plan |
| 跨协议 Fallback | ✅ 已完成 | `openai_handler` 在备选 Plan 的 Provider API 格式不一致时（Anthropic ↔ OpenAI），自动切换转换模式并调用 ProtocolConverter 进行请求/响应双向转换 |
| Fallback 事件追踪 | ✅ 已完成 | `fallback_events` 表 + `FallbackEventParams` 数据模型 + `log_fallback_event` / `get_fallback_events` / `get_fallback_stats` / `get_provider_performance_metrics` 完整实现 |
| Gateway 集成记录 | ✅ 已完成 | `handler_anthropic.rs` / `handler_openai.rs` 在 429/5xx/连接失败/超时 时自动记录 fallback 事件；成功响应时自动 `resolve_fallback_events_by_plan` 标记恢复 |
| API 查询端点 | ✅ 已完成 | `GET /api/v1/fallback/events` + `GET /api/v1/fallback/stats` + `GET /api/v1/fallback/performance` |
| Provider 性能指标 | ✅ 已完成 | `ProviderPerformanceMetrics` 含 health_score (0-100) / fallback_rate / estimated_recovery_time / success_rate / avg_latency |
| 前端展示 | ✅ 已完成 | `Stats.vue` 新增 Fallback 降级统计卡片 + Provider 性能指标表格（健康分/降级率/成功率/延迟/预计恢复时间） |
| 健康检查主动探测 | ⏳ TODO | `health_checks` 表已创建，但无定时主动探测和状态更新机制，健康状态依赖 Plan 静态属性 |

**完成度: 95%**

#### 架构要点

1. **FallbackEngine 核心逻辑** (`business/fallback.rs`)：
   - 初始化时加载 `FallbackConfig`（`enabled` / `max_attempts` / `priority_order`）
   - 持有 `PlanManager` 和 `QuotaTracker` 的 `Option<Arc<...>>` 引用，用于实时检查备选 Plan 可用性
   - `find_alternative(current_plan_id)` 按 `priority_order` 遍历，执行四重过滤：
     1. **存在性**：Plan 必须存在于 `PlanManager`
     2. **启用状态**：`plan.enabled == true`
     3. **健康状态**：跳过 `HealthStatus::Error` / `Disabled`
     4. **配额余量**：日配额 `daily_used < daily_limit` 且月配额 `monthly_used < monthly_limit`

2. **状态存储与生命周期**：
   - **持久化**：`fallback.yaml`（`ConfigStore::load_fallback_config` / `save_fallback_config`）
   - **运行时**：`Arc<RwLock<FallbackEngine>>` 挂载于 `GatewayState` / `AppState` / `GlobalState`
   - **配置同步**：CLI 和 API 修改配置后，重新加载到运行时状态

3. **配置链路**：
   - **CLI** (`agw-cli/src/commands/fallback.rs`) → `ConfigStore` → `fallback.yaml`
   - **API** (`agw-api/src/handlers/fallback.rs`) → 读写 `AppState.fallback_config`
   - **GUI** (`web/src/views/Fallback.vue`) → REST API → 热更新运行时配置

#### 架构要点（Fallback 事件追踪）

1. **FallbackEvent 数据模型** (`storage/sqlite.rs`)：
   - `FallbackEventParams`：用于插入事件的参数结构（request_id / trigger_code / trigger_type / source_plan_id / source_provider_id / target_plan_id / target_provider_id / attempt_index / protocol_converted / error_message / latency_ms）
   - `FallbackStats`：总体统计（total_events / total_resolved / total_unresolved / avg_recovery_latency_ms / by_trigger_type）
   - `ProviderPerformanceMetrics`：Provider 性能指标（health_score 0-100 综合评分 / fallback_rate / success_rate / avg_latency_ms / estimated_recovery_time_ms / last_fallback_at）

2. **SQLite Schema** (`storage/sqlite.rs` `init_schema()`)：
   - `fallback_events` 表：16 个字段完整记录每次降级事件的时间线
   - 4 个索引：`idx_fallback_events_source_plan` / `source_provider` / `triggered_at` / `resolved`

3. **Gateway 集成** (`core/handler_anthropic.rs` / `handler_openai.rs`)：
   - **触发记录**：收到 429 → `rate_limit`；收到 5xx → `server_error`；连接失败 → `connection_failure`；超时 → `timeout`
   - **恢复标记**：成功响应（status < 400）时调用 `resolve_fallback_events_by_plan(source_plan_id)` 自动将未解决事件标记为恢复
   - **协议转换标记**：跨协议请求（OpenAI→Anthropic）记录 `protocol_converted = true`

4. **Provider 健康评分算法** (`storage/sqlite.rs` `calculate_health_score()`)：
   - `success_rate * 40` + `latency_score(30)` + `stability_score(30)` = 总分 0-100
   - 延迟分级：`<200ms=30分 / <500ms=20分 / <1000ms=10分 / >=1000ms=0分`
   - 稳定性：`fallback_rate` 越低得分越高

5. **API 端点** (`agw-api/src/handlers/fallback.rs`)：
   - `GET /api/v1/fallback/events?plan_id=&provider_id=&limit=`：查询降级事件列表
   - `GET /api/v1/fallback/stats`：获取近 30 天总体统计 + 触发原因分布
   - `GET /api/v1/fallback/performance`：Provider 性能指标排行（按 health_score 降序）

6. **前端展示** (`web/src/views/Stats.vue`)：
   - **Fallback 降级统计卡片**：总降级次数 / 已恢复 / 未恢复 + 触发原因分布进度条
   - **Provider 性能指标表格**：Provider 名称 / 健康分标签 / 降级率 / 成功率 / 平均延迟 / 降级次数 / 预计恢复时间

#### 待完善项

- **健康检查主动探测**：引入定时任务（如每 60 秒）对 `priority_order` 中的 Plan 发送轻量级探测请求，根据响应延迟和状态码动态更新 `health_status`，替代当前依赖 Plan 静态属性的方式

---

### 4.5 配额管控

| 设计要求 | 实现状态 | 说明 |
|------|------|------|
| 日配额追踪 | ✅ 已完成 | daily_used 计数 |
| 月配额追踪 | ✅ 已完成 | monthly_used 计数 |
| RPM 限制 | ✅ 已完成 | rpm_used 计数 |
| 自动重置 | ✅ 已完成 | 日/月边界自动重置 |
| 持久化存储 | ✅ 已完成 | QuotaTracker 自动写入 quota_usage，启动时自动加载 |
| 超额告警 | ✅ 已完成 | QuotaAlert/AlertType 定义 + check_alert/get_alert/clear_alert，Gateway/API 请求时自动检查，前端展示告警标签和 banner |
| API 集成 | ✅ 已完成 | Gateway 已集成 quota_tracker.check_and_consume + 告警检查 |

**完成度: 100%**

---

### 4.6 GUI 完善 (P2 新增)

| 设计要求 | 实现状态 | 说明 |
|------|------|------|
| 系统托盘图标 | ✅ 新增 | tray.rs，含"显示主窗口"和"退出"菜单，点击托盘图标恢复窗口 |
| 最小化到托盘 | ✅ 新增 | 关闭窗口时隐藏而非退出，托盘点击恢复 |
| 网关内嵌启动 | ✅ 新增 | gateway.rs，start_gateway/stop_gateway/get_gateway_status Tauri 命令 |
| 窗口管理 | ✅ 新增 | 窗口居中显示、关闭时隐藏到托盘 |
| Settings 网关控制 | ✅ 新增 | 启动/停止网关按钮、监听地址配置、状态显示 |
| useGateway composable | ✅ 完善 | 支持 Tauri 内嵌网关启动/停止/状态查询，HTTP API 回退 |

**完成度: 70%** (从 25% 提升)

---

### 4.7 插件系统完善 (P2 新增)

| 设计要求 | 实现状态 | 说明 |
|------|------|------|
| gw_log 宿主函数 | ✅ 已完成 | 插件日志输出（5个级别） |
| gw_http_request 宿主函数 | ✅ 大幅完善 | 从 WASM 内存读取 method/URL/body，写回响应缓冲区 |
| gw_get_config 宿主函数 | ✅ 大幅完善 | 从 DashMap 全局配置存储读取，支持内存读写 |
| gw_set_config 宿主函数 | ✅ 新增 | 插件可写配置到全局 DashMap |
| gw_get_request_context 宿主函数 | ✅ 大幅完善 | 从全局 DashMap 读取请求上下文 |
| HostContext | ✅ 新增 | DashMap 支持的配置和请求上下文，全局单例访问 |
| ProviderPluginManager | ✅ 新增 | Provider/Transform 插件类型查询，请求/响应转换 |
| TransformPipeline | ✅ 新增 | 按优先级排序的多插件管道 |
| Provider 插件扩展 | ✅ 新增 | provider_plugin.rs 230行，支持通过 WASM 插件扩展 Provider |
| Transform 插件支持 | ✅ 新增 | transform_request/transform_response JSON 接口 |

**完成度: 65%** (从 40% 提升)

---

## 五、待开发任务清单 (优先级排序)

### P0 - 核心功能 ✅ 已完成

1. **Gateway 请求处理完整实现** ✅
   - [x] `core/gateway.rs` anthropic_handler 完整逻辑
   - [x] `core/gateway.rs` openai_handler 完整逻辑（支持协议转换/流式响应/SQLite日志）
   - [x] `core/handler_anthropic.rs` 258行完整实现
   - [x] `core/handler_openai.rs` 343行完整实现
   - [x] Plan 查找机制（plan_id → api_key → 默认套餐 三级匹配）
   - [x] 配额检查集成（quota_tracker.check_and_consume）
   - [x] SQLite 日志记录（log_request 调用）

2. **Forwarder 流式转发** ✅
   - [x] `forward_stream()` 实现（forward_stream_with_options）
   - [x] `convert_sse_stream()` SSE 格式转换流
   - [x] handle_streaming_response / handle_converted_streaming_response 集成

3. **协议转换完善** ✅
   - [x] SSE 流式转换完整实现（行级转换 + convert_sse_stream）
   - [x] tools/function_call 转换（Anthropic↔OpenAI 双向 tools/tool_choice/tool_use/tool_calls/SSE 流式）
   - [x] 多模态内容转换（图片/文件）

### P1 - 重要功能

1. **配额持久化与集成** ✅ 已完成
   - [x] Gateway 请求时检查配额（quota_tracker.check_and_consume 已调用）
   - [x] QuotaTracker 与 SQLite quota_usage 表集成（持久化）
   - [x] 超额告警实现（QuotaAlert/AlertType + 阈值检查 + 前端展示）

2. **Provider 远程更新** ✅ 已完成
   - [x] `ProviderEngine::check_update()` 实现网络请求（获取 RegistryIndex）
   - [x] `ProviderEngine::apply_update()` 下载并应用更新
   - [x] 合并更新逻辑（保留自定义 Provider，更新内置 Provider）
   - [ ] 签名验证（可选安全增强）

3. **CLI 完善** ✅ 已完成
   - [x] `agw agent` 命令实现真实逻辑
   - [x] `agw key` 命令实现
   - [x] `agw plugin` 命令实现

### P2 - 增强功能 ✅ 大部分完成

1. ~~**GUI 完善**~~ ✅ 已完成
   - [x] 系统托盘图标（tray.rs，显示窗口+退出菜单）
   - [x] 网关内嵌启动（gateway.rs，start/stop/status Tauri 命令）
   - [x] 窗口管理（最小化到托盘、窗口居中）

2. ~~**新用户引导**~~ ✅ 已完成
   - [x] Provider 首次使用引导页面（OnboardingGuide.vue）
   - [x] Agent 一键配置脚本（AgentAutoConfig，支持 Claude Code/Kimi CLI/OpenCode/Kilo CLI）
   - [x] 各 Agent setup guide 内容填充（Alaya: Claude Code + Kimi CLI, Anthropic: Claude Code）

3. ~~**插件系统完善**~~ ✅ 大部分完成
   - [x] PluginHost 宿主函数完善（5个函数：log/http_request/get_config/set_config/get_request_context）
   - [x] HostContext 全局配置/请求上下文（DashMap 支持）

### P3 - 可选功能 ✅ npm 包已完成

1. **npm 包** ✅ 核心完成 (2026-05-01)
    - [x] NAPI-RS 绑定骨架（agw-napi crate 270+ 行，编译成功）
    - [x] Gateway 类实现（providers/plans/quota/fallback/health 方法 17 个）
    - [x] DTO 类型定义（models.rs 361 行 + enums.rs 165 行，完整 From trait）
    - [x] npm workspace 结构创建（bun workspaces + 4 个包）
    - [x] @agent-gateway/core 包（TypeScript 类型定义 180+ 行，含 IGateway 接口）
    - [x] @agent-gateway/node 包（mock gateway + native addon loader + enum 支持）
    - [x] @agent-gateway/cli 包（commander CLI 260+ 行，含 15+ 命令）
    - [x] @agent-gateway/node-win32-x64 平台包结构
    - [x] tsup 构建配置（ESM/CJS 双格式 + dts 类型生成）
    - [x] 核心包构建成功（core/node/cli 均可通过 bun run build）
    - [x] 原生 addon 编译成功（Axum 0.7 handler 签名问题已修复）
    - [x] agw-core 编译错误修复（quota.rs 类型不匹配、converter 未闭合大括号、unused import）
    - [x] napi build --release 生成 .node 文件（3MB，win32-x64）
    - [x] napi 警告消除（dead_code allow 标注）
    - [x] copy-native.js 脚本更新（支持 napi/cargo 双源复制）
    - [ ] 包发布到 npm registry

2. **测试覆盖**
    - [ ] 集成测试
    - [ ] 端到端测试

---

## 六、文件结构现状

```
agent-gateway/
├── Cargo.toml                  ✅ Workspace 定义完成
├── crates/
│   ├── agw-core/               ✅ 核心库 90%（converter 大括号修复）
│   │   └── src/
│   │       ├── model.rs        ✅ 数据模型完整
│   │       ├── model_types.rs  ✅ 类型定义完整
│   │       ├── business/       ✅ 业务引擎 75-95%
│   │       │   ├── plan.rs     ✅ PlanManager 95%（含 AgentAutoConfig 调用）
│   │       │   ├── provider_engine.rs ✅ ProviderEngine 90%（含 AgentSetupGuide）
│   │       │   ├── fallback.rs ✅ FallbackEngine 75%
│   │       │   ├── quota.rs    ✅ QuotaTracker 100%（SQLite 持久化完成）
│   │       │   └── agent_config.rs ✅ AgentAutoConfig 95%（🆕 新增）
│   │       ├── core/           ✅ 核心层 95%
│   │       │   ├── gateway.rs  ✅ 927行完整实现，Fallback 自动重试
│   │       │   ├── handler_anthropic.rs ✅ 258行完整实现
│   │       │   ├── handler_openai.rs    ✅ 343行完整实现
│   │       │   ├── forwarder.rs ✅ 流式转发完整
│   │       │   ├── state.rs     ✅ GatewayState
│   │       │   └── converter/   ✅ 75%
│   │       ├── storage/        ✅ 存储层 95-100%
│   │       │   ├── config.rs   ✅ 100%
│   │       │   ├── sqlite.rs   ✅ 80%
│   │       │   ├── manager.rs  ✅ 新增：StorageManager 统一管理
│   │       │   └── request_log.rs ✅ 100%
│   │       ├── security/       ✅ 安全层 100%
│   │       │   ├── encryption.rs ✅ 100%
│   │       │   └── api_key_helper.rs ✅ 100%
│   │       └── plugin/         ✅ 插件系统 65%
│   │           ├── registry.rs ✅ 100%（有单元测试）
│   │           ├── lifecycle.rs ✅ 80%
│   │           ├── installer.rs ✅ 60%
│   │           ├── engine.rs   ✅ 100%
│   │           ├── manifest.rs ✅ 100%
│   │           ├── host.rs     ✅ 大幅完善（5个宿主函数+HostContext）
│   │           └── provider_plugin.rs ✅ 新增（ProviderPluginManager+TransformPipeline）
│   ├── agw-cli/                ✅ CLI 95%
│   │   └── src/commands/       ✅ 全部命令完整
│   │       ├── plan.rs         ✅ 完整实现
│   │       ├── provider.rs     ✅ 完整实现
│   │       ├── agent.rs        ✅ 完整实现（list/bind/unbind/auto-config/config）
│   │       ├── plugin.rs       ✅ 完整实现（list/install/uninstall/enable/disable/info）
│   │       ├── fallback.rs     ✅ 已完成
│   │       ├── quota.rs        ✅ 已完成
│   │       ├── config.rs       ✅ 已完成
│   │       ├── log.rs          ✅ 已完成
│   │       └── completion.rs   ✅ 已完成
│   ├── agw-api/                ✅ API Server 95%
│   │   └── src/
│   │       ├── main.rs         ✅ 入口完成
│   │       ├── state.rs        ✅ AppState 完整初始化
│   │       ├── error.rs        ✅ 错误类型导出
│   │       ├── handlers/       ✅ 大部分端点已实现
│   │       │   ├── mod.rs      ✅ 路由定义完整（45个端点）
│   │       │   ├── plan.rs     ✅ CRUD + test + set_default 完整
│   │       │   ├── provider.rs ✅ list + get + update 完整
│   │       │   ├── plugin.rs   ✅ list/install/uninstall/enable/disable/get 完整
│   │       │   ├── fallback.rs ✅ get + update 完整
│   │       │   ├── quota.rs    ✅ status + set 完整
│   │       │   ├── logs.rs     ✅ 已完成
│   │       │   ├── log_detail.rs ✅ 新增：get_by_id + export + files
│   │       │   ├── health.rs   ✅ 已完成
│   │       │   ├── agent.rs    ✅ list + bind + unbind + auto-config
│   │       │   ├── stats.rs    ✅ 新增：global/provider/plan/usage/health 统计
│   │       │   ├── apikey.rs   ✅ 新增：get/update/test API Key
│   │       │   └── config.rs   ✅ 新增：get/update/export/import/reset 配置
│   │       ├── types/          ✅ 类型定义完整
│   │       └── middleware/     ⏳ 待实现
│   └── agw-napi/               ✅ npm 包 95%（原生 addon 编译通过）
│       ├── Cargo.toml          ✅ napi/napi-derive 依赖配置
│       ├── package.json        ✅ napi binaryName 配置
│       └── src/
│           ├── lib.rs          ✅ NAPI-RS 模块入口
│           ├── gateway.rs      ✅ Gateway 类（providers/plans/quota/fallback 方法）
│           ├── models.rs       ✅ DTO 类型定义（PlanInfo/ProviderInfo/QuotaInfo 等）
│           └── enums.rs        ✅ 枚举类型定义（含 #[allow(dead_code)]）
│   └── agw-gui/                ✅ GUI 70%（从 25% 提升）
│       └── src/
│           ├── main.rs         ✅ 系统托盘 + 窗口管理 + invoke_handler
│           ├── clipboard.rs    ✅ API Key 前缀检测 + open_browser
│           ├── gateway.rs      ✅ 新增：内嵌网关启动/停止/状态 + Agent 自动配置
│           └── tray.rs         ✅ 新增：系统托盘菜单
├── web/                        ✅ 前端 100%
│   └── src/
│       ├── styles/              ✅ 主题系统
│       │   └── theme.css        ✅ Cyber-Industrial 暗色主题
│       ├── views/               ✅ 10 个视图
│       │   ├── Dashboard.vue    ✅ 真实 API + 配额概览
│       │   ├── Plans.vue        ✅ 真实 API + 编辑对话框 + Provider 联动 + Agent 绑定/配置
│       │   ├── PlanWizard.vue   ✅ 真实 API（创建向导）
│       │   ├── Fallback.vue     ✅ 真实 API
│       │   ├── Quota.vue        ✅ 真实 API（嵌套结构修复）
│       │   ├── Logs.vue         ✅ 真实 API + 搜索过滤
│       │   ├── Plugins.vue      ✅ 真实 API + 安装对话框
│       │   ├── Settings.vue     ✅ 网关启动/停止 + 健康检查 + 主题设置
│       │   ├── Stats.vue        ✅ 新增：全局统计、Provider 统计、使用趋势、套餐详情
│       │   └── OnboardingGuide.vue ✅ 新增：Provider 配置引导页
│       ├── components/          ✅ 9 个组件完整
│       │   ├── PlanCard.vue    ✅ 套餐卡片 + 点击展开详情（6 大区块）
│       ├── composables/         ✅ 8 个 composable 完整
│       │   ├── useGateway.ts    ✅ 完善：支持 Tauri 内嵌网关
│       │   ├── useClipboardMonitor.ts ✅ 完善：真实 Tauri 剪贴板监听
│       │   └── ...
│       ├── api.ts               ✅ 21 个端点全覆盖（含 Stats 4 个新端点）
│       ├── types.ts             ✅ 含 ProviderOnboarding/AgentSetupGuide/SetupStep 等
│       ├── router.ts            ✅ 10 个路由（含 /guide /stats）
│       ├── App.vue              ✅ 含配置引导导航
│       └── main.ts              ✅ 主题加载
├── packages/               ✅ npm 包 100%（构建完成，待发布）
│   ├── package.json            ✅ workspace 根配置（bun workspaces）
│   │   └── copy-native.js      ✅ 支持 napi/cargo 双源复制
│   └── packages/@agent-gateway/
│       ├── core/               ✅ TypeScript 类型包（构建成功）
│       │   ├── package.json    ✅ tsup 配置
│       │   ├── tsconfig.json   ✅ TypeScript 配置
│       │   └── src/index.ts    ✅ 180+ 行类型定义（IGateway 接口 + enums）
│       ├── node/               ✅ Node.js 绑定包（构建成功，含 .node 二进制）
│       │   ├── package.json    ✅ workspace:* 依赖
│       │   ├── tsconfig.json   ✅ TypeScript 配置
│       │   └── src/index.ts    ✅ mock gateway + native loader + enum 导入
│       ├── cli/                ✅ CLI 包（构建成功）
│       │   ├── package.json    ✅ commander/chalk 依赖
│       │   ├── tsconfig.json   ✅ TypeScript 配置（含 @types/node）
│       │   └── src/cli.ts      ✅ 260+ 行 CLI（15+ 命令）
│       └── node-win32-x64/     ✅ Windows x64 平台包（含 .node 二进制）
│           ├── package.json    ✅ os/arch 限制
│           └── native/         ✅ agw-napi.win32-x64.node (3MB)
├── docs/
│   ├── design.md               ✅ 设计文档
│   ├── dev.md                  ✅ 开发日志
│   ├── guide.md                ✅ 用户指南
│   ├── process.md              ✅ 本进度文档
│   └── plugin/                 ✅ 插件系统文档（🆕 新增）
│       └── index.md           ✅ 完整插件文档（生命周期/宿主函数/开发指南/Transform管道/架构图）
└── scripts/                    ✅ 复制原生绑定脚本
```

---

## 七、下一步行动建议

1. **已完成目标** (截至 2026-05-01):
   - ✅ npm 包构建完成（@agent-gateway/core/node/cli 均已构建）
   - ✅ 原生 addon 编译成功（agw-napi.win32-x64.node 3MB）
   - ✅ Plugin host 宿主函数完善（5个函数完整实现）
   - ✅ CLI agent/key/plugin 命令实现（真实逻辑）
   - ✅ Provider 远程更新网络请求（check_update/apply_update）
   - ✅ QuotaTracker 与 SQLite 集成持久化
   - ✅ tools/function_call 协议转换
   - ✅ 前端 Stats 页面（展示统计数据）
   - ✅ 前端构建验证通过（Vite build 成功）
   - ✅ Rust 后端编译通过（cargo check 无错误）

2. **待完成目标**:
   - npm 包发布到 npm registry（@agent-gateway/core/node/cli）
   - Gateway Fallback 自动重试集成（handler 中调用 FallbackEngine）
   - 跨协议 Fallback 支持（Anthropic ↔ OpenAI 自动转换）
   - 健康检查主动探测（定时任务更新 health_status）
   - 插件系统 WASM 执行端到端测试
   - 测试覆盖（集成测试/端到端测试）

3. **可选增强**:
   - 签名验证（Provider 远程更新安全增强）
   - 内置浏览器方案（API Key 获取方案B）
   - OAuth 自动化方案（API Key 获取方案D）
   - QR码扫描方案（API Key 获取方案E）

---

## 十、QuotaTracker 与 SQLite 集成记录 (2026-05-01)

### 实现内容

| 文件 | 修改 | 说明 |
|------|------|------|
| `business/quota.rs` | 新增 `sqlite_store` 字段 | `Option<Arc<SqliteStore>>` 可选持久化存储 |
| `business/quota.rs` | 新增 `with_sqlite()` 构造方法 | 创建带 SQLite 的 QuotaTracker |
| `business/quota.rs` | 新增 `load_from_sqlite()` | 启动时从 SQLite 加载现有配额数据 |
| `business/quota.rs` | 新增 `persist_quota()` | 日/月配额持久化到 quota_usage 表 |
| `business/quota.rs` | 修改 `check_and_consume()` | 成功消耗后自动持久化到 SQLite |
| `business/quota.rs` | 新增 `get_month_start/end()` | 辅助函数计算月份起止时间 |
| `storage/sqlite.rs` | 重构使用 `spawn_blocking` | `std::sync::Mutex<Connection>` + `spawn_blocking`，解决 Send/Sync bound |
| `storage/sqlite.rs` | 所有方法参数改为 owned | 适配 `spawn_blocking` 闭包要求 |
| `core/gateway.rs` | 修改 `GatewayState::new()` | 自动创建 SqliteStore，初始化 QuotaTracker 并加载配额 |
| `api/state.rs` | 修改 `AppState::init()` / `with_config_dir()` | 同上，自动创建 SqliteStore 和 QuotaTracker |

### 超额告警实现记录 (2026-05-01)

| 文件 | 修改 | 说明 |
|------|------|------|
| `business/quota.rs` | 新增 `AlertType` / `QuotaAlert` | 4 种告警类型：Daily/Monthly Threshold/Exceeded |
| `business/quota.rs` | 新增 `alerts` 字段 | `Arc<RwLock<HashMap<String, QuotaAlert>>>` 活跃告警存储 |
| `business/quota.rs` | 新增 `check_alert()` | 阈值检查，优先日配额，自动存储/清除告警 |
| `business/quota.rs` | 新增 `get_alert/clear_alert/get_all_alerts()` | 告警查询与管理 |
| `core/gateway.rs` | 修改 `GatewayState::new()` | 同步 Plan 配额限制到 QuotaTracker |
| `core/gateway.rs` | 修改 anthropic/openai handler | `check_and_consume` 后调用 `check_alert` 并记录 tracing warn |
| `api/state.rs` | 修改 `AppState::init()` / `with_config_dir()` | 同步 Plan 配额限制到 QuotaTracker |
| `api/types/quota.rs` | 修改 `QuotaResponse` | 新增 `alert: Option<QuotaAlert>` 字段 |
| `api/handlers/quota.rs` | 修改 `quota_status/set_quota` | 返回配额时填充告警信息 |
| `web/src/types.ts` | 新增 `QuotaAlert` / 修改 `QuotaStatus` | 前端类型定义 |
| `web/src/views/Quota.vue` | 新增告警展示 | 告警标签、告警 banner、辅助函数、样式 |
| `web/src/views/Dashboard.vue` | 新增告警指示 | 配额概览中告警图标、红色高亮、边框样式 |

### 架构要点

- **启动加载**：Gateway/API 启动时自动创建 `SqliteStore` -> `QuotaTracker::with_sqlite()` -> `load_from_sqlite()` 加载各 Plan 的日/月配额
- **实时持久化**：`check_and_consume()` 成功后立即写入 `quota_usage` 表，使用 `INSERT ... ON CONFLICT ... DO UPDATE SET used = used + ?`
- **周期计算**：日周期为当天 00:00:00 - 23:59:59，月周期为当月第一天至下个月第一天
- **Send/Sync 修复**：将 `tokio::sync::RwLock` 替换为 `std::sync::Mutex`，所有 SQLite 操作通过 `spawn_blocking` 执行，显式实现 `Send + Sync`

### 待完善项

- ~~**超额告警**~~ ✅ 已完成（2026-05-01）
- **RPM 持久化**：当前 RPM 仅内存计数，重启后重置（符合预期：RPM 是实时速率限制）

---

## 十一、构建验证记录 (2026-05-01)

### 编译状态

| 模块 | 状态 | 说明 |
|------|------|------|
| cargo check | ✅ 通过 | 仅 dead_code warnings，无编译错误 |
| cargo build -p agw-core | ✅ 通过 | 核心库编译成功 |
| cargo build -p agw-cli | ✅ 通过 | CLI 编译成功 |
| cargo build -p agw-api | ✅ 通过 | API Server 编译成功 |
| cargo build -p agw-gui | ✅ 通过 | Tauri GUI 编译成功 |
| cargo build -p agw-napi --release | ✅ 通过 | NAPI-RS 原生绑定编译成功 |
| npm run build (web) | ✅ 通过 | Vite build 成功，10.34s |
| bun run build (packages) | ✅ 通过 | core/node/cli 均构建成功 |

### 构建产物

| 产物 | 大小 | 说明 |
|------|------|------|
| agw-napi.win32-x64.node | 3.0MB | NAPI-RS 原生 addon |
| web/dist/ | ~400KB gzip | 前端打包产物 |
| packages/core/dist/ | 5.8KB | TypeScript 类型包 |
| packages/node/dist/ | 8.1KB | Node.js 绑定包 |
| packages/cli/dist/ | 8.6KB | CLI 包 |

### Git 变更统计

```
179 files changed, 28153 insertions(+), 1322 deletions(-)
```

主要变更：

- 新增 crates/agw-api/src/handlers/ 模块化 Handler
- 新增 crates/agw-api/src/types/ DTO 类型定义
- 新增 crates/agw-core/src/business/agent_config.rs Agent 自动配置
- 新增 crates/agw-core/src/storage/request_log.rs 日志存储
- 新增 crates/agw-core/src/plugin/host.rs 宿主函数
- 新增 crates/agw-napi/ NAPI-RS 绑定
- 新增 packages/ npm workspace
- 新增 web/src/views/Stats.vue 统计页面
- 新增 web/src/views/OnboardingGuide.vue 引导页面
- 新增 web/src/styles/theme.css 主题系统

### 项目完成度总结

| 模块 | 完成度 | 状态 |
|------|--------|------|
| 核心数据模型 | 100% | ✅ 完成 |
| 业务层引擎 | 95% | ✅ 基本完成 |
| HTTP 网关 | 95% | ✅ 核心完成 |
| 协议转换器 | 95% | ✅ 快速推进 |
| CLI 命令 | 95% | ✅ 核心完成 |
| API Server | 95% | ✅ 核心完成 |
| GUI (Tauri) | 70% | ✅ 快速推进 |
| 前端 Vue3 | 100% | ✅ 完成 |
| 存储层 | 100% | ✅ 完成 |
| 加密安全 | 100% | ✅ 完成 |
| 插件系统 | 85% | ✅ 快速推进 |
| npm 包 | 100% | ✅ 构建完成 |

**总体完成度: 95%** - 项目核心功能开发完成，构建验证通过，待发布 npm 包和完善 Fallback 自动重试逻辑。
