<template>
  <div class="plan-card" :class="{ 'is-default': plan.id === defaultPlanId, 'is-expanded': expanded }">
    <div class="plan-glow"></div>

    <!-- Collapsed Header -->
    <div class="plan-card-main" @click="toggleExpand">
      <div class="plan-header">
        <div class="plan-provider">
          <el-tag size="small" :type="getProviderType(plan.providerId)" effect="dark" round>
            {{ getProviderName(plan.providerId) || 's' }}
          </el-tag>
          <span class="plan-name">{{ plan.name }}</span>
          <el-tag v-if="getPlanTier()" size="small" :type="getTierType(getPlanTier())" effect="plain" round
            class="tier-tag">
            {{ getPlanTier() }}
          </el-tag>
        </div>
        <div class="plan-header-right">
          <el-tag v-if="plan.id === defaultPlanId" type="primary" size="small" effect="dark" round>默认</el-tag>
          <el-icon class="expand-icon" :class="{ 'is-rotated': expanded }">
            <ArrowDown />
          </el-icon>
        </div>
      </div>

      <div class="plan-body">
        <div class="plan-info">
          <div class="info-item">
            <span class="label">
              <el-icon>
                <Monitor />
              </el-icon>
              模型
            </span>
            <span class="value model-value">{{ plan.selectedModelId }}</span>
          </div>
          <div class="info-item">
            <span class="label">
              <el-icon>
                <Platform />
              </el-icon>
              Agent
            </span>
            <div class="agent-tags">
              <el-tag v-for="agent in plan.boundAgents" :key="agent.agentId" size="small" effect="plain" round>
                {{ agent.agentId }}
              </el-tag>
              <span v-if="!plan.boundAgents?.length" class="muted">未绑定</span>
            </div>
          </div>
        </div>

        <div class="quota-section">
          <div class="quota-header">
            <span class="quota-label">
              <el-icon>
                <DataLine />
              </el-icon>
              日配额使用
            </span>
            <span class="quota-value">{{ quotaUsed }} / {{ quotaLimit }}</span>
          </div>
          <el-progress :percentage="quotaPercent" :stroke-width="8" :color="getQuotaColor(quotaPercent)"
            :show-text="false" />
        </div>

        <div class="health-status">
          <div class="status-indicator" :class="'status-' + plan.healthStatus"></div>
          <el-tag :type="getHealthType(plan.healthStatus)" size="small" effect="dark" round>
            {{ getHealthLabel(plan.healthStatus) }}
          </el-tag>
          <span v-if="plan.lastHealthCheck" class="last-check">
            {{ formatTime(plan.lastHealthCheck) }}
          </span>
        </div>
      </div>
    </div>

    <!-- Expanded Detail (per design doc 5.3) -->
    <transition name="detail-expand">
      <div v-if="expanded" class="plan-detail">
        <div class="detail-toolbar">
          <span class="detail-title">{{ plan.name }} 详情</span>
          <el-button text size="small" @click="expanded = false">
            <el-icon>
              <ArrowUp />
            </el-icon>
            收起
          </el-button>
        </div>

        <!-- Provider Info Section -->
        <div class="detail-section">
          <div class="section-header">
            <el-icon>
              <OfficeBuilding />
            </el-icon>
            <span>Provider 信息</span>
          </div>
          <div class="section-body">
            <div class="detail-row">
              <span class="detail-label">名称</span>
              <span class="detail-value">{{ providerInfo.name || plan.providerId }}</span>
            </div>
            <div v-if="providerInfo.homepage" class="detail-row">
              <span class="detail-label">官网</span>
              <a class="detail-link" :href="providerInfo.homepage" target="_blank">
                {{ providerInfo.homepage }}
                <el-icon size="12">
                  <Link />
                </el-icon>
              </a>
            </div>
            <div v-if="providerInfo.docsUrl" class="detail-row">
              <span class="detail-label">文档</span>
              <a class="detail-link" :href="providerInfo.docsUrl" target="_blank">
                {{ providerInfo.docsUrl }}
                <el-icon size="12">
                  <Link />
                </el-icon>
              </a>
            </div>
            <div v-if="providerInfo.getApiKeyUrl" class="detail-row">
              <span class="detail-label">API Key</span>
              <a class="detail-link" :href="providerInfo.getApiKeyUrl" target="_blank">
                获取 API Key
                <el-icon size="12">
                  <Link />
                </el-icon>
              </a>
            </div>
            <div class="detail-row">
              <span class="detail-label">API 格式</span>
              <el-tag size="small" effect="plain" round>{{ providerInfo.apiFormat || plan.providerId }}</el-tag>
            </div>
          </div>
        </div>

        <!-- Coding Plan Info Section -->
        <div class="detail-section">
          <div class="section-header">
            <el-icon>
              <Tickets />
            </el-icon>
            <span>Coding Plan 信息</span>
          </div>
          <div class="section-body">
            <div class="detail-row">
              <span class="detail-label">方案</span>
              <span class="detail-value">{{ currentPlanTemplate?.name || plan.planId }}</span>
            </div>
            <div v-if="currentPlanTemplate" class="detail-row">
              <span class="detail-label">等级</span>
              <el-tag size="small" :type="getTierType(currentPlanTemplate.tier)" effect="plain" round>
                {{ getTierLabel(currentPlanTemplate.tier) }}
              </el-tag>
            </div>
            <div v-if="currentPlanTemplate?.description" class="detail-row">
              <span class="detail-label">描述</span>
              <span class="detail-value description-value">{{ currentPlanTemplate.description }}</span>
            </div>
            <div v-if="currentPlanTemplate?.price" class="detail-row">
              <span class="detail-label">订阅</span>
              <span class="detail-value price-value">{{ currentPlanTemplate.price }}</span>
            </div>
            <div v-if="currentPlanTemplate?.features?.length" class="detail-row">
              <span class="detail-label">特性</span>
              <div class="feature-tags">
                <el-tag v-for="f in currentPlanTemplate.features" :key="f" size="small" effect="plain" round
                  type="info">
                  {{ f }}
                </el-tag>
              </div>
            </div>
          </div>
        </div>

        <!-- Model Config Section -->
        <div class="detail-section">
          <div class="section-header">
            <el-icon>
              <Cpu />
            </el-icon>
            <span>模型配置</span>
          </div>
          <div class="section-body">
            <div class="detail-row">
              <span class="detail-label">当前</span>
              <el-tag size="default" effect="dark" round type="primary">{{ plan.selectedModelId }}</el-tag>
            </div>
            <div v-if="availableModels.length" class="detail-row">
              <span class="detail-label">可选</span>
              <div class="model-chips">
                <el-tag v-for="m in availableModels" :key="m.modelId" size="small"
                  :type="m.modelId === plan.selectedModelId ? 'primary' : 'info'"
                  :effect="m.modelId === plan.selectedModelId ? 'dark' : 'plain'" round class="model-chip">
                  {{ m.name }}
                </el-tag>
              </div>
            </div>
            <div v-if="currentModelInfo" class="detail-row">
              <span class="detail-label">能力</span>
              <div class="capability-tags">
                <el-tag v-for="cap in currentModelInfo.capabilities" :key="cap" size="small" effect="plain" round
                  :type="getCapabilityType(cap)">
                  {{ getCapabilityLabel(cap) }}
                </el-tag>
              </div>
            </div>
            <div v-if="currentModelInfo?.contextLength" class="detail-row">
              <span class="detail-label">上下文</span>
              <span class="detail-value mono">{{ formatContextLength(currentModelInfo.contextLength) }}</span>
            </div>
          </div>
        </div>

        <!-- Agent Tools Binding Section -->
        <div class="detail-section">
          <div class="section-header">
            <el-icon>
              <Connection />
            </el-icon>
            <span>Agent 工具绑定</span>
          </div>
          <div class="section-body">
            <div v-if="!plan.boundAgents.length" class="empty-agents">
              <span class="muted">未绑定任何 Agent 工具</span>
            </div>
            <div v-for="agent in plan.boundAgents" :key="agent.agentId" class="agent-binding-row">
              <div class="agent-binding-header">
                <span class="agent-name">{{ getAgentName(agent.agentId) }}</span>
                <el-tag size="small" :type="agent.configured ? 'success' : 'danger'" effect="dark" round>
                  {{ getAgentStatusLabel(agent) }}
                </el-tag>
              </div>
              <div class="agent-binding-detail">
                <div v-if="getAgentSetupGuide(agent.agentId)" class="agent-env-vars">
                  <div v-for="ev in getAgentEnvVars(agent.agentId)" :key="ev.name" class="env-var-row">
                    <span class="mono env-var-name">{{ ev.name }}</span>
                    <span class="env-var-value">{{ ev.value }}</span>
                  </div>
                  <div v-if="getAgentConfigPaths(agent.agentId)" class="config-path">
                    <el-icon size="12">
                      <Document />
                    </el-icon>
                    {{ getAgentConfigPaths(agent.agentId) }}
                  </div>
                </div>
                <div class="agent-actions">
                  <el-button v-if="!agent.configured" size="small" type="primary"
                    @click="$emit('autoConfig', plan.id, agent.agentId)">
                    <el-icon>
                      <SetUp />
                    </el-icon>
                    一键配置
                  </el-button>
                  <el-button size="small" @click="$emit('testAgent', plan.id, agent.agentId)">
                    <el-icon>
                      <Connection />
                    </el-icon>
                    测试连接
                  </el-button>
                </div>
              </div>
            </div>
            <!-- Unbound agents supported by provider -->
            <div v-if="unboundAgents.length" class="unbound-section">
              <div class="unbound-header">可绑定的 Agent 工具</div>
              <div class="unbound-agents">
                <el-tag v-for="ua in unboundAgents" :key="ua.agentId" size="small" effect="plain" round type="info"
                  class="unbound-tag" @click="$emit('bindAgent', plan.id, ua.agentId)">
                  + {{ ua.name }}
                </el-tag>
              </div>
            </div>
          </div>
        </div>

        <!-- API Key Section -->
        <div class="detail-section">
          <div class="section-header">
            <el-icon>
              <Key />
            </el-icon>
            <span>API Key</span>
          </div>
          <div class="section-body">
            <div class="detail-row">
              <span class="detail-label">状态</span>
              <el-tag :type="plan.apiKeyMasked ? 'success' : 'danger'" size="small" effect="dark" round>
                {{ plan.apiKeyMasked ? '已配置' : '未配置' }}
              </el-tag>
            </div>
            <div v-if="plan.apiKeyMasked" class="detail-row">
              <span class="detail-label">Key</span>
              <span class="detail-value mono api-key-masked">{{ plan.apiKeyMasked }}</span>
            </div>
            <div class="api-key-actions">
              <el-button size="small" @click="$emit('edit', plan)">
                <el-icon>
                  <Edit />
                </el-icon>
                更新 Key
              </el-button>
              <el-button v-if="providerInfo.getApiKeyUrl" size="small" type="primary"
                @click="openUrl(providerInfo.getApiKeyUrl)">
                <el-icon>
                  <Link />
                </el-icon>
                获取 Key
              </el-button>
            </div>
          </div>
        </div>

        <!-- Quota & Limits Section -->
        <div class="detail-section">
          <div class="section-header">
            <el-icon>
              <Odometer />
            </el-icon>
            <span>配额与限制</span>
          </div>
          <div class="section-body">
            <div class="quota-grid">
              <div class="quota-item">
                <span class="quota-item-label">日配额</span>
                <span class="quota-item-value">
                  {{ plan.customQuotaDaily ?? currentPlanTemplate?.quotaDaily ?? '无限制' }}
                  <template v-if="plan.customQuotaDaily || currentPlanTemplate?.quotaDaily">
                    {{ quotaUsed }} 已用
                  </template>
                </span>
                <el-progress v-if="quotaPercent > 0" :percentage="quotaPercent" :stroke-width="6"
                  :color="getQuotaColor(quotaPercent)" :show-text="false" class="quota-progress" />
              </div>
              <div class="quota-item">
                <span class="quota-item-label">月配额</span>
                <span class="quota-item-value">
                  {{ plan.customQuotaMonthly ?? currentPlanTemplate?.quotaMonthly ?? '无限制' }}
                </span>
              </div>
              <div class="quota-item">
                <span class="quota-item-label">RPM 限制</span>
                <span class="quota-item-value">{{ plan.customRpmLimit ?? currentPlanTemplate?.rpmLimit ?? '无限制'
                  }}</span>
              </div>
              <div class="quota-item">
                <span class="quota-item-label">Fallback 优先级</span>
                <span class="quota-item-value">{{ plan.priority }}</span>
              </div>
            </div>
          </div>
        </div>

        <!-- Action Buttons -->
        <div class="detail-actions">
          <el-button size="default" @click="$emit('edit', plan)">
            <el-icon>
              <Edit />
            </el-icon>
            编辑
          </el-button>
          <el-button size="default" type="success" @click="$emit('test', plan)">
            <el-icon>
              <Connection />
            </el-icon>
            测试连接
          </el-button>
          <el-button v-if="plan.id !== defaultPlanId" size="default" type="warning" @click="$emit('setDefault', plan)">
            <el-icon>
              <Star />
            </el-icon>
            设为默认
          </el-button>
          <el-button size="default" type="danger" @click="$emit('delete', plan)">
            <el-icon>
              <Delete />
            </el-icon>
            删除
          </el-button>
        </div>
      </div>
    </transition>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { Platform } from '@element-plus/icons-vue'
