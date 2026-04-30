import { ref } from 'vue'
import type { QuotaStatus } from '@/types'
import { fetchQuotaStatus } from '@/api'

export function useQuota() {
  const quotas = ref<QuotaStatus[]>([])
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  const loadQuotas = async (planId?: string) => {
    isLoading.value = true
    error.value = null
    try {
      quotas.value = await fetchQuotaStatus(planId)
    } catch (e) {
      error.value = '加载配额失败'
    } finally {
      isLoading.value = false
    }
  }

  const resetQuota = async (planId: string) => {
    try {
      await fetchQuotaStatus(planId)
      await loadQuotas()
    } catch (e) {
      error.value = '重置配额失败'
      throw e
    }
  }

  const getPercent = (used: number, limit: number) => {
    if (!limit) return 0
    return Math.floor((used / limit) * 100)
  }

  return {
    quotas,
    isLoading,
    error,
    loadQuotas,
    resetQuota,
    getPercent
  }
}