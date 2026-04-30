import axios from 'axios'
import type { Provider, UserPlan, FallbackConfig, QuotaStatus, RequestLog, Plugin } from './types'

const api = axios.create({
  baseURL: '/api/v1',
  timeout: 30000
})

// Provider APIs
export async function fetchProviders(): Promise<Provider[]> {
  const { data } = await api.get('/providers')
  return data.providers
}

export async function fetchProvider(providerId: string): Promise<Provider> {
  const { data } = await api.get(`/providers/${providerId}`)
  return data
}

// Plan APIs
export async function fetchPlans(): Promise<UserPlan[]> {
  const { data } = await api.get('/plans')
  return data.plans
}

export async function createPlan(plan: Partial<UserPlan>): Promise<UserPlan> {
  const { data } = await api.post('/plans', plan)
  return data
}

export async function updatePlan(id: string, plan: Partial<UserPlan>): Promise<UserPlan> {
  const { data } = await api.put(`/plans/${id}`, plan)
  return data
}

export async function deletePlan(id: string): Promise<void> {
  await api.delete(`/plans/${id}`)
}

export async function testPlan(id: string): Promise<boolean> {
  const { data } = await api.post(`/plans/${id}/test`)
  return data.success
}

// Fallback APIs
export async function fetchFallbackConfig(): Promise<FallbackConfig> {
  const { data } = await api.get('/fallback')
  return data
}

export async function updateFallbackConfig(config: FallbackConfig): Promise<FallbackConfig> {
  const { data } = await api.put('/fallback', config)
  return data
}

// Quota APIs
export async function fetchQuotaStatus(planId?: string): Promise<QuotaStatus[]> {
  const { data } = await api.get('/quota', { params: { plan_id: planId } })
  return data.quotas
}

// Log APIs
export async function fetchLogs(limit: number = 100): Promise<RequestLog[]> {
  const { data } = await api.get('/logs', { params: { limit } })
  return data.logs
}

// Plugin APIs
export async function fetchPlugins(): Promise<Plugin[]> {
  const { data } = await api.get('/plugins')
  return data.plugins
}

export async function installPlugin(source: string): Promise<Plugin> {
  const { data } = await api.post('/plugins/install', { source })
  return data
}

export async function uninstallPlugin(id: string): Promise<void> {
  await api.delete(`/plugins/${id}`)
}

export async function enablePlugin(id: string): Promise<void> {
  await api.post(`/plugins/${id}/enable`)
}

export async function disablePlugin(id: string): Promise<void> {
  await api.post(`/plugins/${id}/disable`)
}

// Health check
export async function healthCheck(): Promise<boolean> {
  try {
    const { data } = await api.get('/health')
    return data.status === 'ok'
  } catch {
    return false
  }
}

export default api