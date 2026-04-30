import { ref } from 'vue'

export function useClipboardMonitor() {
  const detectedKey = ref<string>('')
  let interval: ReturnType<typeof setInterval> | null = null

  const start = async () => {
    if (!window.__TAURI__) return

    interval = setInterval(async () => {
      try {
        const { invoke } = await import('@tauri-apps/api/core')
        const result = await invoke<string | null>('check_clipboard_for_key', {
          expectedPrefix: null
        })
        if (result) {
          detectedKey.value = result
        }
      } catch {
        // ignore clipboard errors
      }
    }, 2000)
  }

  const stop = () => {
    if (interval !== null) {
      clearInterval(interval)
      interval = null
    }
  }

  const clear = () => {
    detectedKey.value = ''
  }

  return {
    detectedKey,
    start,
    stop,
    clear
  }
}

declare global {
  interface Window {
    __TAURI__?: unknown
  }
}