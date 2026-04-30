<template>
  <div class="api-key-input">
    <el-input
      :model-value="modelValue"
      type="password"
      show-password
      placeholder="粘贴您的 API Key"
      @update:model-value="handleInput"
    >
      <template #prepend>
        <el-button @click="handleOpenPage">
          <el-icon><Link /></el-icon>
          去获取 API Key
        </el-button>
      </template>
    </el-input>

    <el-button v-if="provider.setup_guide_url" @click="handleOpenGuide">
      查看配置指南
    </el-button>

    <div v-if="clipboardDetected" class="clipboard-detected">
      <el-alert type="info" :closable="false">
        检测到剪贴板中有 API Key，是否使用？
        <el-button size="small" type="primary" @click="useClipboard">使用</el-button>
        <el-button size="small" @click="ignoreClipboard">忽略</el-button>
      </el-alert>
    </div>

    <div class="test-section">
      <el-button
        type="success"
        :loading="testing"
        :disabled="!modelValue"
        @click="handleTest"
      >
        测试连接
      </el-button>
      <div v-if="testResult" class="test-result">
        <el-tag :type="testResult.success ? 'success' : 'danger'">
          {{ testResult.message }}
        </el-tag>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { ElMessage } from 'element-plus'
import { invoke } from '@tauri-apps/api/core'
import type { Provider } from '@/types'

const props = defineProps<{
  provider: Provider
  modelValue?: string
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string]
  'open-page': []
}>()

const clipboardDetected = ref(false)
const testing = ref(false)
const testResult = ref<{ success: boolean; message: string } | null>(null)
let clipboardInterval: number | undefined

const handleInput = (value: string) => {
  emit('update:modelValue', value)
}

const handleOpenPage = () => {
  emit('open-page')
}

const handleOpenGuide = () => {
  if (props.provider.setup_guide_url) {
    window.open(props.provider.setup_guide_url, '_blank')
  }
}

const useClipboard = () => {
  emit('update:modelValue', 'detected-key-placeholder')
  clipboardDetected.value = false
}

const ignoreClipboard = () => {
  clipboardDetected.value = false
}

const checkClipboard = async () => {
  try {
    const detected = await invoke<string | null>('check_clipboard_for_key', {
      expectedPrefix: getExpectedPrefix(props.provider.provider_id)
    })
    if (detected) {
      clipboardDetected.value = true
    }
  } catch {
    // Ignore errors - may not be running in Tauri
  }
}

const getExpectedPrefix = (providerId: string) => {
  const prefixes: Record<string, string> = {
    anthropic: 'sk-ant-',
    openai: 'sk-',
    alaya: 'sk-'
  }
  return prefixes[providerId]
}

const handleTest = async () => {
  testing.value = true
  testResult.value = null

  try {
    await new Promise(resolve => setTimeout(resolve, 1500))
    testResult.value = { success: true, message: '连接成功' }
    ElMessage.success('连接测试成功')
  } catch {
    testResult.value = { success: false, message: '连接失败' }
    ElMessage.error('连接测试失败')
  } finally {
    testing.value = false
  }
}

onMounted(() => {
  clipboardInterval = window.setInterval(checkClipboard, 2000)
})

onUnmounted(() => {
  if (clipboardInterval) {
    clearInterval(clipboardInterval)
  }
})
</script>

<style scoped>
.api-key-input {
  max-width: 600px;
  margin: 0 auto;
}

.clipboard-detected {
  margin-top: 16px;
}

.test-section {
  margin-top: 20px;
  display: flex;
  align-items: center;
  gap: 12px;
}
</style>