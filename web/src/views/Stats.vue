<template>
  <div class="stats-page">
    <!-- Global Stats Cards -->
    <el-row :gutter="20" class="stats-row">
      <el-col :xs="24" :sm="12" :lg="8" :xl="6">
        <div class="stat-card stat-card-blue">
          <div class="stat-icon">
            <el-icon :size="28"><TrendCharts /></el-icon>
          </div>
          <div class="stat-content">
            <div class="stat-value agw-mono">{{ formatNumber(globalStats.totalRequests) }}</div>
            <div class="stat-label">总请求数</div>
          </div>
          <div class="stat-glow"></div>
        </div>
      </el-col>
      <el-col :xs="24" :sm="12" :lg="8" :xl="6">
        <div class="stat-card stat-card-emerald">
          <div class="stat-icon">
            <el-icon :size="28"><CircleCheck /></el-icon>
          </div>
          <div class="stat-content">
            <div class="stat-value agw-mono">{{ formatPercent(globalStats.successRate) }}</div>
            <div class="stat-label">成功率</div>
            <el-progress
              :percentage="Math.round(globalStats.successRate * 100)"
              :stroke-width="4"
              :show-text="false"
              class="stat-progress"
              :color="globalStats.successRate >= 0.95 ? '#10b981' : globalStats.successRate >= 0.9 ? '#f59e0b' : '#f43f5e'"
            />
          </div>
          <div class="stat-glow"></div>
        </div>
      </el-col>
      <el-col :xs="24" :sm="12" :lg="8" :xl="6">
        <div class="stat-card stat-card-amber">
          <div class="stat-icon">
            <el-icon :size="28"><Timer /></el-icon>
          </div>
          <div class="stat-content">
            <div class="stat-value agw-mono">{{ Math.round(globalStats.avgLatencyMs) }}<span class="stat-unit">ms</span></div>
            <div class="stat-label">平均延迟</div>
          </div>
          <div class="stat-glow"></div>
        </div>
      </el-col>
      <el-col :xs="24" :sm="12" :lg="8" :xl="6">
        <div class="stat-card stat-card-rose">
          <div class="stat-icon">
            <el-icon :size="28"><Warning /></el-icon>
          </div>
          <div class="stat-content">
            <div class="stat-value agw-mono">{{ formatNumber(globalStats.totalErrors) }}</div>
            <div class="stat-label">错误数</div>
          </div>
          <div class="stat-glow"></div>
        </div>
      </el-col>
      <el-col :xs="24" :sm="12" :lg="8" :xl="6">
        <div class="stat-card stat-card-purple">
          <div class="stat-icon">
            <el-icon :size="28"><Connection /></el-icon>
          </div>
          <div class="stat-content">
            <div class="stat-value agw-mono">{{ globalStats.plansCount }}</div>
            <div class="stat-label">已配置套餐</div>
          </div>
          <div class="stat-glow"></div>
        </div>
      </el-col>
      <el-col :xs="24" :sm="12" :lg="8" :xl="6">
        <div class="stat-card stat-card-sky">
          <div class="stat-icon">
            <el-icon :size="28"><Box /></el-icon>
          </div>
          <div class="stat-content">
            <div class="stat-value agw-mono">{{ globalStats.providersCount }}</div>
            <div class="stat-label">Provider 数量</div>
          </div>
          <div class="stat-glow"></div>
        </div>
      </el-col>
      <el-col :xs="24" :sm="12" :lg="8" :xl="6">
        <div class="stat-card stat-card-cyan">
          <div class="stat-icon">
            <el-icon :size="28"><User /></el-icon>
          </div>
          <div class="stat-content">
            <div class="stat-value agw-mono">{{ globalStats.activeAgents }}</div>
            <div class="stat-label">活跃 Agents</div>
          </div>
          <div class="stat-glow"></div>
        </div>
      </el-col>
      <el-col :xs="24" :sm="12" :lg="8" :xl="6">
        <div class="stat-card stat-card-indigo">
          <div class="stat-icon">
            <el-icon :size="28"><DataLine /></el-icon>
          </div>
          <div class="stat-content">
            <div class="stat-value agw-mono">{{ formatNumber(globalStats.totalInputTokens + globalStats.totalOutputTokens) }}</div>
            <div class="stat-label">总 Token 数</div>
          </div>
          <div class="stat-glow"></div>
        </div>
      </el-col>
    </el-row>

    <!-- Fallback Stats Row -->
    <el-row :gutter="20" class="content-row">
      <el-col :xs="24" :lg="8">
        <div class="content-card">
          <div class="card-header">
            <span class="card-title">Fallback 降级统计</span>
            <el-button text size="small" @click="loadFallbackData">
              <el-icon><Refresh /></el-icon>
              刷新
            </el-button>
          </div>
          <div v-loading="loading.fallback" class="fallback-stats">
            <el-row :gutter="12">
              <el-col :span="8">
                <div class="mini-stat">
                  <div class="mini-stat-value agw-mono text-rose">{{ fallbackStats.totalEvents }}</div>
                  <div class="mini-stat-label">总降级次数</div>
                </div>
              </el-col>
              <el-col :span="8">
                <div class="mini-stat">
                  <div class="mini-stat-value agw-mono text-emerald">{{ fallbackStats.totalResolved }}</div>
                  <div class="mini-stat-label">已恢复</div>
                </div>
              </el-col>
              <el-col :span="8">
                <div class="mini-stat">
                  <div class="mini-stat-value agw-mono text-amber">{{ fallbackStats.totalUnresolved }}</div>
                  <div class="mini-stat-label">未恢复</div>
                </div>
              </el-col>
            </el-row>
            <div class="trigger-type-section" v-if="fallbackStats.byTriggerType.length > 0">
              <div class="section-subtitle">触发原因分布</div>
              <div class="trigger-bars">
                <div v-for="item in fallbackStats.byTriggerType" :key="item.triggerType" class="trigger-bar-item">
                  <div class="trigger-bar-header">
                    <span class="trigger-label">{{ formatTriggerType(item.triggerType) }}</span>
                    <span class="trigger-count agw-mono">{{ item.count }}</span>
                  </div>
                  <el-progress
                    :percentage="Math.round((item.count / Math.max(fallbackStats.totalEvents, 1)) * 100)"
                    :stroke-width="6"
                    :show-text="false"
                    :color="getTriggerColor(item.triggerType)"
                  />
                </div>
              </div>
            </div>
            <el-empty v-else-if="fallbackStats.totalEvents === 0 && !loading.fallback" description="暂无降级事件" :image-size="60" />
          </div>
        </div>
      </el-col>

      <el-col :xs="24" :lg="16">
        <div class="content-card">
          <div class="card-header">
            <span class="card-title">Provider 性能指标</span>
            <el-button text size="small" @click="loadFallbackData">
              <el-icon><Refresh /></el-icon>
              刷新
            </el-button>
          </div>
          <el-table
            :data="providerPerformance"
            class="stats-table"
            v-loading="loading.fallback"
            stripe
          >
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
          </el-table>
          <el-empty v-if="providerPerformance.length === 0 && !loading.fallback" description="暂无 Provider 性能数据" :image-size="60" />
        </div>
      </el-col>
    </el-row>

    <!-- Main Content: Provider Stats + Usage Trend -->
    <el-row :gutter="20" class="content-row">
      <!-- Provider Stats Table -->
      <el-col :xs="24" :lg="16">
        <div class="content-card">
          <div class="card-header">
            <span class="card-title">Provider 统计</span>
            <el-button text size="small" @click="loadData">
              <el-icon><Refresh /></el-icon>
              刷新
            </el-button>
          </div>
          <el-table
            :data="providerStats"
            class="stats-table"
            v-loading="loading.providers"
            stripe
          >
            <el-table-column label="Provider" min-width="140">
              <template #default="{ row }">
                <div class="provider-cell">
                  <div class="provider-name">{{ row.providerName }}</div>
                  <div class="provider-id agw-mono">{{ row.providerId }}</div>
                </div>
              </template>
            </el-table-column>
            <el-table-column label="套餐数" width="90" align="center">
              <template #default="{ row }">
                <span class="agw-mono">{{ row.plansCount }}</span>
              </template>
            </el-table-column>
            <el-table-column label="请求数" width="100" align="right">
              <template #default="{ row }">
                <span class="agw-mono">{{ formatNumber(row.totalRequests) }}</span>
              </template>
            </el-table-column>
            <el-table-column label="成功率" width="110" align="center">
              <template #default="{ row }">
                <el-tag
                  :type="row.successRate >= 0.95 ? 'success' : row.successRate >= 0.9 ? 'warning' : 'danger'"
                  size="small"
                  round
                  class="rate-tag"
                >
                  {{ formatPercent(row.successRate) }}
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
          </el-table>
          <el-empty v-if="providerStats.length === 0 && !loading.providers" description="暂无 Provider 数据" :image-size="60" />
        </div>

        <!-- Plan Stats Selector -->
        <div class="content-card">
          <div class="card-header">
            <span class="card-title">套餐详细统计</span>
            <el-select
              v-model="selectedPlanId"
              placeholder="选择套餐"
              size="small"
              class="plan-select"
              clearable
              @change="loadPlanStats"
            >
              <el-option
                v-for="plan in plans"
                :key="plan.id"
                :label="plan.name"
                :value="plan.id"
              />
            </el-select>
          </div>

          <div v-if="selectedPlanStats && !loading.plan" class="plan-stats-detail">
            <el-row :gutter="16">
              <el-col :span="8">
                <div class="mini-stat">
                  <div class="mini-stat-value agw-mono">{{ formatNumber(selectedPlanStats.totalRequests) }}</div>
                  <div class="mini-stat-label">总请求</div>
                </div>
              </el-col>
              <el-col :span="8">
                <div class="mini-stat">
                  <div class="mini-stat-value agw-mono">{{ formatPercent(selectedPlanStats.successRate) }}</div>
                  <div class="mini-stat-label">成功率</div>
                </div>
              </el-col>
              <el-col :span="8">
                <div class="mini-stat">
                  <div class="mini-stat-value agw-mono">{{ Math.round(selectedPlanStats.avgLatencyMs) }}ms</div>
                  <div class="mini-stat-label">平均延迟</div>
                </div>
              </el-col>
            </el-row>

            <div class="quota-detail-section">
              <div class="section-subtitle">配额使用</div>
              <div class="quota-bars">
                <div class="quota-bar-item">
                  <div class="quota-bar-header">
                    <span>日配额</span>
                    <span class="quota-bar-value agw-mono">
                      {{ selectedPlanStats.quotaUsage.dailyUsed }}
                      <span v-if="selectedPlanStats.quotaUsage.dailyLimit">/ {{ selectedPlanStats.quotaUsage.dailyLimit }}</span>
                    </span>
                  </div>
                  <el-progress
                    :percentage="Math.round(selectedPlanStats.quotaUsage.dailyPercent * 100)"
                    :stroke-width="8"
                    :color="getQuotaColor(selectedPlanStats.quotaUsage.dailyPercent * 100)"
                    :show-text="false"
                  />
                </div>
                <div class="quota-bar-item">
                  <div class="quota-bar-header">
                    <span>月配额</span>
                    <span class="quota-bar-value agw-mono">
                      {{ selectedPlanStats.quotaUsage.monthlyUsed }}
                      <span v-if="selectedPlanStats.quotaUsage.monthlyLimit">/ {{ selectedPlanStats.quotaUsage.monthlyLimit }}</span>
                    </span>
                  </div>
                  <el-progress
                    :percentage="Math.round(selectedPlanStats.quotaUsage.monthlyPercent * 100)"
                    :stroke-width="8"
                    :color="getQuotaColor(selectedPlanStats.quotaUsage.monthlyPercent * 100)"
                    :show-text="false"
                  />
                </div>
                <div class="quota-bar-item">
                  <div class="quota-bar-header">
                    <span>RPM</span>
                    <span class="quota-bar-value agw-mono">
                      {{ selectedPlanStats.quotaUsage.rpmUsed }}
                      <span v-if="selectedPlanStats.quotaUsage.rpmLimit">/ {{ selectedPlanStats.quotaUsage.rpmLimit }}</span>
                    </span>
                  </div>
                  <el-progress
                    :percentage="Math.round(selectedPlanStats.quotaUsage.rpmPercent * 100)"
                    :stroke-width="8"
                    :color="getQuotaColor(selectedPlanStats.quotaUsage.rpmPercent * 100)"
                    :show-text="false"
                  />
                </div>
              </div>
            </div>
          </div>

          <el-empty v-else-if="!selectedPlanId" description="请选择一个套餐查看详细统计" :image-size="60" />
          <el-skeleton v-else-if="loading.plan" :rows="4" animated />
        </div>
      </el-col>

      <!-- Usage Trend -->
      <el-col :xs="24" :lg="8">
        <div class="content-card">
          <div class="card-header">
            <span class="card-title">使用趋势</span>
            <el-radio-group v-model="granularity" size="small" @change="loadUsageTrend">
              <el-radio-button value="minute">分钟</el-radio-button>
              <el-radio-button value="hour">小时</el-radio-button>
              <el-radio-button value="day">天</el-radio-button>
            </el-radio-group>
          </div>

          <div v-loading="loading.trend" class="trend-content">
            <div v-if="usageTrend.points.length > 0" class="trend-chart">
              <div
                v-for="(point, idx) in visibleTrendPoints"
                :key="idx"
                class="trend-bar-group"
              >
                <div class="trend-bar-wrapper">
                  <div
                    class="trend-bar trend-bar-requests"
                    :style="{ height: getBarHeight(point.requests) + '%' }"
                    :title="`请求: ${point.requests}`"
                  ></div>
                  <div
                    class="trend-bar trend-bar-errors"
                    :style="{ height: getBarHeight(point.errors) + '%' }"
                    :title="`错误: ${point.errors}`"
                  ></div>
                </div>
                <div class="trend-label">{{ formatTrendTime(point.timestamp) }}</div>
              </div>
            </div>
            <el-empty v-else description="暂无趋势数据" :image-size="60" />

            <!-- Trend Summary -->
            <div v-if="usageTrend.points.length > 0" class="trend-summary">
              <div class="trend-summary-item">
                <span class="trend-summary-label">总请求</span>
                <span class="trend-summary-value agw-mono">{{ formatNumber(trendTotalRequests) }}</span>
              </div>
              <div class="trend-summary-item">
                <span class="trend-summary-label">总错误</span>
                <span class="trend-summary-value agw-mono text-danger">{{ formatNumber(trendTotalErrors) }}</span>
              </div>
              <div class="trend-summary-item">
                <span class="trend-summary-label">平均延迟</span>
                <span class="trend-summary-value agw-mono">{{ Math.round(trendAvgLatency) }}ms</span>
              </div>
            </div>
          </div>
        </div>
      </el-col>
    </el-row>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, computed } from 'vue'
