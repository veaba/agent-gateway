import { ref } from 'vue'
import type { Agent, UserPlan } from '@/types'
import { fetchAgents, bindAgent, unbindAgent, autoConfigAgent, fetchPlans } from '@/api'

export function useAgents() {
  const agents = ref<Agent[]>([])
  const plans = ref<UserPlan[]>([])
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  const loadAgents = async () => {
    isLoading.value = true
    error.value = null
    try {
      agents.value = await fetchAgents()
    } catch {
      error.value = '加载 Agent 工具失败'
    } finally {
      isLoading.value = false
    }
  }

  const loadPlans = async () => {
    try {
      plans.value = await fetchPlans()
    } catch {
      error.value = '加载套餐失败'
    }
  }

  const bind = async (planId: string, agentId: string, autoConfig: boolean = false) => {
    try {
      await bindAgent(planId, agentId, autoConfig)
      await loadPlans() // Refresh plans to show new binding
      return true
    } catch {
      error.value = '绑定 Agent 失败'
      return false
    }
  }

  const unbind = async (planId: string, agentId: string) => {
    try {
      await unbindAgent(planId, agentId)
      await loadPlans() // Refresh plans to show removed binding
      return true
    } catch {
      error.value = '解绑 Agent 失败'
      return false
    }
  }

  const autoConfig = async (planId: string, agentId: string) => {
    try {
      await autoConfigAgent(planId, agentId)
      await loadPlans() // Refresh plans to show updated config status
      return true
    } catch {
      error.value = '自动配置 Agent 失败'
      return false
    }
  }

  // Get binding info for a specific agent from a plan
  const getAgentBinding = (planId: string, agentId: string) => {
    const plan = plans.value.find(p => p.id === planId)
    return plan?.bound_agents?.find(b => b.agent_id === agentId)
  }

  // Get all plans that have this agent bound
  const getPlansWithAgent = (agentId: string) => {
    return plans.value.filter(p =>
      p.bound_agents?.some(b => b.agent_id === agentId)
    )
  }

  return {
    agents,
    plans,
    isLoading,
    error,
    loadAgents,
    loadPlans,
    bind,
    unbind,
    autoConfig,
    getAgentBinding,
    getPlansWithAgent
  }
}