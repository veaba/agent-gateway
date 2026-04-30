<template>
  <div class="quota-view">
    <div class="quota-header agw-stagger">
      <div class="header-info">
        <h2 class="section-title">配额使用情况</h2>
        <p class="section-desc">监控各套餐的日配额、月配额和 RPM 限制</p>
      </div>
      <div class="header-actions">
        <el-select v-model="selectedPlanId" placeholder="全部套餐" clearable style="width: 180px">
          <el-option
            v-for="plan in plans"
            :key="plan.id"
            :label="plan.name"
            :value="plan.id"
          />
        </el-select>
        <el-button @click="loadData">
          <el-icon><Refresh /></el-icon>
          刷新
        </el-button>
      </div>
    </div>

    <!-- Quota Cards -->
    <div class="quota-cards agw-stagger" v-loading="loading">
      <div v-if="quotas.length === 0 && !loading" class="empty-state">
        <el-empty description="暂无配额数据" :image-size="80">
          <el-button type="primary" @click="loadData">刷新</el-button>
        </el-empty>
      </div>

      <div v-else class="quota-list">
        <div v-for="quota in filteredQuotas" :key="quota.plan_id" class="quota-card">
          <div class="quota-card-header">
            <div class="plan-info">
              <el-icon class="plan-icon"><Connection /></el-icon>
              <span class="plan-name">{{ getPlanName(quota.plan_id) }}</span>
            </div>
            <div class="header-tags">
              <el-tag
                v-if="quota.alert"
                size="small"
                round
                :type="getAlertType(quota.alert.alert_type)"
                class="alert-tag"
              >
                <el-icon><Warning /></el-icon>
                {{ getAlertLabel(quota.alert.alert_type) }}
              </el-tag>
              <el-tag size="small" round :type="getPlanHealthType(quota.plan_id)">
                {{ getPlanHealth(quota.plan_id) }}
              </el-tag>
            </div>
          </div>

          <div v-if="quota.alert" class="quota-alert-banner">
            <el-icon><Warning /></el-icon>
            <span>{{ quota.alert.message }}</span>
          </div>

          <div class="quota-metrics">
            <!-- Daily Quota -->
            <div class="metric-item">
              <div class="metric-header">
                <span class="metric-label">
                  <el-icon><Calendar /></el-icon>
                  日配额
                </span>
                <span class="metric-value agw-mono">
                  {{ formatNumber(quota.usage.daily_used) }} / {{ quota.limits.daily ? formatNumber(quota.limits.daily) : '∞' }}
                </span>
              </div>
              <el-progress
                :percentage="getPercent(quota.usage.daily_used, quota.limits.daily)"
                :stroke-width="8"
                :show-text="false"
                :color="getQuotaColor(getPercent(quota.usage.daily_used, quota.limits.daily))"
              />
            </div>

            <!-- Monthly Quota -->
            <div class="metric-item">
              <div class="metric-header">
                <span class="metric-label">
                  <el-icon><Month /></el-icon>
                  月配额
                </span>
                <span class="metric-value agw-mono">
                  {{ formatNumber(quota.usage.monthly_used) }} / {{ quota.limits.monthly ? formatNumber(quota.limits.monthly) : '∞' }}
                </span>
              </div>
              <el-progress
                :percentage="getPercent(quota.usage.monthly_used, quota.limits.monthly)"
                :stroke-width="8"
                :show-text="false"
                :color="getQuotaColor(getPercent(quota.usage.monthly_used, quota.limits.monthly))"
              />
            </div>

            <!-- RPM -->
            <div class="metric-item">
              <div class="metric-header">
                <span class="metric-label">
                  <el-icon><Timer /></el-icon>
                  RPM (请求/分钟)
                </span>
                <span class="metric-value agw-mono">
                  {{ quota.usage.rpm_used }} / {{ quota.limits.rpm || '∞' }}
                </span>
              </div>
              <el-progress
                :percentage="getPercent(quota.usage.rpm_used, quota.limits.rpm)"
                :stroke-width="8"
                :show-text="false"
                :color="getRpmColor(getPercent(quota.usage.rpm_used, quota.limits.rpm))"
              />
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { ElMessage } from 'element-plus'
import { Refresh, Connection, Warning, Calendar, Moon as Month, Timer } from '@element-plus/icons-vue'
import { fetchQuotaStatus, fetchPlans } from '@/api'
import type { QuotaStatus, UserPlan } from '@/types'

const quotas = ref<QuotaStatus[]>([])
const plans = ref<UserPlan[]>([])
const loading = ref(false)
const selectedPlanId = ref<string | undefined>(undefined)

// Filtered quotas
const filteredQuotas = computed(() => {
  if (!selectedPlanId.value) return quotas.value
  return quotas.value.filter(q => q.plan_id === selectedPlanId.value)
})