import {
  fetchGlobalStats,
  fetchProviderStats,
  fetchUsageTrend,
  fetchPlanStats,
  fetchPlans,
  fetchFallbackStats,
  fetchFallbackPerformance
} from '@/api'
import type { GlobalStats, ProviderStats, PlanStats, UsageTrend, UserPlan, FallbackStats, ProviderPerformance } from '@/types'

// Loading states
const loading = reactive({
  global: false,
  providers: false,
  trend: false,
  plan: false,
  plans: false,
  fallback: false
})

// Data
const globalStats = ref<GlobalStats>({
  totalRequests: 0,
  totalErrors: 0,
  successRate: 1,
  avgLatencyMs: 0,
  totalInputTokens: 0,
  totalOutputTokens: 0,
  plansCount: 0,
  providersCount: 0,
  activeAgents: 0
})
const providerStats = ref<ProviderStats[]>([])
const usageTrend = ref<UsageTrend>({ points: [], granularity: 'hour' })
const plans = ref<UserPlan[]>([])
const selectedPlanId = ref<string>('')
const selectedPlanStats = ref<PlanStats | null>(null)
const granularity = ref<string>('hour')

// Fallback data
const fallbackStats = ref<FallbackStats>({
  totalEvents: 0,
  totalResolved: 0,
  totalUnresolved: 0,
  avgRecoveryLatencyMs: undefined,
  byTriggerType: []
})
const providerPerformance = ref<ProviderPerformance[]>([])

