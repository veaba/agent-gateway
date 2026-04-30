import { ref } from 'vue'

export function useClipboardMonitor() {
  const detectedKey = ref<string>('')
  let interval: number | null = null

  const start = async () => {
    // TODO: 实现剪贴板监控
    // interval = window.setInterval(async () => {
    //   const content = await invoke<string>('check_clipboard_for_key')
    //   if (content) {
    //     detectedKey.value = content
    //   }
    // }, 2000)
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