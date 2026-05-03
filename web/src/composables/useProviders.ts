import { ref, computed } from 'vue'
import type { Provider, CustomProvider } from '@/types'
import { fetchProviders, fetchProvider, updateProviders, fetchCustomProviders, createCustomProvider, updateCustomProvider, deleteCustomProvider } from '@/api'

export function useProviders() {
  const providers = ref<Provider[]>([])
  const customProviders = ref<CustomProvider[]>([])
  const isLoading = ref(false)
  const isUpdating = ref(false)
  const error = ref<string | null>(null)

  // 合并内置和自定义 Provider
  const allProviders = computed(() => {
    const builtin = providers.value.map(p => ({ ...p, isCustom: false }))
    const custom = customProviders.value.map(cp => ({
      ...cp,
      // 将 CustomProvider 转换为 Provider 格式
      isCustom: true,
      codingPlans: [],
      supportedAgents: [],
    }))
    return [...builtin, ...custom]
  })

  const loadProviders = async () => {
    isLoading.value = true
    error.value = null
    try {
      providers.value = await fetchProviders()
    } catch (e) {
      error.value = '加载服务商失败'
    } finally {
      isLoading.value = false
    }
  }

  const loadCustomProviders = async () => {
    try {
      customProviders.value = await fetchCustomProviders()
    } catch (e) {
      // 可能还没有 custom_providers.yaml，忽略错误
      customProviders.value = []
    }
  }

  const loadAllProviders = async () => {
    await Promise.all([loadProviders(), loadCustomProviders()])
  }

  const getProvider = async (id: string) => {
    try {
      return await fetchProvider(id)
    } catch (e) {
      error.value = '加载服务商详情失败'
      throw e
    }
  }

  const updateProviderDefs = async () => {
    isUpdating.value = true
    error.value = null
    try {
      providers.value = await updateProviders()
      return true
    } catch (e) {
      error.value = '更新服务商定义失败'
      return false
    } finally {
      isUpdating.value = false
    }
  }

  // Custom Provider CRUD
  const createCustom = async (provider: Partial<CustomProvider>) => {
    try {
      const result = await createCustomProvider(provider)
      await loadCustomProviders()
      return result
    } catch (e) {
      error.value = '创建自定义服务商失败'
      throw e
    }
  }

  const updateCustom = async (id: string, provider: Partial<CustomProvider>) => {
    try {
      const result = await updateCustomProvider(id, provider)
      await loadCustomProviders()
      return result
    } catch (e) {
      error.value = '更新自定义服务商失败'
      throw e
    }
  }

  const deleteCustom = async (id: string) => {
    try {
      await deleteCustomProvider(id)
      await loadCustomProviders()
      return true
    } catch (e) {
      error.value = '删除自定义服务商失败'
      return false
    }
  }

  const getCustomProviderById = (id: string) => {
    return customProviders.value.find(cp => cp.id === id || cp.providerId === id)
  }

  return {
    providers,
    customProviders,
    allProviders,
    isLoading,
    isUpdating,
    error,
    loadProviders,
    loadCustomProviders,
    loadAllProviders,
    getProvider,
    updateProviderDefs,
    createCustom,
    updateCustom,
    deleteCustom,
    getCustomProviderById
  }
}