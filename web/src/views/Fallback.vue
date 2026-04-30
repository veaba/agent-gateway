<template>
  <div class="fallback-view">
    <!-- Header -->
    <div class="fallback-header agw-stagger">
      <div class="header-info">
        <h2 class="section-title">降级策略</h2>
        <p class="section-desc">配置自动降级策略，监控降级事件和 Provider 性能</p>
      </div>
      <div class="header-actions">
        <el-button @click="loadAllData">
          <el-icon><Refresh /></el-icon>
          刷新
        </el-button>
      </div>
    </div>

    <!-- Stats Overview -->
    <el-row :gutter="16" class="stats-row">
      <el-col :xs="12" :sm="6">
        <div class="stat-card stat-card-rose">
          <div class="stat-icon">
            <el-icon :size="24"><Warning /></el-icon>
          </div>
          <div class="stat-content">
            <div class="stat-value agw-mono">{{ stats.totalEvents }}</div>
            <div class="stat-label">降级总次数</div>
          </div>
          <div class="stat-glow"></div>
        </div>
      </el-col>
      <el-col :xs="12" :sm="6">
        <div class="stat-card stat-card-emerald">
          <div class="stat-icon">
            <el-icon :size="24"><CircleCheck /></el-icon>
          </div>
          <div class="stat-content">
            <div class="stat-value agw-mono">{{ stats.totalResolved }}</div>
            <div class="stat-label">已恢复</div>
          </div>
          <div class="stat-glow"></div>
        </div>
      </el-col>
      <el-col :xs="12" :sm="6">
        <div class="stat-card stat-card-amber">
          <div class="stat-icon">
            <el-icon :size="24"><Clock /></el-icon>
          </div>
          <div class="stat-content">
            <div class="stat-value agw-mono">{{ stats.totalUnresolved }}</div>
            <div class="stat-label">未恢复</div>
          </div>
          <div class="stat-glow"></div>
        </div>
      </el-col>
      <el-col :xs="12" :sm="6">
        <div class="stat-card stat-card-blue">
          <div class="stat-icon">
            <el-icon :size="24"><Timer /></el-icon>
          </div>
          <div class="stat-content">
            <div class="stat-value agw-mono">
              {{ stats.avgRecoveryLatencyMs ? Math.round(stats.avgRecoveryLatencyMs) + 'ms' : '—' }}
            </div>
            <div class="stat-label">平均恢复延迟</div>
          </div>
          <div class="stat-glow"></div>
        </div>
      </el-col>
    </el-row>

    <!-- Config + Trigger Distribution -->
    <el-row :gutter="20" class="content-row">
      <!-- Config Card -->
      <el-col :xs="24" :lg="10">
        <div class="content-card">
          <div class="card-header">
            <span class="card-title">降级配置</span>
            <el-tag :type="config.enabled ? 'success' : 'info'" size="small" round effect="dark">
              {{ config.enabled ? '已启用' : '已禁用' }}
            </el-tag>
          </div>

          <el-form label-width="120px" class="config-form">
            <el-form-item label="启用自动降级">
              <el-switch v-model="config.enabled" />
            </el-form-item>

            <el-form-item label="最大重试次数">
              <el-input-number v-model="config.max_attempts" :min="1" :max="5" />
            </el-form-item>

            <el-form-item label="优先级顺序">
              <el-select v-model="config.priority_order" multiple placeholder="拖拽排序选择套餐" class="priority-select">
                <el-option v-for="plan in plans" :key="plan.id" :label="plan.name" :value="plan.id" />
              </el-select>
              <div class="priority-hint" v-if="config.priority_order.length > 0">
                <el-icon :size="12"><InfoFilled /></el-icon>
                排列顺序即为降级优先级，首选项为最优先
              </div>
            </el-form-item>

            <el-form-item>
              <el-button type="primary" @click="handleSave" :loading="saving">保存配置</el-button>
            </el-form-item>
          </el-form>
        </div>
      </el-col>

      <!-- Trigger Type Distribution -->
      <el-col :xs="24" :lg="14">
        <div class="content-card">
          <div class="card-header">
            <span class="card-title">触发原因分布</span>
          </div>

          <div v-if="stats.byTriggerType.length > 0" class="trigger-distribution">
            <div v-for="item in stats.byTriggerType" :key="item.triggerType" class="trigger-item">
              <div class="trigger-item-header">
                <div class="trigger-item-left">
                  <span class="trigger-dot" :style="{ backgroundColor: getTriggerColor(item.triggerType) }"></span>
                  <span class="trigger-label">{{ formatTriggerType(item.triggerType) }}</span>
                </div>
                <div class="trigger-item-right">
                  <span class="trigger-count agw-mono">{{ item.count }}</span>
                  <span class="trigger-percent agw-mono">
                    {{ stats.totalEvents > 0 ? ((item.count / stats.totalEvents) * 100).toFixed(1) : 0 }}%
                  </span>
                </div>
              </div>
              <el-progress
                :percentage="stats.totalEvents > 0 ? Math.round((item.count / stats.totalEvents) * 100) : 0"
                :stroke-width="8"
                :show-text="false"
                :color="getTriggerColor(item.triggerType)"
              />
            </div>
          </div>
          <el-empty v-else-if="!loading.stats" description="暂无降级事件数据" :image-size="60" />
        </div>
      </el-col>
    </el-row>

    <!-- Provider Performance -->
    <div class="content-card" v-if="providerPerformance.length > 0">
      <div class="card-header">
        <span class="card-title">Provider 性能指标</span>
      </div>
      <el-table :data="providerPerformance" class="stats-table" v-loading="loading.performance" stripe>
        <el-table-column label="Provider" min-width="140">
          <template #default="{ row }">
            <div class="provider-cell">
              <div class="provider-name">{{ row.providerName }}</div>
              <div class="provider-id agw-mono">{{ row.providerId }}</div>
            </div>
          </template>
        </el-table-column>
        <el-table-column label="健康分" width="90" align="center">
          <template #default="{ row }">
            <el-tag
              :type="row.healthScore >= 80 ? 'success' : row.healthScore >= 60 ? 'warning' : 'danger'"
              size="small"
              round
              class="rate-tag"
            >
              {{ Math.round(row.healthScore) }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column label="降级率" width="90" align="center">
          <template #default="{ row }">
            <span class="agw-mono" :class="{ 'text-rose': row.fallbackRate > 0.1, 'text-amber': row.fallbackRate > 0.05 }">
              {{ (row.fallbackRate * 100).toFixed(1) }}%
            </span>
          </template>
        </el-table-column>
        <el-table-column label="成功率" width="90" align="center">
          <template #default="{ row }">
            <el-tag
              :type="row.successRate >= 0.95 ? 'success' : row.successRate >= 0.9 ? 'warning' : 'danger'"
              size="small"
              round
              class="rate-tag"
            >
              {{ (row.successRate * 100).toFixed(1) }}%
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column label="平均延迟" width="100" align="right">
          <template #default="{ row }">
            <span class="latency-value agw-mono" :class="{ 'latency-high': row.avgLatencyMs > 500 }">
              {{ Math.round(row.avgLatencyMs) }}ms
            </span>
          </template>
        </el-table-column>
        <el-table-column label="降级次数" width="90" align="right">
          <template #default="{ row }">
            <span class="agw-mono">{{ row.fallbackEvents }}</span>
          </template>
        </el-table-column>
        <el-table-column label="预计恢复" width="110" align="right">
          <template #default="{ row }">
            <span v-if="row.estimatedRecoveryTimeMs" class="agw-mono">
              {{ Math.round(row.estimatedRecoveryTimeMs / 1000) }}s
            </span>
            <span v-else class="text-muted">—</span>
          </template>
        </el-table-column>
        <el-table-column label="最近降级" width="160" align="right">
          <template #default="{ row }">
            <span v-if="row.lastFallbackAt" class="agw-mono time-value">
              {{ formatTime(row.lastFallbackAt) }}
            </span>
            <span v-else class="text-muted">—</span>
          </template>
        </el-table-column>
      </el-table>
    </div>

    <!-- Fallback Events -->
    <div class="content-card">
      <div class="card-header">
        <span class="card-title">降级事件记录</span>
        <div class="card-filters">
          <el-select v-model="filterPlanId" placeholder="筛选套餐" clearable size="small" class="filter-select">
            <el-option v-for="plan in plans" :key="plan.id" :label="plan.name" :value="plan.id" />
          </el-select>
          <el-input-number v-model="eventLimit" :min="20" :max="500" size="small" class="limit-input" />
        </div>
      </div>

      <el-table :data="events" class="stats-table" v-loading="loading.events" stripe>
        <el-table-column label="时间" width="160">
          <template #default="{ row }">
            <span class="time-value agw-mono">{{ formatTime(row.triggeredAt) }}</span>
          </template>
        </el-table-column>
        <el-table-column label="触发原因" width="120">
          <template #default="{ row }">
            <el-tag
              :type="getTriggerTagType(row.triggerType)"
              size="small"
              effect="dark"
              round
            >
              {{ formatTriggerType(row.triggerType) }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column label="状态码" width="80" align="center">
          <template #default="{ row }">
            <span v-if="row.triggerCode" class="agw-mono">{{ row.triggerCode }}</span>
            <span v-else class="text-muted">—</span>
          </template>
        </el-table-column>
        <el-table-column label="源套餐" min-width="120">
          <template #default="{ row }">
            <span class="plan-name">{{ getPlanName(row.sourcePlanId) }}</span>
          </template>
        </el-table-column>
        <el-table-column label="目标套餐" min-width="120">
          <template #default="{ row }">
            <span v-if="row.targetPlanId" class="plan-name">{{ getPlanName(row.targetPlanId) }}</span>
            <span v-else class="text-muted">—</span>
          </template>
        </el-table-column>
        <el-table-column label="协议转换" width="90" align="center">
          <template #default="{ row }">
            <el-tag v-if="row.protocolConverted" type="warning" size="small" round effect="plain">是</el-tag>
            <span v-else class="text-muted">否</span>
          </template>
        </el-table-column>
        <el-table-column label="延迟" width="90" align="right">
          <template #default="{ row }">
            <span v-if="row.latencyMs" class="latency-value agw-mono" :class="{ 'latency-high': row.latencyMs > 1000 }">
              {{ row.latencyMs }}ms
            </span>
            <span v-else class="text-muted">—</span>
          </template>
        </el-table-column>
        <el-table-column label="状态" width="90" align="center">
          <template #default="{ row }">
            <el-tag v-if="row.resolved" type="success" size="small" round>已恢复</el-tag>
            <el-tag v-else type="danger" size="small" round effect="dark">未恢复</el-tag>
          </template>
        </el-table-column>
        <el-table-column label="恢复时间" width="160">
          <template #default="{ row }">
            <span v-if="row.recoveredAt" class="time-value agw-mono">{{ formatTime(row.recoveredAt) }}</span>
            <span v-else class="text-muted">—</span>
          </template>
        </el-table-column>
        <el-table-column label="错误信息" min-width="180">
          <template #default="{ row }">
            <el-tooltip v-if="row.errorMessage" :content="row.errorMessage" placement="top">
              <span class="error-msg">{{ truncateError(row.errorMessage) }}</span>
            </el-tooltip>
            <span v-else class="text-muted">—</span>
          </template>
        </el-table-column>
      </el-table>

      <div class="table-footer" v-if="events.length > 0">
        <span class="records-count agw-mono">共 {{ events.length }} 条记录</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, watch } from 'vue'
import { ElMessage } from 'element-plus'
import type { FallbackConfig, FallbackStats, FallbackEvent, ProviderPerformance, UserPlan } from '@/types'
import {
  fetchFallbackConfig, updateFallbackConfig, fetchPlans,
  fetchFallbackEvents, fetchFallbackStats, fetchFallbackPerformance
} from '@/api'

const config = ref<FallbackConfig>({
  enabled: true,
  max_attempts: 3,
  priority_order: []
})
const plans = ref<UserPlan[]>([])
const events = ref<FallbackEvent[]>([])
const stats = ref<FallbackStats>({
  totalEvents: 0,
  totalResolved: 0,
  totalUnresolved: 0,
  avgRecoveryLatencyMs: undefined,
  byTriggerType: []
})
const providerPerformance = ref<ProviderPerformance[]>([])

const saving = ref(false)
const loading = reactive({
  config: false,
  events: false,
  stats: false,
  performance: false
})

const filterPlanId = ref<string>('')
const eventLimit = ref(100)

const loadConfig = async () => {
  loading.config = true
  try {
    const [fallbackConfig, planList] = await Promise.all([
      fetchFallbackConfig(),
      fetchPlans()
    ])
    config.value = fallbackConfig
    plans.value = planList
  } catch {
    ElMessage.error('加载配置失败')
  } finally {
    loading.config = false
  }
}

const loadEvents = async () => {
  loading.events = true
  try {
    events.value = await fetchFallbackEvents(filterPlanId.value || undefined, undefined, eventLimit.value)
  } catch {
    ElMessage.error('加载事件失败')
  } finally {
    loading.events = false
  }
}

const loadStats = async () => {
  loading.stats = true
  try {
    stats.value = await fetchFallbackStats()
  } catch {
    ElMessage.error('加载统计失败')
  } finally {
    loading.stats = false
  }
}

const loadPerformance = async () => {
  loading.performance = true
  try {
    providerPerformance.value = await fetchFallbackPerformance()
  } catch {
    ElMessage.error('加载性能数据失败')
  } finally {
    loading.performance = false
  }
}

const loadAllData = () => {
  loadConfig()
  loadEvents()
  loadStats()
  loadPerformance()
}

const handleSave = async () => {
  saving.value = true
  try {
    await updateFallbackConfig(config.value)
    ElMessage.success('保存成功')
  } catch {
    ElMessage.error('保存失败')
  } finally {
    saving.value = false
  }
}

watch([filterPlanId, eventLimit], () => {
  loadEvents()
})

const formatTriggerType = (type: string) => {
  const map: Record<string, string> = {
    rate_limit: '速率限制',
    server_error: '服务端错误',
    connection_failure: '连接失败',
    timeout: '请求超时',
    quota_exceeded: '配额耗尽'
  }
  return map[type] || type
}

const getTriggerColor = (type: string) => {
  const map: Record<string, string> = {
    rate_limit: '#f59e0b',
    server_error: '#f43f5e',
    connection_failure: '#8b5cf6',
    timeout: '#06b6d4',
    quota_exceeded: '#10b981'
  }
  return map[type] || '#6b7280'
}

const getTriggerTagType = (type: string) => {
  const map: Record<string, string> = {
    rate_limit: 'warning',
    server_error: 'danger',
    connection_failure: '',
    timeout: 'info',
    quota_exceeded: 'success'
  }
  return (map[type] || 'info') as '' | 'success' | 'warning' | 'danger' | 'info'
}

const getPlanName = (planId: string) => {
  const plan = plans.value.find(p => p.id === planId)
  return plan?.name || planId
}

const formatTime = (timestamp: string) => {
  if (!timestamp) return '—'
  const date = new Date(timestamp)
  const month = String(date.getMonth() + 1).padStart(2, '0')
  const day = String(date.getDate()).padStart(2, '0')
  const hour = String(date.getHours()).padStart(2, '0')
  const minute = String(date.getMinutes()).padStart(2, '0')
  const second = String(date.getSeconds()).padStart(2, '0')
  return `${month}-${day} ${hour}:${minute}:${second}`
}

const truncateError = (msg: string) => {
  return msg.length > 40 ? msg.slice(0, 40) + '...' : msg
}

onMounted(() => {
  loadAllData()
})
</script>

<style scoped>
.fallback-view {
  animation: fadeIn 0.5s ease;
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(10px); }
  to { opacity: 1; transform: translateY(0); }
}