// Trigger type formatting
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

// Computed
const visibleTrendPoints = computed(() => {
  // Show last 24 points max
  const pts = usageTrend.value.points
  if (pts.length <= 24) return pts
  return pts.slice(pts.length - 24)
})

const maxRequestsInTrend = computed(() => {
  const max = Math.max(...visibleTrendPoints.value.map(p => p.requests), 1)
  return max
})

const trendTotalRequests = computed(() => {
  return usageTrend.value.points.reduce((sum, p) => sum + p.requests, 0)
})

const trendTotalErrors = computed(() => {
  return usageTrend.value.points.reduce((sum, p) => sum + p.errors, 0)
})

const trendAvgLatency = computed(() => {
  const pts = usageTrend.value.points
  if (pts.length === 0) return 0
  const total = pts.reduce((sum, p) => sum + p.avgLatencyMs, 0)
  return total / pts.length
})

// Helper functions
const formatNumber = (n: number) => {
  if (n >= 1000000) return (n / 1000000).toFixed(1) + 'M'
  if (n >= 1000) return (n / 1000).toFixed(1) + 'K'
  return String(n)
}

const formatPercent = (p: number) => {
  return (p * 100).toFixed(1) + '%'
}

const getQuotaColor = (percent: number) => {
  if (percent >= 90) return '#f43f5e'
  if (percent >= 70) return '#f59e0b'
  return '#10b981'
}

