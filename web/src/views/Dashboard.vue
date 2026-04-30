<template>
  <div class="dashboard">
    <!-- Stats Cards -->
    <el-row :gutter="20" class="stats-row agw-stagger">
      <el-col :xs="24" :sm="12" :lg="6">
        <div class="stat-card stat-card-cyan">
          <div class="stat-icon">
            <el-icon :size="28"><Connection /></el-icon>
          </div>
          <div class="stat-content">
            <div class="stat-value agw-mono">{{ stats.plans }}</div>
            <div class="stat-label">已配置套餐</div>
          </div>
          <div class="stat-glow"></div>
        </div>
      </el-col>
      <el-col :xs="24" :sm="12" :lg="6">
        <div class="stat-card stat-card-emerald">
          <div class="stat-icon">
            <el-icon :size="28"><PieChart /></el-icon>
          </div>
          <div class="stat-content">
            <div class="stat-value agw-mono">{{ stats.quotaUsedPercent }}%</div>
            <div class="stat-label">配额使用率</div>
            <el-progress
              :percentage="stats.quotaUsedPercent"
              :stroke-width="5"
              :show-text="false"
              class="stat-progress"
            />
          </div>
          <div class="stat-glow"></div>
        </div>
      </el-col>
      <el-col :xs="24" :sm="12" :lg="6">
        <div class="stat-card" :class="stats.healthStatus === 'healthy' ? 'stat-card-emerald' : 'stat-card-rose'">
          <div class="stat-icon">
            <el-icon :size="28"><CircleCheck /></el-icon>
          </div>
          <div class="stat-content">
            <div class="stat-value">
              <el-tag
                :type="stats.healthStatus === 'healthy' ? 'success' : 'danger'"
                effect="dark"
                round
                size="small"
                class="health-tag"
              >
                {{ stats.healthStatus === 'healthy' ? '正常' : '异常' }}
              </el-tag>
            </div>
            <div class="stat-label">健康状态 · {{ stats.lastCheck }}</div>
          </div>
          <div class="stat-glow"></div>
        </div>
      </el-col>
      <el-col :xs="24" :sm="12" :lg="6">
        <div class="stat-card stat-card-sky">
          <div class="stat-icon">
            <el-icon :size="28"><TrendCharts /></el-icon>
          </div>
          <div class="stat-content">
            <div class="stat-value agw-mono">{{ stats.requestsToday.toLocaleString() }}</div>
            <div class="stat-label">今日请求数</div>
          </div>
          <div class="stat-glow"></div>
        </div>
      </el-col>
    </el-row>

    <!-- Main Content -->
    <el-row :gutter="20" class="content-row">
      <el-col :xs="24" :lg="16">
        <div class="content-card">
          <div class="card-header">
            <span class="card-title">最近的请求</span>
            <div class="card-actions">
              <el-button text size="small" @click="loadLogs">
                <el-icon><Refresh /></el-icon>
                刷新
              </el-button>
              <el-button text size="small" @click="$router.push('/logs')">
                查看全部
                <el-icon class="el-icon--right"><ArrowRight /></el-icon>
              </el-button>
            </div>
          </div>
          <el-table
            :data="recentRequests"
            class="dashboard-table"
            :row-class-name="getRowClass"
            v-loading="loading.logs"
            stripe
          >
            <el-table-column prop="timestamp" label="时间" width="90">
              <template #default="{ row }">
                <span class="agw-mono">{{ formatTime(row.timestamp) }}</span>
              </template>
            </el-table-column>
            <el-table-column prop="plan_id" label="套餐" min-width="100">
              <template #default="{ row }">
                <span class="plan-name">{{ getPlanName(row.plan_id) }}</span>
              </template>
            </el-table-column>
            <el-table-column prop="agent_id" label="Agent" width="110" />
            <el-table-column prop="model_id" label="模型" min-width="100">
              <template #default="{ row }">
                <span class="model-badge">{{ row.model_id }}</span>
              </template>
            </el-table-column>
            <el-table-column label="状态" width="70" align="center">
              <template #default="{ row }">
                <el-tag
                  :type="getStatusType(row.status_code)"
                  size="small"
                  round
                  class="status-tag"
                >
                  {{ row.status_code || '—' }}
                </el-tag>
              </template>
            </el-table-column>
            <el-table-column label="延迟" width="80" align="right">
              <template #default="{ row }">
                <span class="latency-value agw-mono" :class="{ 'latency-high': row.latency_ms > 500 }">
                  {{ row.latency_ms ? `${row.latency_ms}ms` : '—' }}
                </span>
              </template>
            </el-table-column>
          </el-table>
        </div>
      </el-col>
      <el-col :xs="24" :lg="8">
        <div class="content-card">
          <div class="card-header">
            <span class="card-title">快速操作</span>
          </div>
          <div class="quick-actions">
            <div class="action-item action-primary" @click="$router.push('/plans/add')">
              <div class="action-icon">
                <el-icon :size="22"><Plus /></el-icon>
              </div>
              <div class="action-content">
                <div class="action-title">添加新套餐</div>
                <div class="action-desc">配置新的 AI 服务</div>
              </div>
              <el-icon class="action-arrow"><ArrowRight /></el-icon>
            </div>
            <div class="action-item" @click="$router.push('/plans')">
              <div class="action-icon">
                <el-icon :size="22"><Connection /></el-icon>
              </div>
              <div class="action-content">
                <div class="action-title">管理套餐</div>
                <div class="action-desc">编辑或删除现有套餐</div>
              </div>
              <el-icon class="action-arrow"><ArrowRight /></el-icon>
            </div>
            <div class="action-item" @click="$router.push('/fallback')">
              <div class="action-icon">
                <el-icon :size="22"><RefreshLeft /></el-icon>
              </div>
              <div class="action-content">
                <div class="action-title">配置降级策略</div>
                <div class="action-desc">设置自动故障转移</div>
              </div>
              <el-icon class="action-arrow"><ArrowRight /></el-icon>
            </div>
            <div class="action-item" @click="$router.push('/quota')">
              <div class="action-icon">
                <el-icon :size="22"><DataLine /></el-icon>
              </div>
              <div class="action-content">
                <div class="action-title">查看配额</div>
                <div class="action-desc">监控使用量与限额</div>
              </div>
              <el-icon class="action-arrow"><ArrowRight /></el-icon>
            </div>
          </div>
        </div>

        <!-- Quota Overview -->
        <div class="content-card quota-overview">
          <div class="card-header">
            <span class="card-title">配额概览</span>
          </div>
          <div class="quota-list" v-loading="loading.quota">
            <div v-for="quota in quotaSummary" :key="quota.plan_id" class="quota-item" :class="{ 'quota-alert': quota.alert }">
              <div class="quota-plan">
                <span class="quota-plan-name">
                  <el-icon v-if="quota.alert" class="alert-icon"><Warning /></el-icon>
                  {{ getPlanName(quota.plan_id) }}
                </span>
                <span class="quota-percent agw-mono" :class="{ 'text-alert': quota.alert }">{{ quota.percent }}%</span>
              </div>
              <el-progress
                :percentage="quota.percent"
                :stroke-width="6"
                :show-text="false"
                :color="getQuotaColor(quota.percent)"
              />
            </div>
            <el-empty v-if="quotaSummary.length === 0 && !loading.quota" description="暂无配额数据" :image-size="60" />
          </div>
        </div>
      </el-col>
    </el-row>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, computed } from 'vue'