.fallback-header {
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
  padding: 18px;
  display: flex;
  align-items: flex-start;
  gap: 14px;
  overflow: hidden;
  transition: all 0.3s ease;
  margin-bottom: 16px;
}

.stat-card:hover {
  transform: translateY(-2px);
  border-color: rgba(255, 255, 255, 0.1);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
}

.stat-icon {
  width: 44px;
  height: 44px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.stat-card-rose .stat-icon { background: linear-gradient(135deg, rgba(244, 63, 94, 0.2), rgba(239, 68, 68, 0.15)); color: #fb7185; }
.stat-card-emerald .stat-icon { background: linear-gradient(135deg, rgba(16, 185, 129, 0.2), rgba(0, 212, 170, 0.15)); color: #34d399; }
.stat-card-amber .stat-icon { background: linear-gradient(135deg, rgba(245, 158, 11, 0.2), rgba(217, 119, 6, 0.15)); color: #fbbf24; }
.stat-card-blue .stat-icon { background: linear-gradient(135deg, rgba(59, 130, 246, 0.2), rgba(37, 99, 235, 0.15)); color: #60a5fa; }

.stat-content { flex: 1; min-width: 0; }
.stat-value { font-size: 22px; font-weight: 700; color: #e8eaf0; line-height: 1.2; }
.stat-label { font-size: 12px; color: #6b7280; margin-top: 4px; }

.stat-glow {
  position: absolute;
  top: -30%;
  right: -20%;
  width: 70px;
  height: 70px;
  border-radius: 50%;
  opacity: 0.12;
  pointer-events: none;
}

.stat-card-rose .stat-glow { background: #f43f5e; }
.stat-card-emerald .stat-glow { background: #10b981; }
.stat-card-amber .stat-glow { background: #f59e0b; }
.stat-card-blue .stat-glow { background: #3b82f6; }

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

.card-filters {
  display: flex;
  gap: 10px;
  align-items: center;
}

.filter-select {
  width: 160px;
}

.limit-input {
  width: 110px;
}

/* ── Config Form ── */
.config-form {
  margin-top: 8px;
}

.priority-select {
  width: 100%;
}

.priority-hint {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 11px;
  color: #6b7280;
  margin-top: 4px;
}

/* ── Trigger Distribution ── */
.trigger-distribution {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.trigger-item {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.trigger-item-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.trigger-item-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.trigger-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}

.trigger-label {
  font-size: 13px;
  color: #c4c9d8;
}

.trigger-item-right {
  display: flex;
  align-items: center;
  gap: 8px;
}

.trigger-count {
  font-size: 14px;
  font-weight: 600;
  color: #e8eaf0;
}

.trigger-percent {
  font-size: 12px;
  color: #6b7280;
}

/* ── Table ── */
.stats-table {
  --el-table-header-bg-color: transparent;
}

.stats-table :deep(th.el-table__cell) {
  font-weight: 600;
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: #6b7280;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
}

.stats-table :deep(td.el-table__cell) {
  font-size: 13px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.04);
}

.stats-table :deep(.el-table__row--striped td.el-table__cell) {
  background: rgba(255, 255, 255, 0.02);
}

.provider-cell {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.provider-name { font-weight: 500; color: #e8eaf0; }
.provider-id { font-size: 11px; color: #4a5068; }
.plan-name { font-weight: 500; color: #e8eaf0; }

.rate-tag {
  font-family: var(--agw-font-mono, monospace);
  font-weight: 600;
}

.latency-value { font-size: 12px; color: #94a3b8; }
.latency-value.latency-high { color: #f43f5e; }

.time-value { font-size: 12px; color: #94a3b8; }

.text-rose { color: #fb7185; }
.text-amber { color: #fbbf24; }
.text-emerald { color: #34d399; }
.text-muted { color: #4a5068; }

.error-msg {
  font-size: 12px;
  color: #f87171;
  cursor: help;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 160px;
  display: inline-block;
}

.table-footer {
  display: flex;
  justify-content: flex-end;
  padding: 12px 0 0;
  border-top: 1px solid rgba(255, 255, 255, 0.05);
  margin-top: 8px;
}

.records-count {
  font-size: 12px;
  color: #6b7280;
}
</style>