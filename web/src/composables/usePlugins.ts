import { ref } from 'vue'
import type { Plugin } from '@/types'
import { fetchPlugins, installPlugin, uninstallPlugin, enablePlugin, disablePlugin } from '@/api'

export interface PluginInfo {
  id: string
  name: string
  version: string
  author: string
  description: string
  status: 'enabled' | 'disabled' | 'error'
}

export function usePlugins() {
  const plugins = ref<PluginInfo[]>([])
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  const loadPlugins = async () => {
    isLoading.value = true
    error.value = null
    try {
      plugins.value = await fetchPlugins()
    } catch (e) {
      error.value = '加载插件失败'
    } finally {
      isLoading.value = false
    }
  }

  const install = async (source: string) => {
    try {
      const plugin = await installPlugin(source)
      plugins.value.push(plugin)
      return plugin
    } catch (e) {
      error.value = '安装插件失败'
      throw e
    }
  }

  const uninstall = async (id: string) => {
    try {
      await uninstallPlugin(id)
      plugins.value = plugins.value.filter(p => p.id !== id)
    } catch (e) {
      error.value = '卸载插件失败'
      throw e
    }
  }

  const enable = async (id: string) => {
    try {
      await enablePlugin(id)
      const plugin = plugins.value.find(p => p.id === id)
      if (plugin) plugin.status = 'enabled'
    } catch (e) {
      error.value = '启用插件失败'
      throw e
    }
  }

  const disable = async (id: string) => {
    try {
      await disablePlugin(id)
      const plugin = plugins.value.find(p => p.id === id)
      if (plugin) plugin.status = 'disabled'
    } catch (e) {
      error.value = '禁用插件失败'
      throw e
    }
  }

  return {
    plugins,
    isLoading,
    error,
    loadPlugins,
    install,
    uninstall,
    enable,
    disable
  }
}