import type { UserPlan, Provider, CodingPlan, Model } from '@/types'
import { healthLabels, providerNames, statusLabels } from '@/constants/HealthLabel'

const props = defineProps<{
  plan: UserPlan
  defaultPlanId?: string
  provider?: Provider
}>()

const emit = defineEmits<{
  edit: [plan: UserPlan]
  delete: [plan: UserPlan]
  test: [plan: UserPlan]
  setDefault: [plan: UserPlan]
  bindAgent: [planId: string, agentId: string]
  autoConfig: [planId: string, agentId: string]
  testAgent: [planId: string, agentId: string]
}>()

const expanded = ref(false)

const toggleExpand = () => {
  expanded.value = !expanded.value
}

const providerInfo = computed(() => props.provider || {
  providerId: props.plan.providerId,
  name: getProviderName(props.plan.providerId),
  apiFormat: 'anthropic',
  homepage: '',
  docsUrl: '',
} as Provider)

const currentPlanTemplate = computed((): CodingPlan | undefined => {
  if (!props.provider?.codingPlans) return undefined
  return props.provider.codingPlans.find(cp => cp.planId === props.plan.planId)
})

const availableModels = computed((): Model[] => {
  if (!props.provider?.models) return []
  const template = currentPlanTemplate.value
  if (!template) return props.provider.models || []
  return props.provider.models.filter(m => template.supportedModelIds.includes(m.modelId))
})

