<template>
  <div class="onboarding-view">
    <div v-if="!selectedProvider" class="provider-select">
      <h2 class="section-title">选择 Provider 开始配置</h2>
      <p class="section-desc">选择一个 Provider，我们将引导您完成配置</p>
      <div class="provider-grid">
        <div
          v-for="provider in providers"
          :key="provider.provider_id"
          class="provider-card"
          @click="selectProvider(provider)"
        >
          <div class="provider-icon">
            <el-icon :size="32"><Monitor /></el-icon>
          </div>
          <div class="provider-info">
            <h3>{{ provider.name }}</h3>
            <p>{{ provider.description }}</p>
          </div>
          <el-tag v-if="provider.onboarding" type="success" size="small" round>
            有配置引导
          </el-tag>
        </div>
      </div>
    </div>

    <div v-else class="onboarding-guide">
      <div class="guide-header">
        <el-button text @click="selectedProvider = null">
          <el-icon><ArrowLeft /></el-icon>
          返回选择
        </el-button>
        <h2>{{ selectedProvider.name }} 配置引导</h2>
        <p v-if="selectedProvider.onboarding">{{ selectedProvider.onboarding.description }}</p>
      </div>

      <div class="guide-actions">
        <div class="action-row">
          <el-button v-if="selectedProvider.onboarding?.signup_url" type="primary" @click="openUrl(selectedProvider.onboarding.signup_url)">
            <el-icon><Link /></el-icon>
            注册账号
          </el-button>
          <el-button v-if="selectedProvider.onboarding?.get_key_url" @click="openUrl(selectedProvider.onboarding.get_key_url)">
            <el-icon><Key /></el-icon>
            获取 API Key
          </el-button>
          <el-button v-if="selectedProvider.onboarding?.plans_comparison_url" @click="openUrl(selectedProvider.onboarding.plans_comparison_url)">
            <el-icon><DataLine /></el-icon>
            套餐对比
          </el-button>
          <el-button v-if="selectedProvider.onboarding?.setup_guide_url" @click="openUrl(selectedProvider.onboarding.setup_guide_url)">
            <el-icon><Document /></el-icon>
            详细文档
          </el-button>
        </div>
      </div>

      <div v-if="selectedProvider.onboarding?.agent_setup_guides?.length" class="agent-guides">
        <h3 class="guide-subtitle">Agent 工具配置</h3>
        <div
          v-for="guide in selectedProvider.onboarding.agent_setup_guides"
          :key="guide.agent_id"
          class="agent-guide-card"
        >
          <div class="agent-guide-header">
            <h4>{{ guide.agent_name }}</h4>
            <el-tag v-if="guide.auto_config_supported" type="success" size="small">支持自动配置</el-tag>
          </div>

          <div v-if="guide.env_vars.length" class="env-vars">
            <h5>环境变量</h5>
            <div v-for="env in guide.env_vars" :key="env.name" class="env-var-item">
              <code class="env-var-name">{{ env.name }}</code>
              <code class="env-var-value">{{ env.value }}</code>
              <span class="env-var-desc">{{ env.description }}</span>
              <el-button size="small" text @click="copyText(env.name + '=' + env.value)">
                <el-icon><CopyDocument /></el-icon>
              </el-button>
            </div>
          </div>

          <div v-if="guide.manual_steps.length" class="steps">
            <h5>配置步骤</h5>
            <el-steps direction="vertical" :active="guide.manual_steps.length" finish-status="success">
              <el-step
                v-for="step in guide.manual_steps"
                :key="step.step_number"
                :title="step.description"
              >
                <template v-if="step.command || step.copyable_text" #description>
                  <div class="step-detail">
                    <code v-if="step.command" class="step-command">{{ step.command }}</code>
                    <el-button v-if="step.copyable_text" size="small" text @click="copyText(step.copyable_text)">
                      <el-icon><CopyDocument /></el-icon> 复制
                    </el-button>
                    <span v-if="step.note" class="step-note">{{ step.note }}</span>
                  </div>
                </template>
              </el-step>
            </el-steps>
          </div>

          <div v-if="guide.auto_config_supported" class="auto-config-section">
            <el-button
              type="primary"
              :loading="autoConfigLoading === guide.agent_id"
              @click="handleAutoConfig(guide.agent_id)"
            >
              <el-icon><MagicStick /></el-icon>
              一键自动配置 {{ guide.agent_name }}
            </el-button>
          </div>

          <div v-if="autoConfigResult[guide.agent_id]" class="config-result">
            <el-alert
              :title="autoConfigResult[guide.agent_id].success ? '配置成功' : '配置失败'"
              :type="autoConfigResult[guide.agent_id].success ? 'success' : 'error'"
              :description="autoConfigResult[guide.agent_id].message"
              show-icon
              closable
            />
            <div v-if="autoConfigResult[guide.agent_id]?.report" class="report-details">
              <p v-for="path in autoConfigResult[guide.agent_id]?.report?.paths ?? []" :key="path">
                <el-icon><Document /></el-icon> {{ path }}
              </p>
              <p v-if="autoConfigResult[guide.agent_id]?.report?.reload_command">
                <el-icon><Refresh /></el-icon>
                执行以生效: <code>{{ autoConfigResult[guide.agent_id]?.report?.reload_command }}</code>
              </p>
            </div>
          </div>

          <div class="config-paths">
            <h5>配置文件路径</h5>
            <p v-if="guide.config_file_paths.macos">macOS: <code>{{ guide.config_file_paths.macos }}</code></p>
            <p v-if="guide.config_file_paths.linux">Linux: <code>{{ guide.config_file_paths.linux }}</code></p>
            <p v-if="guide.config_file_paths.windows">Windows: <code>{{ guide.config_file_paths.windows }}</code></p>
          </div>
        </div>
      </div>

      <div v-else class="no-guides">
        <el-empty description="该 Provider 暂无配置引导" />
        <p class="help-text">
          您可以参考
          <el-link v-if="selectedProvider.docs_url" type="primary" @click="openUrl(selectedProvider.docs_url)">
            {{ selectedProvider.name }} 文档
          </el-link>
          进行手动配置。
        </p>
      </div>

      <div class="guide-footer">
        <el-button type="primary" @click="$router.push('/plans/add')">
          开始添加套餐
        </el-button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { ElMessage } from 'element-plus'