import { fetchPlans, fetchQuotaStatus, fetchLogs, healthCheck } from '@/api'
import type { UserPlan, QuotaStatus, RequestLog } from '@/types'

// Loading states
const loading = reactive({
  plans: false,
  quota: false,
  logs: false,
  health: false
})

// Data
const plans = ref<UserPlan[]>([])
const quotas = ref<QuotaStatus[]>([])
const logs = ref<RequestLog[]>([])
const isHealthy = ref(false)

// Stats computed
const stats = computed(() => {
  const totalQuotaUsed = quotas.value.reduce((sum, q) => sum + q.usage.daily_used, 0)
  const totalQuotaLimit = quotas.value.reduce((sum, q) => sum + (q.limits.daily || 1), 0)
  const quotaPercent = totalQuotaLimit > 0 ? Math.floor((totalQuotaUsed / totalQuotaLimit) * 100) : 0

  const healthyPlans = plans.value.filter(p => p.health_status === 'healthy').length
  const healthStatus = plans.value.length === 0 ? 'unknown' : (healthyPlans === plans.value.length ? 'healthy' : 'error')

  const todayRequests = logs.value.length

  return {
    plans: plans.value.length,
    quotaUsedPercent: quotaPercent,
    healthStatus,
    lastCheck: '刚刚',
    requestsToday: todayRequests
  }
})

