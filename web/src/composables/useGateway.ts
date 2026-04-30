import { ref } from 'vue'
import api, { healthCheck } from '@/api'

export function useGateway() {
  const isConnected = ref(false)
  const isLoading = ref(false)

  const checkConnection = async () => {
    isLoading.value = true
    try {
      isConnected.value = await healthCheck()
    } finally {
      isLoading.value = false
    }
  }

  const startGateway = async () => {
    try {
      await api.post('/gateway/start')
      isConnected.value = true
    } catch {
      isConnected.value = false
    }
  }

  const stopGateway = async () => {
    try {
      await api.post('/gateway/stop')
      isConnected.value = false
    } catch {
      // ignore
    }
  }

  return {
    isConnected,
    isLoading,
    checkConnection,
    startGateway,
    stopGateway
  }
}