const currentModelInfo = computed((): Model | undefined => {
  return availableModels.value.find(m => m.modelId === props.plan.selectedModelId)
})

const unboundAgents = computed(() => {
  if (!props.provider?.supportedAgents) return []
  const boundIds = props.plan.boundAgents.map(a => a.agentId)
  if (currentPlanTemplate.value?.supportedAgentIds) {
    return props.provider.supportedAgents.filter(
      a => currentPlanTemplate.value!.supportedAgentIds.includes(a.agentId) && !boundIds.includes(a.agentId)
    )
  }
  return props.provider.supportedAgents.filter(a => !boundIds.includes(a.agentId))
})

const quotaUsed = computed(() => props.plan.quotaUsed ?? 0)
const quotaLimit = computed(() => props.plan.quotaLimit ?? (currentPlanTemplate.value?.quotaDaily ?? 500))
const quotaPercent = computed(() => {
  if (!quotaLimit.value) return 0
  return Math.min(100, Math.floor((quotaUsed.value / quotaLimit.value) * 100))
})

const getProviderType = (providerId: string) => {
  const types: Record<string, string> = {
    alaya: 'primary',
    anthropic: 'success',
    kimi: 'warning'
  }
  return types[providerId] || 'info'
}

const getProviderName = (providerId: string) => {
  return providerNames[providerId] || providerId
}

