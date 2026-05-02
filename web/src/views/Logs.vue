<template>
  <div class="logs-view">
    <div class="logs-header agw-stagger">
      <div class="header-info">
        <h2 class="section-title">请求日志</h2>
        <p class="section-desc">查看最近的 API 请求记录</p>
      </div>
      <div class="header-actions">
        <el-input
          v-model="searchKeyword"
          placeholder="搜索套餐或模型..."
          clearable
          class="search-input"
        >
          <template #prefix>
            <el-icon><Search /></el-icon>
          </template>
        </el-input>
        <el-button @click="loadLogs">
          <el-icon><Refresh /></el-icon>
          刷新
        </el-button>
      </div>
    </div>

    <div class="logs-content">
      <el-table
        :data="filteredLogs"
        class="logs-table"
        :row-class-name="getRowClass"
        v-loading="loading"
        stripe
      >
        <el-table-column prop="timestamp" label="时间" width="160">
          <template #default="{ row }">
            <span class="time-value agw-mono">{{ formatTimestamp(row.timestamp) }}</span>
          </template>
        </el-table-column>
        <el-table-column prop="plan_id" label="套餐" min-width="120">
          <template #default="{ row }">
            <span class="plan-name">{{ getPlanName(row.plan_id) }}</span>
          </template>
        </el-table-column>
        <el-table-column prop="agent_id" label="Agent" width="110">
          <template #default="{ row }">
            <span class="agent-badge">{{ row.agent_id || '—' }}</span>
          </template>
        </el-table-column>
        <el-table-column prop="model_id" label="模型" min-width="120">
          <template #default="{ row }">
            <span class="model-badge agw-mono">{{ row.model_id }}</span>
          </template>
        </el-table-column>
        <el-table-column label="状态" width="90" align="center">
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
        <el-table-column label="延迟" width="100" align="right">
          <template #default="{ row }">
            <span
              class="latency-value agw-mono"
              :class="getLatencyClass(row.latency_ms)"
            >
              {{ row.latency_ms ? `${row.latency_ms}ms` : '—' }}
            </span>
          </template>
        </el-table-column>
        <el-table-column prop="level" label="级别" width="80" align="center">
          <template #default="{ row }">
            <el-tag
              :type="getLevelType(row.level)"
              size="small"
              effect="plain"
            >
              {{ row.level || 'INFO' }}
            </el-tag>
          </template>
        </el-table-column>
      </el-table>

      <div class="logs-footer">
        <span class="logs-count agw-mono">
          共 {{ filteredLogs.length }} 条记录
        </span>
        <el-button-group>
          <el-button size="small" :disabled="logs.length < 50" @click="loadLogs(100)">
            更多
          </el-button>
          <el-button size="small" @click="loadLogs">
            <el-icon><Refresh /></el-icon>
            刷新
          </el-button>
        </el-button-group>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { ElMessage } from 'element-plus'
import { fetchLogs, fetchPlans } from '@/api'
import type { RequestLog, UserPlan } from '@/types'

const logs = ref<RequestLog[]>([])
const plans = ref<UserPlan[]>([])
const loading = ref(false)
const searchKeyword = ref('')

// Filtered logs based on search
const filteredLogs = computed(() => {
  if (!searchKeyword.value) return logs.value
  const keyword = searchKeyword.value.toLowerCase()
  return logs.value.filter(log =>
    log.plan_id.toLowerCase().includes(keyword) ||
    log.model_id.toLowerCase().includes(keyword) ||
    log.agent_id?.toLowerCase().includes(keyword)
  )
})

// Load logs
const loadLogs = async (limit = 50) => {
  loading.value = true
  try {
    const [logsData, plansData] = await Promise.all([
      fetchLogs(limit),
      fetchPlans()
    ])
    logs.value = logsData
    plans.value = plansData
  } catch {
    ElMessage.error('加载日志失败')
  } finally {
    loading.value = false
  }
}

