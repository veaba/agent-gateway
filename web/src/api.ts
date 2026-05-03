import axios from "axios";
import type {
  Provider,
  UserPlan,
  FallbackConfig,
  QuotaStatus,
  RequestLog,
  Plugin,
  GlobalStats,
  ProviderStats,
  PlanStats,
  UsageTrend,
  FallbackEvent,
  FallbackStats,
  ProviderPerformance,
  Agent,
  CustomAgent,
  CustomProvider,
} from "./types";

const api = axios.create({
  baseURL: "/api/v1",
  timeout: 30000,
});

// Provider APIs
export async function fetchProviders(): Promise<Provider[]> {
  const { data } = await api.get("/providers");
  return data.data.providers;
}

export async function fetchProvider(providerId: string): Promise<Provider> {
  const { data } = await api.get(`/providers/${providerId}`);
  return data.data;
}

export async function updateProviders(): Promise<Provider[]> {
  const { data } = await api.post("/providers/update");
  return data.data.providers;
}

// Plan APIs
export async function fetchPlans(): Promise<UserPlan[]> {
  const { data } = await api.get("/plans");
  return data.data.plans;
}

export async function createPlan(plan: Partial<UserPlan>): Promise<UserPlan> {
  const { data } = await api.post("/plans", plan);
  return data.data;
}

export async function updatePlan(id: string, plan: Partial<UserPlan>): Promise<UserPlan> {
  const { data } = await api.put(`/plans/${id}`, plan);
  return data.data;
}

export async function deletePlan(id: string): Promise<void> {
  await api.delete(`/plans/${id}`);
}

export async function testPlan(id: string): Promise<boolean> {
  const { data } = await api.post(`/plans/${id}/test`);
  return data.data.success;
}

// Fallback APIs
export async function fetchFallbackConfig(): Promise<FallbackConfig> {
  const { data } = await api.get("/fallback");
  return data.data;
}

export async function updateFallbackConfig(config: FallbackConfig): Promise<FallbackConfig> {
  const { data } = await api.put("/fallback", config);
  return data.data;
}

// Quota APIs
export async function fetchQuotaStatus(planId?: string): Promise<QuotaStatus[]> {
  const { data } = await api.get("/quota", { params: { plan_id: planId } });
  return data.data.quotas;
}

// Log APIs
export async function fetchLogs(limit: number = 100): Promise<RequestLog[]> {
  console.log("Fetching logs with limit:", limit);
  const { data } = await api.get("/logs", { params: { limit } });
  return data.data.logs;
}

// Plugin APIs
export async function fetchPlugins(): Promise<Plugin[]> {
  const { data } = await api.get("/plugins");
  return data.data.plugins;
}

export async function installPlugin(source: string): Promise<Plugin> {
  const { data } = await api.post("/plugins/install", { source });
  return data.data;
}

export async function updatePlugin(id: string, source?: string): Promise<Plugin> {
  const { data } = await api.post(`/plugins/${id}/update`, { source });
  return data.data;
}

export async function uninstallPlugin(id: string): Promise<void> {
  await api.delete(`/plugins/${id}`);
}

export async function enablePlugin(id: string): Promise<void> {
  await api.post(`/plugins/${id}/enable`);
}

export async function disablePlugin(id: string): Promise<void> {
  await api.post(`/plugins/${id}/disable`);
}

// Agent APIs
export async function fetchAgents(): Promise<Agent[]> {
  const { data } = await api.get("/agents");
  return data.data.agents;
}

export async function bindAgent(planId: string, agentId: string, autoConfig: boolean = false): Promise<void> {
  await api.post(`/plans/${planId}/agents/${agentId}/bind`, { auto_config: autoConfig });
}

export async function unbindAgent(planId: string, agentId: string): Promise<void> {
  await api.delete(`/plans/${planId}/agents/${agentId}/unbind`);
}

export async function autoConfigAgent(planId: string, agentId: string): Promise<void> {
  await api.post(`/plans/${planId}/agents/${agentId}/auto-config`);
}

// Custom Agent APIs
export async function fetchCustomAgents(): Promise<CustomAgent[]> {
  const { data } = await api.get("/custom-agents");
  return data.data.customAgents;
}

export async function createCustomAgent(agent: Partial<CustomAgent>): Promise<CustomAgent> {
  const { data } = await api.post("/custom-agents", agent);
  return data.data;
}

export async function updateCustomAgent(id: string, agent: Partial<CustomAgent>): Promise<CustomAgent> {
  const { data } = await api.put(`/custom-agents/${id}`, agent);
  return data.data;
}

export async function deleteCustomAgent(id: string): Promise<void> {
  await api.delete(`/custom-agents/${id}`);
}

// Custom Provider APIs
export async function fetchCustomProviders(): Promise<CustomProvider[]> {
  const { data } = await api.get("/custom-providers");
  return data.data.customProviders;
}

export async function createCustomProvider(provider: Partial<CustomProvider>): Promise<CustomProvider> {
  const { data } = await api.post("/custom-providers", provider);
  return data.data;
}

export async function updateCustomProvider(id: string, provider: Partial<CustomProvider>): Promise<CustomProvider> {
  const { data } = await api.put(`/custom-providers/${id}`, provider);
  return data.data;
}

export async function deleteCustomProvider(id: string): Promise<void> {
  await api.delete(`/custom-providers/${id}`);
}

// Stats APIs
export async function fetchGlobalStats(): Promise<GlobalStats> {
  const { data } = await api.get("/stats");
  return data.data;
}

export async function fetchProviderStats(): Promise<ProviderStats[]> {
  const { data } = await api.get("/stats/providers");
  return data.data;
}

export async function fetchUsageTrend(granularity: string = "hour"): Promise<UsageTrend> {
  const { data } = await api.get("/stats/usage", { params: { granularity } });
  return data.data;
}

export async function fetchPlanStats(planId: string): Promise<PlanStats> {
  const { data } = await api.get(`/stats/${planId}`);
  return data.data;
}

// Fallback Event APIs
export async function fetchFallbackEvents(
  planId?: string,
  providerId?: string,
  limit: number = 100,
): Promise<FallbackEvent[]> {
  const { data } = await api.get("/fallback/events", {
    params: { plan_id: planId, providerId: providerId, limit },
  });
  return data.data;
}

export async function fetchFallbackStats(): Promise<FallbackStats> {
  const { data } = await api.get("/fallback/stats");
  return data.data;
}

export async function fetchFallbackPerformance(): Promise<ProviderPerformance[]> {
  const { data } = await api.get("/fallback/performance");
  return data.data;
}

// Health check (直接访问 /health，不走 /api/v1)
export async function healthCheck(): Promise<boolean> {
  try {
    const { data } = await axios.get("/health");
    return data.status === "ok";
  } catch {
    return false;
  }
}

export default api;