import { fetchProviders } from '@/api'
import type { Provider } from '@/types'

interface AutoConfigResult {
  success: boolean
  message: string
  report?: {
    agent: string
    method: string
    paths: string[]
    requires_reload: boolean
    reload_command?: string
  }
}

const providers = ref<Provider[]>([])
const selectedProvider = ref<Provider | null>(null)
const autoConfigLoading = ref<string | null>(null)
const autoConfigResult = ref<Record<string, AutoConfigResult>>({})

const selectProvider = (provider: Provider) => {
  selectedProvider.value = provider
  autoConfigResult.value = {}
}

const openUrl = (url: string) => {
  window.open(url, '_blank')
}

const copyText = async (text: string) => {
  try {
    await navigator.clipboard.writeText(text)
    ElMessage.success('已复制到剪贴板')
  } catch {
    ElMessage.warning('复制失败，请手动复制')
  }
}

const handleAutoConfig = async (agentId: string) => {
  autoConfigLoading.value = agentId
  try {
    if (window.__TAURI__) {
      const { invoke } = await import('@tauri-apps/api/core')
      const report = await invoke<{
        agent: string
        method: string
        paths: string[]
        requires_reload: boolean
        reload_command: string | null
      }>('auto_config_agent', { agentId, gatewayAddr: null })
      autoConfigResult.value[agentId] = {
        success: true,
        message: `${report.agent} 配置成功，方法: ${report.method}`,
        report: {
          ...report,
          reload_command: report.reload_command || undefined,
        },
      }
    } else {
      autoConfigResult.value[agentId] = {
        success: false,
        message: '自动配置仅在桌面应用中可用。请按照手动步骤配置。',
      }
    }
  } catch (e: unknown) {
    autoConfigResult.value[agentId] = {
      success: false,
      message: `配置失败: ${e instanceof Error ? e.message : String(e)}`,
    }
  } finally {
    autoConfigLoading.value = null
  }
}

onMounted(async () => {
  try {
    providers.value = await fetchProviders()
  } catch {
    ElMessage.error('加载 Provider 列表失败')
  }
})
</script>

<style scoped>
.onboarding-view {
  max-width: 800px;
  animation: fadeIn 0.5s ease;
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(10px); }
  to { opacity: 1; transform: translateY(0); }
}

.section-title {
  font-size: 20px;
  font-weight: 600;
  color: #e8eaf0;
  margin: 0 0 8px 0;
}

.section-desc {
  color: #94a3b8;
  margin: 0 0 24px 0;
  font-size: 14px;
}

