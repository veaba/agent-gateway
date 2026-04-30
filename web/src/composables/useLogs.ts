import { ref } from 'vue'
import type { RequestLog } from '@/types'
import { fetchLogs } from '@/api'

export function useLogs() {
  const logs = ref<RequestLog[]>([])
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  const loadLogs = async (limit = 100) => {
    isLoading.value = true
    error.value = null
    try {
      logs.value = await fetchLogs(limit)
    } catch (e) {
      error.value = '加载日志失败'
    } finally {
      isLoading.value = false
    }
  }

  return {
    logs,
    isLoading,
    error,
    loadLogs
  }
}