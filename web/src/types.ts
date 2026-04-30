// API 类型定义

export interface Provider {
  provider_id: string
  name: string
  description: string
  logo_url?: string
  homepage: string
  docs_url: string
  get_api_key_url?: string
  setup_guide_url?: string
  api_format: 'anthropic' | 'openai' | 'custom'
  requires_api_key: boolean
  coding_plans: CodingPlan[]
  models: Model[]
  supported_agents: AgentRef[]
}

export interface CodingPlan {
  plan_id: string
  name: string
  description: string
  tier: 'free' | 'pro' | 'enterprise' | 'custom'
  supported_model_ids: string[]
  supported_agent_ids: string[]
  default_model_id: string
  default_agent_id: string
  quota_daily?: number
  quota_monthly?: number
  rpm_limit?: number
  price?: string
  features: string[]
}

export interface Model {
  model_id: string
  name: string
  description?: string
  context_length?: number
  capabilities: string[]
  provider_id: string
}

export interface AgentRef {
  agent_id: string
  name: string
}

export interface PlanQuota {
  quota_used: number
  quota_limit: number
}

export interface UserPlan {
  id: string
  provider_id: string
  plan_id: string
  name: string
  api_key: string
  selected_model_id: string
  bound_agents: AgentBinding[]
  enabled: boolean
  priority: number
  custom_quota_daily?: number
  custom_quota_monthly?: number
  custom_rpm_limit?: number
  alert_threshold?: number
  notes?: string
  created_at: string
  last_health_check?: string
  health_status: 'unknown' | 'healthy' | 'warning' | 'error' | 'disabled'
  quota_used?: number
  quota_limit?: number
}

export interface AgentBinding {
  agent_id: string
  configured: boolean
  config_status: 'not_configured' | 'auto_configured' | 'manually_configured' | 'config_error' | 'needs_update'
  last_connected?: string
  error_message?: string
}

export interface FallbackConfig {
  enabled: boolean
  max_attempts: number
  priority_order: string[]
}

export interface QuotaStatus {
  plan_id: string
  daily_used: number
  daily_limit: number
  monthly_used: number
  monthly_limit: number
  rpm_used: number
  rpm_limit: number
}

export interface RequestLog {
  request_id: string
  plan_id: string
  agent_id?: string
  model_id: string
  status_code?: number
  latency_ms?: number
  created_at: string
}

export interface Plugin {
  id: string
  name: string
  version: string
  author: string
  description: string
  status: 'enabled' | 'disabled' | 'error'
}

export interface PlanQuota {
  quota_used: number
  quota_limit: number
}