// Helper functions
const getPlanName = (planId: string) => {
  const plan = plans.value.find(p => p.id === planId)
  return plan?.name || planId
}

const getStatusType = (statusCode?: number) => {
  if (!statusCode) return 'info'
  if (statusCode === 200) return 'success'
  if (statusCode === 429) return 'warning'
  if (statusCode >= 400) return 'danger'
  return 'info'
}

const getLatencyClass = (latency?: number) => {
  if (!latency) return ''
  if (latency > 2000) return 'latency-critical'
  if (latency > 1000) return 'latency-high'
  if (latency > 500) return 'latency-medium'
  return ''
}

const getLevelType = (level?: string) => {
  const types: Record<string, string> = {
    DEBUG: 'info',
    INFO: 'primary',
    WARN: 'warning',
    ERROR: 'danger'
  }
  return types[level || 'INFO'] || 'info'
}

const formatTimestamp = (timestamp: string) => {
  if (!timestamp) return '—'
  const date = new Date(timestamp)
  return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}-${String(date.getDate()).padStart(2, '0')} ${String(date.getHours()).padStart(2, '0')}:${String(date.getMinutes()).padStart(2, '0')}:${String(date.getSeconds()).padStart(2, '0')}`
}

const getRowClass = ({ row }: { row: RequestLog }) => {
  if (!row.status_code) return ''
  if (row.status_code >= 200 && row.status_code < 400) return 'success-row'
  return 'error-row'
}

onMounted(() => {
  loadLogs()
})
</script>

<style scoped>
.logs-view {
  animation: fadeIn 0.5s ease;
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(10px); }
  to { opacity: 1; transform: translateY(0); }
}

.logs-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
  padding: 18px 24px;
  background: var(--agw-bg-card);
  backdrop-filter: blur(20px);
  border: 1px solid var(--agw-border-default);
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
  color: var(--agw-text-primary);
  margin: 0;
}

.section-desc {
  font-size: 13px;
  color: var(--agw-text-secondary);
  margin: 0;
}

.header-actions {
  display: flex;
  gap: 10px;
}

.search-input {
  width: 220px;
}

.logs-content {
  background: var(--agw-bg-card);
  backdrop-filter: blur(20px);
  border: 1px solid var(--agw-border-default);
  border-radius: 14px;
  overflow: hidden;
}

.logs-table {
  --el-table-header-bg-color: transparent;
  border-radius: 14px;
}

.logs-table :deep(th.el-table__cell) {
  font-weight: 600;
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: var(--agw-text-secondary);
  border-bottom: 1px solid var(--agw-border-subtle);
}

.logs-table :deep(td.el-table__cell) {
  font-size: 13px;
  border-bottom: 1px solid var(--agw-border-subtle);
}

.logs-table :deep(.el-table__row--striped td.el-table__cell) {
  background: var(--agw-bg-hover);
}

.time-value {
  font-size: 12px;
  color: var(--agw-text-secondary);
}

.plan-name {
  font-weight: 500;
  color: var(--agw-text-primary);
}

.agent-badge {
  font-size: 12px;
  background: var(--agw-bg-hover);
  padding: 2px 8px;
  border-radius: 4px;
  color: var(--agw-text-secondary);
}

.model-badge {
  font-size: 11px;
  background: var(--agw-sky-dim);
  padding: 2px 8px;
  border-radius: 4px;
  color: #38bdf8;
}

.status-tag {
  font-family: var(--agw-font-mono, monospace);
  font-weight: 600;
}

.latency-value {
  font-size: 12px;
  color: var(--agw-text-secondary);
}

.latency-medium { color: #f59e0b; }
.latency-high { color: #f97316; }
.latency-critical { color: #f43f5e; }

.logs-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  border-top: 1px solid var(--agw-border-subtle);
}

.logs-count {
  font-size: 12px;
  color: var(--agw-text-secondary);
}
</style>