.provider-grid {
  display: grid;
  gap: 16px;
}

.provider-card {
  background: rgba(20, 23, 34, 0.7);
  backdrop-filter: blur(20px);
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 14px;
  padding: 20px;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 16px;
  transition: all 0.2s ease;
}

.provider-card:hover {
  border-color: rgba(34, 211, 238, 0.4);
  box-shadow: 0 0 20px rgba(34, 211, 238, 0.1);
}

.provider-icon {
  color: #22d3ee;
}

.provider-info {
  flex: 1;
}

.provider-info h3 {
  margin: 0 0 4px 0;
  color: #e8eaf0;
  font-size: 16px;
}

.provider-info p {
  margin: 0;
  color: #94a3b8;
  font-size: 13px;
}

.guide-header {
  margin-bottom: 24px;
}

.guide-header h2 {
  font-size: 20px;
  font-weight: 600;
  color: #e8eaf0;
  margin: 8px 0;
}

.guide-header p {
  color: #94a3b8;
  font-size: 14px;
  margin: 4px 0 0;
}

.guide-actions {
  margin-bottom: 24px;
}

.action-row {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
}

.guide-subtitle {
  font-size: 16px;
  font-weight: 600;
  color: #e8eaf0;
  margin: 0 0 16px;
}

.agent-guide-card {
  background: rgba(20, 23, 34, 0.7);
  backdrop-filter: blur(20px);
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 14px;
  padding: 20px;
  margin-bottom: 16px;
}

.agent-guide-header {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 16px;
}

.agent-guide-header h4 {
  margin: 0;
  font-size: 15px;
  color: #e8eaf0;
}

.env-vars {
  margin-bottom: 16px;
}

.env-vars h5 {
  font-size: 13px;
  color: #94a3b8;
  margin: 0 0 8px;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.env-var-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 0;
  font-size: 13px;
}

.env-var-name {
  color: #22d3ee;
  background: rgba(34, 211, 238, 0.1);
  padding: 2px 8px;
  border-radius: 4px;
  font-family: monospace;
}

.env-var-value {
  color: #a5f3fc;
  background: rgba(34, 211, 238, 0.05);
  padding: 2px 8px;
  border-radius: 4px;
  font-family: monospace;
}

.env-var-desc {
  color: #6b7280;
  font-size: 12px;
}

.steps {
  margin-bottom: 16px;
}

.steps h5 {
  font-size: 13px;
  color: #94a3b8;
  margin: 0 0 8px;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.step-detail {
  display: flex;
  flex-direction: column;
  gap: 4px;
  margin-top: 4px;
}

.step-command {
  background: rgba(0, 0, 0, 0.3);
  padding: 4px 8px;
  border-radius: 4px;
  font-family: monospace;
  font-size: 12px;
  color: #a5f3fc;
}

.step-note {
  font-size: 12px;
  color: #f59e0b;
}

.auto-config-section {
  margin: 16px 0;
}

.config-result {
  margin-bottom: 16px;
}

.report-details {
  margin-top: 8px;
  font-size: 13px;
  color: #94a3b8;
}

.report-details p {
  margin: 4px 0;
  display: flex;
  align-items: center;
  gap: 6px;
}

.config-paths {
  margin-top: 12px;
}

.config-paths h5 {
  font-size: 13px;
  color: #94a3b8;
  margin: 0 0 4px;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.config-paths p {
  margin: 2px 0;
  font-size: 13px;
  color: #94a3b8;
}

.config-paths code {
  color: #a5f3fc;
  background: rgba(0, 0, 0, 0.3);
  padding: 1px 4px;
  border-radius: 3px;
  font-family: monospace;
  font-size: 12px;
}

.no-guides {
  padding: 40px 0;
  text-align: center;
}

.help-text {
  font-size: 14px;
  color: #94a3b8;
}

.guide-footer {
  margin-top: 32px;
  padding-top: 24px;
  border-top: 1px solid rgba(255, 255, 255, 0.06);
  display: flex;
  justify-content: flex-end;
}

@media (prefers-color-scheme: light) {
  .provider-card,
  .agent-guide-card {
    background: rgba(248, 250, 252, 0.9);
    border-color: rgba(0, 0, 0, 0.08);
  }
  .guide-header h2,
  .section-title,
  .agent-guide-header h4,
  .guide-subtitle {
    color: #1e293b;
  }
  .section-desc,
  .help-text,
  .env-var-desc,
  .config-paths p {
    color: #64748b;
  }
}
</style>