// Load data
const loadData = async () => {
  loading.value = true
  try {
    const [quotasData, plansData] = await Promise.all([
      fetchQuotaStatus(selectedPlanId.value),
      fetchPlans()
    ])
    quotas.value = quotasData
    plans.value = plansData
  } catch {
    ElMessage.error('加载配额数据失败')
  } finally {
    loading.value = false
  }
}

// Helper functions
const getPlanName = (planId: string) => {
  const plan = plans.value.find(p => p.id === planId)
  return plan?.name || planId
}

const getPlanHealth = (planId: string) => {
  const plan = plans.value.find(p => p.id === planId)
  const status = plan?.health_status || 'unknown'
  const labels: Record<string, string> = {
    healthy: '正常',
    warning: '警告',
    error: '错误',
    unknown: '未知',
    disabled: '已禁用'
  }
  return labels[status] || status
}

const getPlanHealthType = (planId: string) => {
  const plan = plans.value.find(p => p.id === planId)
  const status = plan?.health_status || 'unknown'
  const types: Record<string, string> = {
    healthy: 'success',
    warning: 'warning',
    error: 'danger',
    unknown: 'info',
    disabled: 'info'
  }
  return types[status] || 'info'
}

const getPercent = (used: number, limit?: number) => {
  if (!limit) return 0
  return Math.min(Math.floor((used / limit) * 100), 100)
}

const getQuotaColor = (percent: number) => {
  if (percent >= 90) return '#f43f5e'
  if (percent >= 70) return '#f59e0b'
  return '#10b981'
}

const getRpmColor = (percent: number) => {
  if (percent >= 90) return '#f43f5e'
  if (percent >= 70) return '#f59e0b'
  if (percent >= 50) return '#22d3ee'
  return '#10b981'
}

const getAlertType = (alertType: string) => {
  if (alertType.includes('exceeded')) return 'danger'
  return 'warning'
}

const getAlertLabel = (alertType: string) => {
  const labels: Record<string, string> = {
    daily_threshold: '日配额告警',
    monthly_threshold: '月配额告警',
    daily_exceeded: '日配额超额',
    monthly_exceeded: '月配额超额'
  }
  return labels[alertType] || '配额告警'
}

const formatNumber = (num?: number) => {
  if (num === undefined || num === null) return '0'
  if (num >= 1000000) return `${(num / 1000000).toFixed(1)}M`
  if (num >= 1000) return `${(num / 1000).toFixed(1)}K`
  return num.toString()
}

onMounted(() => {
  loadData()
})
</script>

<style scoped>
.quota-view {
  animation: fadeIn 0.5s ease;
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(10px); }
  to { opacity: 1; transform: translateY(0); }
}

.quota-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
  padding: 18px 24px;
  background: rgba(20, 23, 34, 0.7);
  backdrop-filter: blur(20px);
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 14px;
}

.header-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.section-title {
  font-size: 18px;
  font-weight: 700;
  color: #e8eaf0;
  margin: 0;
}

.section-desc {
  font-size: 13px;
  color: #6b7280;
  margin: 0;
}

.header-actions {
  display: flex;
  gap: 10px;
}

.quota-cards {
  background: rgba(20, 23, 34, 0.7);
  backdrop-filter: blur(20px);
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 14px;
  padding: 20px;
}

.empty-state {
  padding: 40px;
  text-align: center;
}

.quota-list {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(400px, 1fr));
  gap: 20px;
}

.quota-card {
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 12px;
  padding: 18px;
  transition: all 0.3s ease;
}

.quota-card:hover {
  border-color: rgba(255, 255, 255, 0.1);
  background: rgba(255, 255, 255, 0.05);
}

.quota-card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
  padding-bottom: 12px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.05);
}

.plan-info {
  display: flex;
  align-items: center;
  gap: 8px;
}

.plan-icon {
  color: #0ea5e9;
  font-size: 16px;
}

.plan-name {
  font-weight: 600;
  font-size: 15px;
  color: #e8eaf0;
}

.quota-metrics {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.metric-item {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.metric-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.metric-label {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: #6b7280;
}

.metric-value {
  font-size: 12px;
  font-weight: 600;
  color: #94a3b8;
}

.header-tags {
  display: flex;
  align-items: center;
  gap: 8px;
}

.alert-tag {
  font-weight: 600;
}

.alert-tag .el-icon {
  margin-right: 4px;
}

.quota-alert-banner {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 12px;
  margin-bottom: 14px;
  background: rgba(244, 63, 94, 0.08);
  border: 1px solid rgba(244, 63, 94, 0.2);
  border-radius: 8px;
  font-size: 12px;
  color: #fb7185;
}

.quota-alert-banner .el-icon {
  font-size: 14px;
  flex-shrink: 0;
}
</style>