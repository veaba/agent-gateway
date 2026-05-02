/**
 * @agent-gateway/node - Node.js bindings for agent-gateway
 *
 * This package wraps the native NAPI-RS bindings for use in Node.js environments.
 */

import type {
  ProviderInfo,
  PlanInfo,
  CreatePlanInput,
  UpdatePlanInput,
  TestConnectionResult,
  FallbackConfigInfo,
  QuotaInfo,
  HealthResponse,
  IGateway,
  ApiFormat,
  PlanTier,
  HealthStatus,
} from '@agent-gateway/core';

// Re-export core types
export * from '@agent-gateway/core';

// Import enums for use in mock
import { ApiFormat as ApiFormatEnum, PlanTier as PlanTierEnum, HealthStatus as HealthStatusEnum } from '@agent-gateway/core';

// Try to load native addon, fallback to mock for development
let Gateway: IGateway | null = null;

function loadNativeAddon(): IGateway | null {
  try {
    // Try loading the native addon
    // The addon will be built to target/platform specific paths
    const path = require('path');
    const platform = process.platform;
    const arch = process.arch;
    const bindingPath = path.join(
      __dirname,
      '..',
      '..',
      'prebuilds',
      `agw-napi.${platform}-${arch}.node`
    );

    // In production, the addon is loaded from prebuilds
    // For development, we try to load from various locations
    try {
      const addon = require(bindingPath);
      return createGatewayFromAddon(addon);
    } catch {
      // Try alternative paths
      try {
        const altPath = path.join(__dirname, `../native/agw-napi.${platform}-${arch}.node`);
        const addon = require(altPath);
        return createGatewayFromAddon(addon);
      } catch {
        // Fallback: try direct require of the package
        try {
          const addon = require('@agent-gateway/node-win32-x64');
          return createGatewayFromAddon(addon);
        } catch {
          return null;
        }
      }
    }
  } catch {
    return null;
  }
}

function createGatewayFromAddon(addon: any): IGateway {
  // The native addon exports a Gateway class
  const nativeGateway = new addon.Gateway();

  return {
    async listProviders() {
      return nativeGateway.listProviders();
    },

    async getProvider(providerId: string) {
      return nativeGateway.getProvider(providerId);
    },

    async listPlans() {
      return nativeGateway.listPlans();
    },

    async getPlan(id: string) {
      return nativeGateway.getPlan(id);
    },

    async createPlan(input: CreatePlanInput) {
      return nativeGateway.createPlan(input);
    },

    async updatePlan(id: string, input: UpdatePlanInput) {
      return nativeGateway.updatePlan(id, input);
    },

    async deletePlan(id: string) {
      return nativeGateway.deletePlan(id);
    },

    async testPlan(id: string): Promise<TestConnectionResult> {
      return nativeGateway.testPlan(id);
    },

    async getFallbackConfig() {
      return nativeGateway.getFallbackConfig();
    },

    async setFallbackEnabled(enabled: boolean) {
      return nativeGateway.setFallbackEnabled(enabled);
    },

    async setFallbackPriority(priorityOrder: string[]) {
      return nativeGateway.setFallbackPriority(priorityOrder);
    },

    async getQuotaUsage(planId: string) {
      return nativeGateway.getQuotaUsage(planId);
    },

    async setQuotaLimits(planId: string, daily?: number, monthly?: number, rpm?: number) {
      return nativeGateway.setQuotaLimits(planId, daily, monthly, rpm);
    },

    validateApiKey(content: string): boolean {
      return nativeGateway.validateApiKey(content);
    },

    maskApiKey(apiKey: string): string {
      return nativeGateway.maskApiKey(apiKey);
    },

    health(): HealthResponse {
      return nativeGateway.health();
    },
  };
}