const getPlanTier = () => currentPlanTemplate.value?.tier || ''

const getTierType = (tier: string) => {
  const types: Record<string, string> = {
    free: 'success',
    pro: 'warning',
    enterprise: 'danger',
    custom: 'info'
  }
  return types[tier] || 'info'
}

const getTierLabel = (tier: string) => {
  const labels: Record<string, string> = {
    free: '免费版',
    pro: '专业版',
    enterprise: '企业版',
    custom: '自定义'
  }
  return labels[tier] || tier
}

const getHealthType = (status: string) => {
  const types: Record<string, string> = {
    healthy: 'success',
    warning: 'warning',
    error: 'danger',
    unknown: 'info',
    disabled: 'info'
  }
  return types[status] || 'info'
}

const getHealthLabel = (status: string) => {
  return healthLabels[status] || status
}

const getQuotaColor = (percent: number) => {
  if (percent >= 90) return '#f56c6c'
  if (percent >= 70) return '#e6a23c'
  return '#67c23a'
}

const getAgentName = (agentId: string) => {
  if (props.provider?.supportedAgents) {
    const found = props.provider.supportedAgents.find(a => a.agentId === agentId)
    if (found) return found.name
  }
  const names: Record<string, string> = {
    'claude-code': 'Claude Code',
    'kimi-cli': 'Kimi CLI',
    'opencode': 'OpenCode',
    'kilo-cli': 'Kilo CLI'
  }
  return names[agentId] || agentId
}

