// API 类型定义

export interface Provider {
  providerId: string
  name: string
  description: string
  logoUrl?: string
  homepage: string
  docsUrl: string
  getApiKeyUrl?: string
  setupGuideUrl?: string
  apiFormat: 'anthropic' | 'openai' | 'custom' | string
  requiresApiKey: boolean
  onboarding?: ProviderOnboarding
  codingPlans?: CodingPlan[]
  models?: Model[]
  supportedAgents?: AgentRef[]
}

export interface CodingPlan {
  planId: string
  name: string
  description: string
  tier: 'free' | 'pro' | 'enterprise' | 'custom' | string
  supportedModelIds?: string[]
  supportedAgentIds?: string[]
  defaultModelId?: string
  defaultAgentId?: string
  quotaDaily?: number
  quotaMonthly?: number
  rpmLimit?: number
  price?: string
  features?: string[]
}

export interface Model {
  modelId: string
  name: string
  description?: string
  contextLength?: number
  capabilities: string[]
  providerId: string
}

export interface AgentRef {
  agentId: string
  name: string
}

// Full Agent definition
export interface Agent {
  agent_id: string
  name: string
  description?: string
  logo_url?: string
  homepage?: string
  install_url?: string
  supported_formats: string[]
  config_methods: string[]
}

export interface PlanQuota {
  quota_used: number
  quota_limit: number
}

export interface UserPlan {
  id: string
  providerId: string
  planId: string
  name: string
  apiKeyMasked?: string  // From backend (masked)
  selectedModelId: string
  boundAgents: AgentBinding[]
  enabled: boolean
  priority: number
  customQuotaDaily?: number
  customQuotaMonthly?: number
  customRpmLimit?: number
  alertThreshold?: number
  notes?: string
  createdAt: string
  lastHealthCheck?: string
  healthStatus: 'unknown' | 'healthy' | 'warning' | 'error' | 'disabled'
  quotaUsed?: number
  quotaLimit?: number
}

export interface AgentBinding {
  agentId: string
  configured: boolean
  configStatus: 'not_configured' | 'auto_configured' | 'manually_configured' | 'config_error' | 'needs_update'
  lastConnected?: string
  errorMessage?: string
}

export interface FallbackConfig {
  enabled: boolean
  max_attempts: number
  priority_order: string[]
}

export interface QuotaAlert {
  plan_id: string
  alert_type: 'daily_threshold' | 'monthly_threshold' | 'daily_exceeded' | 'monthly_exceeded'
  triggered_at: string
  usage_percent: number
  message: string
}

export interface QuotaStatus {
  plan_id: string
  usage: {
    daily_used: number
    monthly_used: number
    rpm_used: number
  }
  limits: {
    daily?: number
    monthly?: number
    rpm?: number
  }
  alert?: QuotaAlert
}

export interface RequestLog {
  id: string
  timestamp: string
  level: string
  message: string
  target?: string
  plan_id?: string
  request_id?: string
  agent_id?: string
  model_id?: string
  status_code?: number
  latency_ms?: number
  error?: string
}

export interface Plugin {
  id: string
  name: string
  version: string
  author: string
  description: string
  status: 'enabled' | 'disabled' | 'error'
}

// ── Stats Types ──

export interface GlobalStats {
  totalRequests: number
  totalErrors: number
  successRate: number
  avgLatencyMs: number
  totalInputTokens: number
  totalOutputTokens: number
  plansCount: number
  providersCount: number
  activeAgents: number
}

export interface ProviderStats {
  providerId: string
  providerName: string
  totalRequests: number
  plansCount: number
  avgLatencyMs: number
  successRate: number
}

export interface PlanStats {
  planId: string
  planName: string
  providerId: string
  totalRequests: number
  totalErrors: number
  successRate: number
  avgLatencyMs: number
  inputTokens: number
  outputTokens: number
  quotaUsage: QuotaUsageStats
}

export interface QuotaUsageStats {
  dailyUsed: number
  dailyLimit?: number
  dailyPercent: number
  monthlyUsed: number
  monthlyLimit?: number
  monthlyPercent: number
  rpmUsed: number
  rpmLimit?: number
  rpmPercent: number
}

export interface UsageTrendPoint {
  timestamp: string
  requests: number
  errors: number
  avgLatencyMs: number
  inputTokens: number
  outputTokens: number
}

export interface UsageTrend {
  points: UsageTrendPoint[]
  granularity: string
}

export interface ProviderOnboarding {
  description: string
  signup_url: string
  plans_comparison_url?: string
  get_key_url?: string
  setup_guide_url?: string
  faq_url?: string
  agent_setup_guides: AgentSetupGuide[]
}

export interface AgentSetupGuide {
  agent_id: string
  agent_name: string
  auto_config_supported: boolean
  auto_config_script?: string
  manual_steps: SetupStep[]
  config_file_paths: PlatformPaths
  env_vars: EnvVarConfig[]
}

export interface SetupStep {
  step_number: number
  description: string
  command?: string
  copyable_text?: string
  note?: string
}

export interface PlatformPaths {
  macos?: string
  linux?: string
  windows?: string
}

export interface EnvVarConfig {
  name: string
  value: string
  description: string
}

export interface PlanQuota {
  quota_used: number
  quota_limit: number
}

// ── Fallback Event Types ──

export interface FallbackEvent {
  id: number
  requestId: string
  triggeredAt: string
  triggerCode?: number
  triggerType: string
  sourcePlanId: string
  sourceProviderId?: string
  targetPlanId?: string
  targetProviderId?: string
  attemptIndex: number
  protocolConverted: boolean
  errorMessage?: string
  latencyMs?: number
  recoveredAt?: string
  recoveryLatencyMs?: number
  resolved: boolean
}

export interface TriggerTypeCount {
  triggerType: string
  count: number
}

export interface FallbackStats {
  totalEvents: number
  totalResolved: number
  totalUnresolved: number
  avgRecoveryLatencyMs?: number
  byTriggerType: TriggerTypeCount[]
}

export interface ProviderPerformance {
  providerId: string
  providerName: string
  totalRequests: number
  fallbackEvents: number
  fallbackRate: number
  avgLatencyMs: number
  successRate: number
  estimatedRecoveryTimeMs?: number
  lastFallbackAt?: string
  healthScore: number
}

// ── Custom Agent Types ──

export interface CustomAgent {
  id: string
  agentId: string      // code/identifier
  name: string
  version: string
  logoUrl?: string
  description?: string
  createdAt: string
  updatedAt: string
  isCustom: boolean
}

// ── Custom Provider Types ──

export interface CustomProvider {
  id: string
  providerId: string      // code/identifier
  name: string
  description?: string
  logoUrl?: string
  homepage?: string
  docsUrl?: string
  getApiKeyUrl?: string
  apiFormat: 'anthropic' | 'openai' | 'custom' | string
  baseUrl: string
  requiresApiKey: boolean
  models: CustomModel[]
  createdAt: string
  updatedAt: string
  isCustom: boolean
}

export interface CustomModel {
  modelId: string
  name: string
  description?: string
  contextLength?: number
  capabilities: string[]
}