// Create mock gateway for when native addon is not available
function createMockGateway(): IGateway {
  const mockPlans: Map<string, PlanInfo> = new Map();

  return {
    async listProviders(): Promise<ProviderInfo[]> {
      return [
        {
          providerId: 'alaya',
          name: 'Alaya',
          description: 'Alaya AI Coding Platform',
          homepage: 'https://alaya.com',
          docs_url: 'https://docs.alaya.com',
          api_format: ApiFormatEnum.Anthropic,
          requires_api_key: true,
          onboarding: {
            description: 'Get started with Alaya',
            signup_url: 'https://alaya.com/signup',
            agent_setup_guides: [],
          },
          coding_plans: [
            {
              plan_id: 'lite',
              name: 'Lite',
              description: 'Basic plan',
              tier: PlanTierEnum.Free,
              supported_model_ids: ['glm-5'],
              supported_agent_ids: ['claude-code'],
              default_model_id: 'glm-5',
              default_agent_id: 'claude-code',
              features: [],
            },
          ],
          models: [],
          supported_agents: [],
          version: '1.0.0',
        },
      ];
    },

    async getProvider(providerId: string): Promise<ProviderInfo | null> {
      const providers = await this.listProviders();
      return providers.find(p => p.providerId === providerId) || null;
    },

    async listPlans(): Promise<PlanInfo[]> {
      return Array.from(mockPlans.values());
    },

    async getPlan(id: string): Promise<PlanInfo | null> {
      return mockPlans.get(id) || null;
    },

    async createPlan(input: CreatePlanInput): Promise<PlanInfo> {
      const plan: PlanInfo = {
        id: `${input.providerId}-${Date.now()}`,
        providerId: input.providerId,
        plan_id: input.plan_id,
        name: input.name,
        api_key_masked: input.api_key.length > 8
          ? `${input.api_key.slice(0, 4)}...${input.api_key.slice(-4)}`
          : '****',
        selected_model_id: input.selected_model_id,
        bound_agents: [],
        enabled: true,
        priority: 0,
        created_at: new Date().toISOString(),
        health_status: HealthStatusEnum.Unknown,
      };
      mockPlans.set(plan.id, plan);
      return plan;
    },

    async updatePlan(id: string, input: UpdatePlanInput): Promise<PlanInfo> {
      const plan = mockPlans.get(id);
      if (!plan) throw new Error(`Plan not found: ${id}`);

      const updated = { ...plan, ...input };
      mockPlans.set(id, updated);
      return updated;
    },

    async deletePlan(id: string): Promise<void> {
      mockPlans.delete(id);
    },

    async testPlan(id: string): Promise<TestConnectionResult> {
      return {
        plan_id: id,
        success: true,
        message: 'Mock connection successful',
      };
    },

    async getFallbackConfig(): Promise<FallbackConfigInfo> {
      return {
        enabled: true,
        max_attempts: 3,
        priority_order: [],
      };
    },

    async setFallbackEnabled(enabled: boolean): Promise<FallbackConfigInfo> {
      return { enabled, max_attempts: 3, priority_order: [] };
    },

    async setFallbackPriority(priorityOrder: string[]): Promise<FallbackConfigInfo> {
      return { enabled: true, max_attempts: 3, priority_order: priorityOrder };
    },

    async getQuotaUsage(planId: string): Promise<QuotaInfo | null> {
      return {
        plan_id: planId,
        usage: { plan_id: planId, daily_used: 0, monthly_used: 0, rpm_used: 0 },
        limits: { daily: 1000, monthly: 30000, rpm: 60 },
      };
    },

    async setQuotaLimits(planId: string, daily?: number, monthly?: number, rpm?: number): Promise<void> {
      // Mock implementation
    },

    validateApiKey(content: string): boolean {
      const prefixes = ['sk-', 'sk-ant-', 'AIza', 'gsk_', 'kilo_'];
      return prefixes.some(prefix => content.startsWith(prefix));
    },

    maskApiKey(apiKey: string): string {
      if (apiKey.length <= 8) return '****';
      return `${apiKey.slice(0, 4)}...${apiKey.slice(-4)}`;
    },

    health(): HealthResponse {
      return { status: 'ok', version: '0.1.0' };
    },
  };
}

// Initialize gateway
Gateway = loadNativeAddon() || createMockGateway();

/**
 * Get the agent-gateway instance
 * Automatically loads native addon if available, falls back to mock
 */
export function getGateway(): IGateway {
  if (!Gateway) {
    Gateway = createMockGateway();
  }
  return Gateway;
}

/**
 * Check if native bindings are available
 */
export function hasNativeBindings(): boolean {
  return loadNativeAddon() !== null;
}

/**
 * Create a new gateway instance
 * For native bindings, this creates a new instance
 * For mock, this returns the same mock gateway
 */
export function createGateway(): IGateway {
  if (loadNativeAddon()) {
    return getGateway();
  }
  return createMockGateway();
}

// Default export
export default {
  getGateway,
  createGateway,
  hasNativeBindings,
};