const getAgentStatusLabel = (agent: { configured: boolean; configStatus: string }) => {
  if (agent.configured) return '已配置'
  return statusLabels[agent.configStatus] || '未配置'
}

const getAgentSetupGuide = (agentId: string) => {
  if (!props.provider?.onboarding) return null
  return props.provider.onboarding.agentSetupGuides.find(g => g.agentId === agentId)
}

const getAgentEnvVars = (agentId: string) => {
  const guide = getAgentSetupGuide(agentId)
  return guide?.envVars || []
}

const getAgentConfigPaths = (agentId: string) => {
  const guide = getAgentSetupGuide(agentId)
  if (!guide?.configFilePaths) return ''
  const paths = guide.configFilePaths
  return paths.linux || paths.macos || paths.windows || ''
}

const getCapabilityType = (cap: string) => {
  const types: Record<string, string> = {
    code: 'success',
    reasoning: 'primary',
    'long-context': 'warning',
    'math': 'danger',
    'chinese-optimized': 'info',
    'multimodal': 'warning'
  }
  return types[cap] || 'info'
}

const getCapabilityLabel = (cap: string) => {
  const labels: Record<string, string> = {
    code: '代码生成',
    reasoning: '推理',
    'long-context': '长上下文',
    'math': '数学',
    'chinese-optimized': '中文优化',
    'multimodal': '多模态'
  }
  return labels[cap] || cap
}

const formatContextLength = (len: number) => {
  if (len >= 1000000) return `${(len / 1000000).toFixed(1)}M`
  if (len >= 1000) return `${(len / 1000).toFixed(0)}K`
  return `${len}`
}

const formatTime = (time: string) => {
  const date = new Date(time)
  return `${date.getHours()}:${String(date.getMinutes()).padStart(2, '0')}`
}

const openUrl = (url: string) => {
  window.open(url, '_blank')
}
</script>

<style scoped>
.plan-card {
  position: relative;
  background: var(--el-bg-color);
  border: 1px solid var(--el-border-color-light);
  border-radius: 16px;
  padding: 0;
  transition: all 0.3s ease;
  overflow: hidden;
}

