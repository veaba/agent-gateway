<template>
  <div class="plan-card" :class="{ 'is-default': plan.id === defaultPlanId }">
    <div class="plan-glow"></div>

    <!-- Card Content -->
    <div class="plan-card-main" @click="$emit('viewDetail', plan)">
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
          <el-icon class="expand-icon">
            <ArrowRight />
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
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { Platform } from '@element-plus/icons-vue'
import type { UserPlan, Provider } from '@/types'
import { healthLabels, providerNames } from '@/constants/HealthLabel'

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
  viewDetail: [plan: UserPlan]
}>()

const providerInfo = computed(() => props.provider || {
  providerId: props.plan.providerId,
  name: getProviderName(props.plan.providerId),
  apiFormat: 'anthropic',
  homepage: '',
  docsUrl: '',
} as Provider)

const currentPlanTemplate = computed(() => {
  if (!props.provider?.codingPlans) return undefined
  return props.provider.codingPlans.find(cp => cp.planId === props.plan.planId)
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

const formatTime = (time: string) => {
  const date = new Date(time)
  return `${date.getHours()}:${String(date.getMinutes()).padStart(2, '0')}`
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
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.plan-card.is-default {
  border-color: var(--el-color-primary);
  background: linear-gradient(135deg, var(--el-color-primary-light-9) 0%, var(--el-bg-color) 100%);
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
  color: var(--el-text-color-secondary);
  font-size: 16px;
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
  0%, 100% {
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
</style>