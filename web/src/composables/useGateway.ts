import { ref } from 'vue'
import api, { healthCheck } from '@/api'

const isEmbedded = ref(false)

export function useGateway() {
  const isConnected = ref(false)
  const isLoading = ref(false)
  const gatewayAddr = ref<string | null>(null)

  const checkConnection = async () => {
    isLoading.value = true
    try {
      isConnected.value = await healthCheck()
    } finally {
      isLoading.value = false
    }
  }

  const startGateway = async (listen?: string) => {
    isLoading.value = true
    try {
      if (window.__TAURI__) {
        const { invoke } = await import('@tauri-apps/api/core')
        const addr = await invoke<string>('start_gateway', { listen: listen || null })
        gatewayAddr.value = addr
        isEmbedded.value = true
        isConnected.value = true
      } else {
        await api.post('/gateway/start')
        isConnected.value = true
      }
    } catch (e) {
      isConnected.value = false
    } finally {
      isLoading.value = false
    }
  }

  const stopGateway = async () => {
    isLoading.value = true
    try {
      if (isEmbedded.value && window.__TAURI__) {
        const { invoke } = await import('@tauri-apps/api/core')
        await invoke('stop_gateway')
        gatewayAddr.value = null
        isEmbedded.value = false
      } else {
        await api.post('/gateway/stop')
      }
      isConnected.value = false
    } catch {
      // ignore
    } finally {
      isLoading.value = false
    }
  }

  const getGatewayStatus = async () => {
    if (window.__TAURI__) {
      try {
        const { invoke } = await import('@tauri-apps/api/core')
        const status = await invoke<{ running: boolean; listen_addr: string | null }>('get_gateway_status')
        isConnected.value = status.running
        gatewayAddr.value = status.listen_addr
        isEmbedded.value = status.running
        return status
      } catch {
        // fallback to HTTP
      }
    }
    return { running: isConnected.value, listen_addr: null }
  }

  return {
    isConnected,
    isLoading,
    isEmbedded,
    gatewayAddr,
    checkConnection,
    startGateway,
    stopGateway,
    getGatewayStatus
  }
}

declare global {
  interface Window {
    __TAURI__?: unknown
  }
}