.plan-card:hover {
  border-color: var(--el-color-primary-light-5);
}

.plan-card.is-default {
  border-color: var(--el-color-primary);
  background: linear-gradient(135deg, var(--el-color-primary-light-9) 0%, var(--el-bg-color) 100%);
}

.plan-card.is-expanded {
  border-color: var(--el-color-primary-light-3);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.15);
}

.plan-glow {
  position: absolute;
  top: 0;
  right: 0;
  width: 120px;
  height: 120px;
  background: radial-gradient(circle, rgba(103, 126, 234, 0.15) 0%, transparent 70%);
  pointer-events: none;
  z-index: 0;
}

.plan-card-main {
  padding: 20px;
  cursor: pointer;
  position: relative;
  z-index: 1;
}

.plan-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.plan-provider {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
}

.plan-name {
  font-weight: 600;
  font-size: 18px;
  color: var(--el-text-color-primary);
}

.tier-tag {
  margin-left: 2px;
}

.plan-header-right {
  display: flex;
  align-items: center;
  gap: 8px;
}

.expand-icon {
  transition: transform 0.3s ease;
  color: var(--el-text-color-secondary);
  font-size: 16px;
}

.expand-icon.is-rotated {
  transform: rotate(180deg);
}

.plan-body {
  margin-bottom: 0;
}

.plan-info {
  margin-bottom: 14px;
}

.info-item {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 10px;
}

.info-item .label {
  display: flex;
  align-items: center;
  gap: 4px;
  color: var(--el-text-color-secondary);
  font-size: 13px;
  min-width: 60px;
}

.info-item .value {
  color: var(--el-text-color-primary);
  font-size: 14px;
}

.model-value {
  font-family: var(--agw-font-mono, monospace);
  background: var(--el-fill-color-light);
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 12px;
}

.agent-tags {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}

.quota-section {
  margin-bottom: 14px;
  padding: 12px;
  background: var(--el-fill-color-light);
  border-radius: 10px;
}

.quota-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.quota-label {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 13px;
  color: var(--el-text-color-secondary);
}

.quota-value {
  font-size: 13px;
  font-weight: 600;
  color: var(--el-text-color-primary);
  font-family: var(--agw-font-mono, monospace);
}

.health-status {
  display: flex;
  align-items: center;
  gap: 8px;
}

.status-indicator {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  animation: pulse 2s infinite;
}

.status-healthy {
  background: #67c23a;
}

.status-warning {
  background: #e6a23c;
}

.status-error {
  background: #f56c6c;
}

.status-unknown {
  background: #909399;
}

.status-disabled {
  background: #c0c4cc;
}

@keyframes pulse {

  0%,
  100% {
    opacity: 1;
  }

  50% {
    opacity: 0.5;
  }
}

.last-check {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

.muted {
  color: var(--el-text-color-muted);
  font-size: 13px;
}

/* ── Detail Section ── */

.plan-detail {
  border-top: 1px solid var(--el-border-color-lighter);
  padding: 20px;
  position: relative;
  z-index: 1;
  background: linear-gradient(180deg, var(--el-fill-color-lighter) 0%, transparent 20px);
}

.detail-toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
  padding-bottom: 12px;
  border-bottom: 1px solid var(--el-border-color-subtle, rgba(255, 255, 255, 0.04));
}

.detail-title {
  font-weight: 600;
  font-size: 16px;
  color: var(--el-text-color-primary);
}

.detail-section {
  margin-bottom: 16px;
  background: var(--el-bg-color);
  border: 1px solid var(--el-border-color-light);
  border-radius: 12px;
  overflow: hidden;
}

.section-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 16px;
  background: var(--el-fill-color-light);
  font-weight: 600;
  font-size: 13px;
  color: var(--el-text-color-primary);
  border-bottom: 1px solid var(--el-border-color-lighter);
}

.section-body {
  padding: 14px 16px;
}

.detail-row {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  margin-bottom: 10px;
  min-height: 28px;
}

