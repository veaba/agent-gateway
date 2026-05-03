import { ref } from 'vue'
import type { Agent, UserPlan, CustomAgent } from '@/types'
import {
  fetchAgents,
  bindAgent,
  unbindAgent,
  autoConfigAgent,
  fetchPlans,
  fetchCustomAgents,
  createCustomAgent,
  updateCustomAgent,
  deleteCustomAgent
} from '@/api'

export function useAgents() {
  const agents = ref<Agent[]>([])
  const customAgents = ref<CustomAgent[]>([])
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

  const loadCustomAgents = async () => {
    try {
      customAgents.value = await fetchCustomAgents()
    } catch {
      error.value = '加载自定义 Agent 失败'
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

  // Custom Agent CRUD
  const createCustom = async (agent: Partial<CustomAgent>) => {
    try {
      const newAgent = await createCustomAgent(agent)
      await loadCustomAgents()
      return newAgent
    } catch {
      error.value = '创建自定义 Agent 失败'
      return null
    }
  }

  const updateCustom = async (id: string, updates: Partial<CustomAgent>) => {
    try {
      const updated = await updateCustomAgent(id, updates)
      await loadCustomAgents()
      return updated
    } catch {
      error.value = '更新自定义 Agent 失败'
      return null
    }
  }

  const deleteCustom = async (id: string) => {
    try {
      await deleteCustomAgent(id)
      await loadCustomAgents()
      return true
    } catch {
      error.value = '删除自定义 Agent 失败'
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

  // Get combined list of all agents (built-in + custom)
  const getAllAgents = () => {
    const customAsAgents: Agent[] = customAgents.value.map(ca => ({
      agent_id: ca.agentId,
      name: ca.name,
      description: ca.description,
      logo_url: ca.logoUrl,
      homepage: '',
      install_url: '',
      supported_formats: [],
      config_methods: [],
      isCustom: true,
      version: ca.version,
      customId: ca.id,
    } as any))
    return [...agents.value, ...customAsAgents]
  }

  return {
    agents,
    customAgents,
    plans,
    isLoading,
    error,
    loadAgents,
    loadCustomAgents,
    loadPlans,
    bind,
    unbind,
    autoConfig,
    createCustom,
    updateCustom,
    deleteCustom,
    getAgentBinding,
    getPlansWithAgent,
    getAllAgents,
  }
}