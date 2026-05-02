// TypeScript types for agent-gateway

// Enums
export enum ApiFormat {
  Anthropic = "Anthropic",
  OpenAi = "OpenAi",
  Custom = "Custom",
}

export enum PlanTier {
  Free = "Free",
  Pro = "Pro",
  Enterprise = "Enterprise",
  Custom = "Custom",
}

export enum ModelCapability {
  Code = "Code",
  Reasoning = "Reasoning",
  LongContext = "LongContext",
  ChineseOptimized = "ChineseOptimized",
  Math = "Math",
  Multimodal = "Multimodal",
}

export enum AgentConfigStatus {
  NotConfigured = "NotConfigured",
  AutoConfigured = "AutoConfigured",
  ManuallyConfigured = "ManuallyConfigured",
  ConfigError = "ConfigError",
  NeedsUpdate = "NeedsUpdate",
}

export enum HealthStatus {
  Unknown = "Unknown",
  Healthy = "Healthy",
  Warning = "Warning",
  Error = "Error",
  Disabled = "Disabled",
}

// Interfaces
export interface ProviderOnboarding {
  description: string;
  signup_url: string;
  plans_comparison_url?: string;
  get_key_url?: string;
  setup_guide_url?: string;
  faq_url?: string;
  agent_setup_guides: AgentSetupGuide[];
}

export interface AgentSetupGuide {
  agent_id: string;
  agent_name: string;
  auto_config_supported: boolean;
  auto_config_script?: string;
  manual_steps: SetupStep[];
  config_file_paths: PlatformPaths;
  env_vars: EnvVarConfig[];
}

export interface SetupStep {
  step_number: number;
  description: string;
  command?: string;
  copyable_text?: string;
  note?: string;
}

export interface EnvVarConfig {
  name: string;
  value: string;
  description: string;
}

export interface PlatformPaths {
  macos?: string;
  linux?: string;
  windows?: string;
}

export interface ProviderInfo {
  providerId: string;
  name: string;
  description: string;
  logo_url?: string;
  homepage: string;
  docs_url: string;
  get_api_key_url?: string;
  setup_guide_url?: string;
  api_format: ApiFormat;
  requires_api_key: boolean;
  onboarding: ProviderOnboarding;
  coding_plans: CodingPlanInfo[];
  models: ModelInfo[];
  supported_agents: AgentRefInfo[];
  version: string;
}

export interface CodingPlanInfo {
  plan_id: string;
  name: string;
  description: string;
  tier: PlanTier;
  supported_model_ids: string[];
  supported_agent_ids: string[];
  default_model_id: string;
  default_agent_id: string;
  quota_daily?: number;
  quota_monthly?: number;
  rpm_limit?: number;
  price?: string;
  features: string[];
}

export interface ModelInfo {
  model_id: string;
  name: string;
  description?: string;
  context_length?: number;
  capabilities: ModelCapability[];
  providerId: string;
}

export interface AgentRefInfo {
  agent_id: string;
  name: string;
}

export interface PlanInfo {
  id: string;
  providerId: string;
  plan_id: string;
  name: string;
  api_key_masked: string;
  selected_model_id: string;
  bound_agents: AgentBindingInfo[];
  enabled: boolean;
  priority: number;
  custom_quota_daily?: number;
  custom_quota_monthly?: number;
  custom_rpm_limit?: number;
  alert_threshold?: number;
  notes?: string;
  created_at: string;
  last_health_check?: string;
  health_status: HealthStatus;
}

export interface AgentBindingInfo {
  agent_id: string;
  configured: boolean;
  config_status: AgentConfigStatus;
  last_connected?: string;
  error_message?: string;
}

export interface FallbackConfigInfo {
  enabled: boolean;
  max_attempts: number;
  priority_order: string[];
}

export interface QuotaUsageInfo {
  plan_id: string;
  daily_used: number;
  monthly_used: number;
  rpm_used: number;
}

export interface QuotaLimitsInfo {
  daily?: number;
  monthly?: number;
  rpm?: number;
}

export interface QuotaInfo {
  plan_id: string;
  usage: QuotaUsageInfo;
  limits: QuotaLimitsInfo;
}

export interface HealthResponse {
  status: string;
  version: string;
}

// Input types
export interface CreatePlanInput {
  providerId: string;
  plan_id: string;
  name: string;
  api_key: string;
  selected_model_id: string;
  custom_quota_daily?: number;
  custom_quota_monthly?: number;
  custom_rpm_limit?: number;
  notes?: string;
}

export interface UpdatePlanInput {
  name?: string;
  api_key?: string;
  selected_model_id?: string;
  enabled?: boolean;
  priority?: number;
  custom_quota_daily?: number;
  custom_quota_monthly?: number;
  custom_rpm_limit?: number;
  alert_threshold?: number;
  notes?: string;
}

export interface TestConnectionResult {
  plan_id: string;
  success: boolean;
  message: string;
  latency_ms?: number;
}

// Gateway interface
export interface IGateway {
  listProviders(): Promise<ProviderInfo[]>;
  getProvider(providerId: string): Promise<ProviderInfo | null>;
  listPlans(): Promise<PlanInfo[]>;
  getPlan(id: string): Promise<PlanInfo | null>;
  createPlan(input: CreatePlanInput): Promise<PlanInfo>;
  updatePlan(id: string, input: UpdatePlanInput): Promise<PlanInfo>;
  deletePlan(id: string): Promise<void>;
  testPlan(id: string): Promise<TestConnectionResult>;
  getFallbackConfig(): Promise<FallbackConfigInfo>;
  setFallbackEnabled(enabled: boolean): Promise<FallbackConfigInfo>;
  setFallbackPriority(priorityOrder: string[]): Promise<FallbackConfigInfo>;
  getQuotaUsage(planId: string): Promise<QuotaInfo | null>;
  setQuotaLimits(planId: string, daily?: number, monthly?: number, rpm?: number): Promise<void>;
  validateApiKey(content: string): boolean;
  maskApiKey(apiKey: string): string;
  health(): HealthResponse;
}