const formatTrendTime = (timestamp: string) => {
  if (!timestamp) return '—'
  const date = new Date(timestamp)
  const month = String(date.getMonth() + 1).padStart(2, '0')
  const day = String(date.getDate()).padStart(2, '0')
  const hour = String(date.getHours()).padStart(2, '0')
  const minute = String(date.getMinutes()).padStart(2, '0')

  if (granularity.value === 'day') {
    return `${month}-${day}`
  }
  return `${hour}:${minute}`
}

const getBarHeight = (value: number) => {
  if (maxRequestsInTrend.value === 0) return 0
  return Math.max((value / maxRequestsInTrend.value) * 100, 2)
}

// Data loading
const loadData = async () => {
  loading.global = true
  loading.providers = true
  loading.trend = true
  loading.plans = true

  try {
    const [global, providers, trend, plansData] = await Promise.all([
      fetchGlobalStats(),
      fetchProviderStats(),
      fetchUsageTrend(granularity.value),
      fetchPlans()
    ])

    globalStats.value = global
    providerStats.value = providers
    usageTrend.value = trend
    plans.value = plansData
  } catch (error) {
    console.error('Failed to load stats data:', error)
  } finally {
    loading.global = false
    loading.providers = false
    loading.trend = false
    loading.plans = false
  }
}