.detail-row:last-child {
  margin-bottom: 0;
}

.detail-label {
  color: var(--el-text-color-secondary);
  font-size: 13px;
  min-width: 70px;
  flex-shrink: 0;
  padding-top: 3px;
}

.detail-value {
  color: var(--el-text-color-primary);
  font-size: 14px;
  flex: 1;
}

.detail-value.mono {
  font-family: var(--agw-font-mono, monospace);
}

.detail-value.price-value {
  font-weight: 600;
  color: var(--el-color-primary);
}

.detail-value.description-value {
  color: var(--el-text-color-regular);
  line-height: 1.5;
}

.detail-value.api-key-masked {
  font-family: var(--agw-font-mono, monospace);
  font-size: 13px;
  background: var(--el-fill-color-light);
  padding: 2px 8px;
  border-radius: 4px;
  letter-spacing: 0.5px;
}

.detail-link {
  color: var(--el-color-primary);
  text-decoration: none;
  font-size: 13px;
  display: inline-flex;
  align-items: center;
  gap: 4px;
  transition: color 0.2s;
}

.detail-link:hover {
  color: var(--el-color-primary-light-3);
  text-decoration: underline;
}

.feature-tags,
.capability-tags,
.model-chips {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.model-chip {
  cursor: default;
}

/* Agent binding section */
.agent-binding-row {
  padding: 12px 0;
  border-bottom: 1px solid var(--el-border-color-lighter);
}

.agent-binding-row:last-child {
  border-bottom: none;
}

.agent-binding-header {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 8px;
}

.agent-name {
  font-weight: 600;
  font-size: 14px;
  color: var(--el-text-color-primary);
}

.agent-binding-detail {
  padding-left: 4px;
}

.agent-env-vars {
  margin-bottom: 8px;
}

.env-var-row {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 4px;
}

.env-var-name {
  font-size: 12px;
  color: var(--agw-cyan, #00d4aa);
  font-family: var(--agw-font-mono, monospace);
  min-width: 220px;
}

.env-var-value {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  font-family: var(--agw-font-mono, monospace);
}

.config-path {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  color: var(--el-text-color-secondary);
  margin-top: 4px;
}

.agent-actions {
  display: flex;
  gap: 8px;
  margin-top: 8px;
}

.empty-agents {
  padding: 12px 0;
}

.unbound-section {
  margin-top: 12px;
  padding-top: 12px;
  border-top: 1px dashed var(--el-border-color-light);
}

.unbound-header {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  margin-bottom: 8px;
}

.unbound-agents {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.unbound-tag {
  cursor: pointer;
  transition: all 0.2s ease;
}

.unbound-tag:hover {
  opacity: 0.8;
}

/* API Key section */
.api-key-actions {
  display: flex;
  gap: 8px;
  margin-top: 8px;
}

/* Quota grid */
.quota-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
}

.quota-item {
  padding: 10px 12px;
  background: var(--el-fill-color-light);
  border-radius: 8px;
}

.quota-item-label {
  display: block;
  font-size: 12px;
  color: var(--el-text-color-secondary);
  margin-bottom: 4px;
}

.quota-item-value {
  display: block;
  font-size: 14px;
  font-weight: 600;
  color: var(--el-text-color-primary);
  font-family: var(--agw-font-mono, monospace);
}

.quota-progress {
  margin-top: 6px;
}

/* Detail actions */
.detail-actions {
  display: flex;
  gap: 8px;
  padding-top: 16px;
  border-top: 1px solid var(--el-border-color-lighter);
  margin-top: 4px;
}

/* Transition */
.detail-expand-enter-active {
  animation: detailExpand 0.3s ease-out;
}

.detail-expand-leave-active {
  animation: detailExpand 0.2s ease-in reverse;
}

@keyframes detailExpand {
  from {
    opacity: 0;
    max-height: 0;
    transform: translateY(-8px);
  }

  to {
    opacity: 1;
    max-height: 2000px;
    transform: translateY(0);
  }
}
</style>