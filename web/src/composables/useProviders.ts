import { ref } from 'vue'
import type { Provider } from '@/types'
import { fetchProviders, fetchProvider } from '@/api'

export function useProviders() {
  const providers = ref<Provider[]>([])
  const isLoading = ref(false)
  const error = ref<string | null>(null)

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

  const getProvider = async (id: string) => {
    try {
      return await fetchProvider(id)
    } catch (e) {
      error.value = '加载服务商详情失败'
      throw e
    }
  }

  return {
    providers,
    isLoading,
    error,
    loadProviders,
    getProvider
  }
}