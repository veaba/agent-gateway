<template>
  <div class="plan-card" :class="{ 'is-default': plan.id === defaultPlanId }">
    <div class="plan-glow"></div>
    <div class="plan-header">
      <div class="plan-provider">
        <el-tag size="small" :type="getProviderType(plan.provider_id)" effect="dark" round>
          {{ getProviderName(plan.provider_id) }}
        </el-tag>
        <span class="plan-name">{{ plan.name }}</span>
      </div>
      <el-tag v-if="plan.id === defaultPlanId" type="primary" size="small" effect="dark" round>默认</el-tag>
    </div>

    <div class="plan-body">
      <div class="plan-info">
        <div class="info-item">
          <span class="label">
            <el-icon><Monitor /></el-icon>
            模型
          </span>
          <span class="value model-value">{{ plan.selected_model_id }}</span>
        </div>
        <div class="info-item">
          <span class="label">
            <el-icon><Robot /></el-icon>
            Agent
          </span>
          <div class="agent-tags">
            <el-tag v-for="agent in plan.bound_agents" :key="agent.agent_id" size="small" effect="plain" round>
              {{ agent.agent_id }}
            </el-tag>
          </div>
        </div>
      </div>

      <div class="quota-section">
        <div class="quota-header">
          <span class="quota-label">
            <el-icon><DataLine /></el-icon>
            日配额使用
          </span>
          <span class="quota-value">{{ quotaUsed }} / {{ quotaLimit }}</span>
        </div>
        <el-progress
          :percentage="quotaPercent"
          :stroke-width="8"
          :color="getQuotaColor(quotaPercent)"
          :show-text="false"
        />
      </div>

      <div class="health-status">
        <div class="status-indicator" :class="'status-' + plan.health_status"></div>
        <el-tag :type="getHealthType(plan.health_status)" size="small" effect="dark" round>
          {{ getHealthLabel(plan.health_status) }}
        </el-tag>
        <span v-if="plan.last_health_check" class="last-check">
          {{ formatTime(plan.last_health_check) }}
        </span>
      </div>
    </div>

    <div class="plan-actions">
      <el-button size="default" class="action-btn" @click="$emit('edit', plan)">
        <el-icon><Edit /></el-icon>
        编辑
      </el-button>
      <el-button size="default" type="success" class="action-btn" @click="$emit('test', plan)">
        <el-icon><Connection /></el-icon>
        测试
      </el-button>
      <el-button size="default" type="danger" class="action-btn" @click="$emit('delete', plan)">
        <el-icon><Delete /></el-icon>
        删除
      </el-button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { UserPlan } from '@/types'

const props = defineProps<{
  plan: UserPlan
  defaultPlanId?: string
}>()

defineEmits<{
  edit: [plan: UserPlan]
  delete: [plan: UserPlan]
  test: [plan: UserPlan]
}>()

const quotaUsed = computed(() => props.plan.quota_used ?? 0)
const quotaLimit = computed(() => props.plan.quota_limit ?? 500)
const quotaPercent = computed(() => {
  if (!quotaLimit.value) return 0
  return Math.floor((quotaUsed.value / quotaLimit.value) * 100)
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
  const names: Record<string, string> = {
    alaya: 'Alaya',
    anthropic: 'Anthropic',
    kimi: 'Kimi'
  }
  return names[providerId] || providerId
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
  const labels: Record<string, string> = {
    healthy: '正常',
    warning: '警告',
    error: '错误',
    unknown: '未知',
    disabled: '已禁用'
  }
  return labels[status] || status
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
  padding: 20px;
  transition: all 0.3s ease;
  overflow: hidden;
}

.plan-card:hover {
  transform: translateY(-4px);
  box-shadow: 0 12px 40px rgba(0, 0, 0, 0.1);
  border-color: var(--el-color-primary-light-5);
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
}

.plan-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.plan-provider {
  display: flex;
  align-items: center;
  gap: 12px;
}

.plan-name {
  font-weight: 600;
  font-size: 18px;
  color: var(--el-text-color-primary);
}

.plan-body {
  margin-bottom: 20px;
}

.plan-info {
  margin-bottom: 16px;
}

.info-item {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 12px;
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
  font-family: 'SF Mono', Monaco, monospace;
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
  margin-bottom: 16px;
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
  font-family: 'SF Mono', Monaco, monospace;
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

.status-healthy { background: #67c23a; }
.status-warning { background: #e6a23c; }
.status-error { background: #f56c6c; }
.status-unknown { background: #909399; }
.status-disabled { background: #c0c4cc; }

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}

.last-check {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

.plan-actions {
  display: flex;
  gap: 8px;
  padding-top: 16px;
  border-top: 1px solid var(--el-border-color-lighter);
}

.action-btn {
  flex: 1;
  border-radius: 8px;
}
</style>