// Recent requests (last 5)
const recentRequests = computed(() => {
  return logs.value.slice(0, 5)
})

// Quota summary for sidebar
const quotaSummary = computed(() => {
  return quotas.value.map(q => ({
    plan_id: q.plan_id,
    percent: q.limits.daily ? Math.floor((q.usage.daily_used / q.limits.daily) * 100) : 0,
    alert: q.alert
  }))
})

// Helper functions
const getPlanName = (planId: string) => {
  const plan = plans.value.find(p => p.id === planId)
  return plan?.name || planId
}

const getStatusType = (statusCode?: number) => {
  if (!statusCode) return 'info'
  if (statusCode >= 200 && statusCode < 300) return 'success'
  if (statusCode === 429) return 'warning'
  return 'danger'
}

const getQuotaColor = (percent: number) => {
  if (percent >= 90) return '#f43f5e'
  if (percent >= 70) return '#f59e0b'
  return '#10b981'
}

const formatTime = (timestamp: string) => {
  if (!timestamp) return '—'
  const date = new Date(timestamp)
  return `${String(date.getHours()).padStart(2, '0')}:${String(date.getMinutes()).padStart(2, '0')}`
}

const getRowClass = ({ row }: { row: RequestLog }) => {
  if (!row.status_code) return ''
  return row.status_code >= 200 && row.status_code < 300 ? 'success-row' : 'error-row'
}

// Data loading
const loadData = async () => {
  loading.plans = true
  loading.quota = true
  loading.logs = true
  loading.health = true

  try {
    const [plansData, quotasData, logsData, health] = await Promise.all([
      fetchPlans(),
      fetchQuotaStatus(),
      fetchLogs(50),
      healthCheck()
    ])

    plans.value = plansData
    quotas.value = quotasData
    logs.value = logsData
    isHealthy.value = health
  } catch (error) {
    console.error('Failed to load dashboard data:', error)
  } finally {
    loading.plans = false
    loading.quota = false
    loading.logs = false
    loading.health = false
  }
}

const loadLogs = async () => {
  loading.logs = true
  try {
    logs.value = await fetchLogs(50)
  } catch (error) {
    console.error('Failed to load logs:', error)
  } finally {
    loading.logs = false
  }
}

onMounted(() => {
  loadData()
})
</script>

<style scoped>
.dashboard {
  animation: fadeIn 0.5s ease;
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(10px); }
  to { opacity: 1; transform: translateY(0); }
}

/* ── Stats Row ── */
.stats-row {
  margin-bottom: 20px;
}

.stat-card {
  position: relative;
  background: rgba(20, 23, 34, 0.7);
  backdrop-filter: blur(20px);
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 14px;
  padding: 20px;
  display: flex;
  align-items: flex-start;
  gap: 16px;
  overflow: hidden;
  transition: all 0.3s ease;
}

.stat-card:hover {
  transform: translateY(-2px);
  border-color: rgba(255, 255, 255, 0.1);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
}

