<template>
  <div class="settings-view">
    <el-card class="settings-card">
      <template #header>
        <div class="card-header">
          <span class="card-title">网关服务</span>
          <el-tag :type="gatewayRunning ? 'success' : 'danger'" size="small" round>
            {{ gatewayRunning ? '运行中' : '已停止' }}
          </el-tag>
        </div>
      </template>

      <el-form label-position="top" class="settings-form">
        <el-form-item label="监听地址">
          <el-input v-model="gatewaySettings.listen_address" placeholder="127.0.0.1:8080" clearable :disabled="gatewayRunning">
            <template #prefix>
              <el-icon><Monitor /></el-icon>
            </template>
          </el-input>
          <div class="form-tip">网关服务监听地址，修改后需重启生效</div>
        </el-form-item>

        <el-form-item>
          <el-button v-if="!gatewayRunning" type="primary" :loading="gatewayLoading" @click="handleStartGateway">
            <el-icon><VideoPlay /></el-icon>
            启动网关
          </el-button>
          <el-button v-else type="danger" :loading="gatewayLoading" @click="handleStopGateway">
            <el-icon><VideoPause /></el-icon>
            停止网关
          </el-button>
        </el-form-item>
      </el-form>
    </el-card>

    <el-card class="settings-card">
      <template #header>
        <div class="card-header">
          <span class="card-title">基本设置</span>
        </div>
      </template>

      <el-form label-position="top" class="settings-form">
        <el-form-item label="日志级别">
          <el-select v-model="settings.log_level" placeholder="选择日志级别" style="width: 100%">
            <el-option label="Debug" value="debug" />
            <el-option label="Info" value="info" />
            <el-option label="Warning" value="warn" />
            <el-option label="Error" value="error" />
          </el-select>
        </el-form-item>

        <el-form-item label="主题模式">
          <el-select v-model="settings.theme" placeholder="选择主题" style="width: 100%">
            <el-option label="深色 (Dark)" value="dark" />
            <el-option label="浅色 (Light)" value="light" />
            <el-option label="跟随系统" value="auto" />
          </el-select>
        </el-form-item>

        <el-divider />

        <el-form-item>
          <el-button type="primary" :loading="saving" @click="handleSave">
            <el-icon><Check /></el-icon>
            保存设置
          </el-button>
          <el-button @click="loadSettings">
            <el-icon><Refresh /></el-icon>
            重置
          </el-button>
        </el-form-item>
      </el-form>
    </el-card>

    <el-card class="settings-card">
      <template #header>
        <div class="card-header">
          <span class="card-title">关于</span>
        </div>
      </template>

      <div class="about-grid">
        <div class="about-item">
          <span class="about-label">版本</span>
          <span class="about-value agw-mono">0.1.0</span>
        </div>
        <div class="about-item">
          <span class="about-label">构建时间</span>
          <span class="about-value">{{ buildTime }}</span>
        </div>
        <div class="about-item">
          <span class="about-label">前端框架</span>
          <span class="about-value">Vue 3 + Element Plus</span>
        </div>
        <div class="about-item">
          <span class="about-label">后端框架</span>
          <span class="about-value">Rust + Axum</span>
        </div>
        <div class="about-item">
          <span class="about-label">服务状态</span>
          <span class="about-value">
            <el-tag :type="isConnected ? 'success' : 'danger'" size="small" round>
              {{ isConnected ? '在线' : '离线' }}
            </el-tag>
          </span>
        </div>
        <div class="about-item">
          <span class="about-label">API 端点</span>
          <span class="about-value agw-mono">{{ gatewaySettings.listen_address }}</span>
        </div>
      </div>

      <el-divider />

      <div class="about-footer">
        <p class="about-desc">
          Agent Gateway 是一个统一的 AI 代理网关，支持多种 AI 服务提供商的套餐管理、自动故障转移和配额控制。
        </p>
        <div class="about-links">
          <el-button link type="primary" @click="openUrl('https://github.com')">
            <el-icon><Link /></el-icon>
            GitHub
          </el-button>
          <el-button link type="primary" @click="openUrl('https://docs.anthropic.com')">
            <el-icon><Document /></el-icon>
            文档
          </el-button>
        </div>
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { ElMessage } from 'element-plus'
import { healthCheck } from '@/api'
import { useGateway } from '@/composables/useGateway'

const saving = ref(false)
const isConnected = ref(false)
const gatewayRunning = ref(false)
const gatewayLoading = ref(false)
const buildTime = ref('2024-01-15')

const { startGateway: doStartGateway, stopGateway: doStopGateway, getGatewayStatus } = useGateway()

const settings = reactive({
  log_level: 'info',
  theme: 'dark'
})

const gatewaySettings = reactive({
  listen_address: '127.0.0.1:8080'
})

const loadSettings = () => {
  settings.log_level = 'info'
  settings.theme = 'dark'
}

const handleSave = async () => {
  saving.value = true
  try {
    await new Promise(resolve => setTimeout(resolve, 500))
    localStorage.setItem('agw-settings', JSON.stringify(settings))
    ElMessage.success('设置已保存')
  } catch {
    ElMessage.error('保存失败')
  } finally {
    saving.value = false
  }
}

const loadSettingsFromStorage = () => {
  try {
    const saved = localStorage.getItem('agw-settings')
    if (saved) {
      const parsed = JSON.parse(saved)
      Object.assign(settings, parsed)
    }
  } catch {
    // Ignore
  }
}

const handleStartGateway = async () => {
  gatewayLoading.value = true
  try {
    await doStartGateway(gatewaySettings.listen_address)
    gatewayRunning.value = true
    ElMessage.success('网关已启动')
  } catch {
    ElMessage.error('网关启动失败')
  } finally {
    gatewayLoading.value = false
  }
}

const handleStopGateway = async () => {
  gatewayLoading.value = true
  try {
    await doStopGateway()
    gatewayRunning.value = false
    ElMessage.success('网关已停止')
  } catch {
    ElMessage.error('网关停止失败')
  } finally {
    gatewayLoading.value = false
  }
}

const checkConnection = async () => {
  isConnected.value = await healthCheck()
}

const openUrl = (url: string) => {
  window.open(url, '_blank')
}

onMounted(async () => {
  loadSettingsFromStorage()
  await checkConnection()
  const status = await getGatewayStatus()
  gatewayRunning.value = status.running
  if (status.listen_addr) {
    gatewaySettings.listen_address = status.listen_addr
  }
})
</script>

<style scoped>
.settings-view {
  display: flex;
  flex-direction: column;
  gap: 20px;
  max-width: 640px;
  animation: fadeIn 0.5s ease;
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(10px); }
  to { opacity: 1; transform: translateY(0); }
}

.settings-card {
  background: rgba(20, 23, 34, 0.7);
  backdrop-filter: blur(20px);
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 14px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.card-title {
  font-size: 15px;
  font-weight: 600;
  color: #e8eaf0;
}

.settings-form {
  padding: 8px 0;
}

.form-tip {
  font-size: 12px;
  color: #6b7280;
  margin-top: 4px;
}

.about-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 16px;
}

.about-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.about-label {
  font-size: 12px;
  color: #6b7280;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.about-value {
  font-size: 14px;
  color: #e8eaf0;
}

.about-footer {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.about-desc {
  font-size: 13px;
  color: #94a3b8;
  margin: 0;
  line-height: 1.6;
}

.about-links {
  display: flex;
  gap: 16px;
}
</style>