const loadFallbackData = async () => {
  loading.fallback = true
  try {
    const [stats, performance] = await Promise.all([
      fetchFallbackStats(),
      fetchFallbackPerformance()
    ])
    fallbackStats.value = stats
    providerPerformance.value = performance
  } catch (error) {
    console.error('Failed to load fallback data:', error)
  } finally {
    loading.fallback = false
  }
}

const loadUsageTrend = async () => {
  loading.trend = true
  try {
    usageTrend.value = await fetchUsageTrend(granularity.value)
  } catch (error) {
    console.error('Failed to load usage trend:', error)
  } finally {
    loading.trend = false
  }
}

const loadPlanStats = async () => {
  if (!selectedPlanId.value) {
    selectedPlanStats.value = null
    return
  }
  loading.plan = true
  try {
    selectedPlanStats.value = await fetchPlanStats(selectedPlanId.value)
  } catch (error) {
    console.error('Failed to load plan stats:', error)
  } finally {
    loading.plan = false
  }
}

onMounted(() => {
  loadData()
  loadFallbackData()
})
</script>

<style scoped>
.stats-page {
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
  margin-bottom: 20px;
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

.stat-card-blue .stat-icon {
  background: linear-gradient(135deg, rgba(59, 130, 246, 0.2), rgba(37, 99, 235, 0.15));
  color: #60a5fa;
}
.stat-card-emerald .stat-icon {
  background: linear-gradient(135deg, rgba(16, 185, 129, 0.2), rgba(0, 212, 170, 0.15));
  color: #34d399;
}
.stat-card-amber .stat-icon {
  background: linear-gradient(135deg, rgba(245, 158, 11, 0.2), rgba(217, 119, 6, 0.15));
  color: #fbbf24;
}
.stat-card-rose .stat-icon {
  background: linear-gradient(135deg, rgba(244, 63, 94, 0.2), rgba(239, 68, 68, 0.15));
  color: #fb7185;
}
.stat-card-purple .stat-icon {
  background: linear-gradient(135deg, rgba(139, 92, 246, 0.2), rgba(124, 58, 237, 0.15));
  color: #a78bfa;
}
.stat-card-sky .stat-icon {
  background: linear-gradient(135deg, rgba(14, 165, 233, 0.15), rgba(34, 211, 238, 0.12));
  color: #22d3ee;
}
.stat-card-cyan .stat-icon {
  background: linear-gradient(135deg, rgba(6, 182, 212, 0.2), rgba(8, 145, 178, 0.15));
  color: #67e8f9;
}
.stat-card-indigo .stat-icon {
  background: linear-gradient(135deg, rgba(99, 102, 241, 0.2), rgba(79, 70, 229, 0.15));
  color: #818cf8;
}

.stat-content {
  flex: 1;
  min-width: 0;
}

.stat-value {
  font-size: 24px;
  font-weight: 700;
  color: #e8eaf0;
  line-height: 1.2;
}

.stat-unit {
  font-size: 14px;
  font-weight: 500;
  color: #6b7280;
  margin-left: 2px;
}

.stat-label {
  font-size: 12px;
  color: #6b7280;
  margin-top: 4px;
}

.stat-progress {
  margin-top: 10px;
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

.stat-card-blue .stat-glow { background: #3b82f6; }
.stat-card-emerald .stat-glow { background: #10b981; }
.stat-card-amber .stat-glow { background: #f59e0b; }
.stat-card-rose .stat-glow { background: #f43f5e; }
.stat-card-purple .stat-glow { background: #8b5cf6; }
.stat-card-sky .stat-glow { background: #22d3ee; }
.stat-card-cyan .stat-glow { background: #06b6d4; }
.stat-card-indigo .stat-glow { background: #6366f1; }

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

.provider-name {
  font-weight: 500;
  color: #e8eaf0;
}

.provider-id {
  font-size: 11px;
  color: #4a5068;
}

.rate-tag {
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

/* ── Plan Stats Detail ── */
.plan-select {
  width: 180px;
}

.plan-select :deep(.el-input__wrapper) {
  background: rgba(255, 255, 255, 0.04);
  box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.08);
}

.plan-stats-detail {
  animation: fadeIn 0.3s ease;
}

.mini-stat {
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid rgba(255, 255, 255, 0.04);
  border-radius: 10px;
  padding: 14px;
  text-align: center;
}

.mini-stat-value {
  font-size: 18px;
  font-weight: 700;
  color: #e8eaf0;
}

.mini-stat-label {
  font-size: 11px;
  color: #6b7280;
  margin-top: 4px;
}

.quota-detail-section {
  margin-top: 20px;
  padding-top: 16px;
  border-top: 1px solid rgba(255, 255, 255, 0.04);
}

.section-subtitle {
  font-size: 13px;
  font-weight: 600;
  color: #c4c9d8;
  margin-bottom: 12px;
}

.quota-bars {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.quota-bar-item {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.quota-bar-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 12px;
  color: #94a3b8;
}

.quota-bar-value {
  font-size: 12px;
  font-weight: 600;
  color: #e8eaf0;
}

/* ── Trend Chart ── */
.trend-content {
  min-height: 200px;
}

.trend-chart {
  display: flex;
  align-items: flex-end;
  gap: 4px;
  height: 160px;
  padding: 10px 0;
  overflow-x: auto;
  border-bottom: 1px solid rgba(255, 255, 255, 0.04);
}

.trend-bar-group {
  flex: 1;
  min-width: 24px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
}

.trend-bar-wrapper {
  width: 100%;
  height: 120px;
  display: flex;
  align-items: flex-end;
  justify-content: center;
  gap: 2px;
}

.trend-bar {
  width: 6px;
  border-radius: 3px 3px 0 0;
  transition: height 0.4s ease;
  min-height: 2px;
}

.trend-bar-requests {
  background: linear-gradient(to top, #0ea5e9, #22d3ee);
}

.trend-bar-errors {
  background: linear-gradient(to top, #f43f5e, #fb7185);
}

.trend-label {
  font-size: 10px;
  color: #4a5068;
  font-family: var(--agw-font-mono, monospace);
  white-space: nowrap;
}

/* ── Trend Summary ── */
.trend-summary {
  display: flex;
  justify-content: space-around;
  padding-top: 16px;
  margin-top: 4px;
}

.trend-summary-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
}

.trend-summary-label {
  font-size: 11px;
  color: #6b7280;
}

.trend-summary-value {
  font-size: 16px;
  font-weight: 700;
  color: #e8eaf0;
}

.text-danger {
  color: #f43f5e;
}

.text-rose {
  color: #fb7185;
}

.text-emerald {
  color: #34d399;
}

.text-amber {
  color: #fbbf24;
}

.text-muted {
  color: #4a5068;
}

/* ── Fallback Stats ── */
.fallback-stats {
  min-height: 200px;
}

.trigger-type-section {
  margin-top: 16px;
  padding-top: 12px;
  border-top: 1px solid rgba(255, 255, 255, 0.04);
}

.trigger-bars {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.trigger-bar-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.trigger-bar-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 12px;
}

.trigger-label {
  color: #94a3b8;
}

.trigger-count {
  font-size: 12px;
  font-weight: 600;
  color: #e8eaf0;
}
</style>