.stat-icon {
  width: 48px;
  height: 48px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.stat-card-cyan .stat-icon {
  background: linear-gradient(135deg, rgba(14, 165, 233, 0.2), rgba(6, 182, 212, 0.15));
  color: #38bdf8;
}

.stat-card-emerald .stat-icon {
  background: linear-gradient(135deg, rgba(16, 185, 129, 0.2), rgba(0, 212, 170, 0.15));
  color: #34d399;
}

.stat-card-sky .stat-icon {
  background: linear-gradient(135deg, rgba(14, 165, 233, 0.15), rgba(34, 211, 238, 0.12));
  color: #22d3ee;
}

.stat-card-rose .stat-icon {
  background: linear-gradient(135deg, rgba(244, 63, 94, 0.2), rgba(239, 68, 68, 0.15));
  color: #fb7185;
}

.stat-content {
  flex: 1;
  min-width: 0;
}

.stat-value {
  font-size: 26px;
  font-weight: 700;
  color: #e8eaf0;
  line-height: 1.2;
}

.stat-label {
  font-size: 12px;
  color: #6b7280;
  margin-top: 4px;
}

.stat-progress {
  margin-top: 10px;
}

.health-tag {
  font-weight: 600;
}

.stat-glow {
  position: absolute;
  top: -30%;
  right: -20%;
  width: 80px;
  height: 80px;
  border-radius: 50%;
  opacity: 0.15;
  pointer-events: none;
}

.stat-card-cyan .stat-glow { background: #0ea5e9; }
.stat-card-emerald .stat-glow { background: #10b981; }
.stat-card-sky .stat-glow { background: #22d3ee; }
.stat-card-rose .stat-glow { background: #f43f5e; }

/* ── Content Row ── */
.content-row {
  margin-bottom: 20px;
}

.content-card {
  background: rgba(20, 23, 34, 0.7);
  backdrop-filter: blur(20px);
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 14px;
  padding: 20px;
  margin-bottom: 20px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.card-title {
  font-size: 15px;
  font-weight: 600;
  color: #e8eaf0;
}

.card-actions {
  display: flex;
  gap: 4px;
}

/* ── Table ── */
.dashboard-table {
  --el-table-header-bg-color: transparent;
}

.dashboard-table :deep(th.el-table__cell) {
  font-weight: 600;
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: #6b7280;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
}

.dashboard-table :deep(td.el-table__cell) {
  font-size: 13px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.04);
}

.dashboard-table :deep(.el-table__row--striped td.el-table__cell) {
  background: rgba(255, 255, 255, 0.02);
}

.plan-name {
  font-weight: 500;
  color: #e8eaf0;
}

.model-badge {
  font-family: var(--agw-font-mono, monospace);
  font-size: 12px;
  background: rgba(255, 255, 255, 0.05);
  padding: 2px 8px;
  border-radius: 4px;
  color: #94a3b8;
}

.status-tag {
  font-family: var(--agw-font-mono, monospace);
  font-weight: 600;
}

.latency-value {
  font-size: 12px;
  color: #94a3b8;
}

.latency-value.latency-high {
  color: #f43f5e;
}

/* ── Quick Actions ── */
.quick-actions {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.action-item {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 14px;
  border-radius: 10px;
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid rgba(255, 255, 255, 0.04);
  cursor: pointer;
  transition: all 0.25s ease;
}

.action-item:hover {
  background: rgba(255, 255, 255, 0.05);
  transform: translateX(3px);
}

.action-item.action-primary {
  background: linear-gradient(135deg, rgba(14, 165, 233, 0.15), rgba(6, 182, 212, 0.1));
  border-color: rgba(14, 165, 233, 0.2);
}

.action-item.action-primary:hover {
  background: linear-gradient(135deg, rgba(14, 165, 233, 0.2), rgba(6, 182, 212, 0.15));
}

.action-icon {
  width: 38px;
  height: 38px;
  border-radius: 9px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(255, 255, 255, 0.05);
  color: #8b92a8;
  flex-shrink: 0;
}

.action-item.action-primary .action-icon {
  background: linear-gradient(135deg, #0ea5e9, #06b6d4);
  color: white;
}

.action-content {
  flex: 1;
  min-width: 0;
}

.action-title {
  font-size: 14px;
  font-weight: 600;
  color: #e8eaf0;
}

.action-desc {
  font-size: 12px;
  color: #6b7280;
  margin-top: 1px;
}

.action-arrow {
  color: #4a5068;
  transition: all 0.25s ease;
}

.action-item:hover .action-arrow {
  transform: translateX(3px);
  color: #0ea5e9;
}

/* ── Quota Overview ── */
.quota-overview {
  margin-bottom: 0;
}

.quota-list {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.quota-item {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.quota-plan {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.quota-plan-name {
  font-size: 13px;
  color: #c4c9d8;
}

.quota-percent {
  font-size: 12px;
  font-weight: 600;
  color: #94a3b8;
}

.alert-icon {
  color: #f43f5e;
  margin-right: 4px;
  font-size: 13px;
}

.text-alert {
  color: #f43f5e;
}

.quota-item.quota-alert {
  padding: 8px 10px;
  background: rgba(244, 63, 94, 0.05);
  border-radius: 8px;
  border: 1px solid rgba(244, 63, 94, 0.1);
}
</style>