<template>
  <div class="dashboard">
    <!-- Stats Cards -->
    <el-row :gutter="24" class="stats-row">
      <el-col :xs="24" :sm="12" :lg="6">
        <div class="stat-card stat-card-primary">
          <div class="stat-icon">
            <el-icon :size="32"><Connection /></el-icon>
          </div>
          <div class="stat-content">
            <div class="stat-value">{{ stats.plans }}</div>
            <div class="stat-label">已配置套餐</div>
          </div>
          <div class="stat-glow"></div>
        </div>
      </el-col>
      <el-col :xs="24" :sm="12" :lg="6">
        <div class="stat-card stat-card-success">
          <div class="stat-icon">
            <el-icon :size="32"><DataLine /></el-icon>
          </div>
          <div class="stat-content">
            <div class="stat-value">{{ stats.quotaUsed }}%</div>
            <div class="stat-label">配额使用率</div>
            <el-progress :percentage="stats.quotaUsed" :stroke-width="6" class="stat-progress" />
          </div>
          <div class="stat-glow"></div>
        </div>
      </el-col>
      <el-col :xs="24" :sm="12" :lg="6">
        <div class="stat-card stat-card-warning">
          <div class="stat-icon">
            <el-icon :size="32"><CircleCheck /></el-icon>
          </div>
          <div class="stat-content">
            <div class="stat-value">
              <el-tag :type="stats.healthStatus === 'healthy' ? 'success' : 'danger'" effect="dark" round>
                {{ stats.healthStatus === 'healthy' ? '正常' : '异常' }}
              </el-tag>
            </div>
            <div class="stat-label">健康状态 · {{ stats.lastCheck }}</div>
          </div>
          <div class="stat-glow"></div>
        </div>
      </el-col>
      <el-col :xs="24" :sm="12" :lg="6">
        <div class="stat-card stat-card-info">
          <div class="stat-icon">
            <el-icon :size="32"><TrendCharts /></el-icon>
          </div>
          <div class="stat-content">
            <div class="stat-value">{{ stats.requestsToday }}</div>
            <div class="stat-label">今日请求数</div>
          </div>
          <div class="stat-glow"></div>
        </div>
      </el-col>
    </el-row>

    <!-- Main Content -->
    <el-row :gutter="24" class="content-row">
      <el-col :xs="24" :lg="16">
        <div class="content-card">
          <div class="card-header">
            <span class="card-title">最近的请求</span>
            <el-button text @click="$router.push('/logs')">
              查看全部
              <el-icon class="el-icon--right"><ArrowRight /></el-icon>
            </el-button>
          </div>
          <el-table :data="recentRequests" class="dashboard-table" :row-class-name="getRowClass">
            <el-table-column prop="time" label="时间" width="100" />
            <el-table-column prop="plan" label="套餐" min-width="120" />
            <el-table-column prop="agent" label="Agent" width="120" />
            <el-table-column prop="model" label="模型" min-width="100" />
            <el-table-column label="状态" width="80" align="center">
              <template #default="{ row }">
                <el-tag :type="row.status >= 200 && row.status < 300 ? 'success' : 'danger'" size="small" round>
                  {{ row.status }}
                </el-tag>
              </template>
            </el-table-column>
            <el-table-column label="延迟" width="80" align="right">
              <template #default="{ row }">
                <span class="latency-value" :class="{ 'latency-high': row.latency > 200 }">
                  {{ row.latency }}ms
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
                <el-icon :size="24"><Plus /></el-icon>
              </div>
              <div class="action-content">
                <div class="action-title">添加新套餐</div>
                <div class="action-desc">配置新的 AI 服务</div>
              </div>
              <el-icon class="action-arrow"><ArrowRight /></el-icon>
            </div>
            <div class="action-item" @click="$router.push('/plans')">
              <div class="action-icon">
                <el-icon :size="24"><Connection /></el-icon>
              </div>
              <div class="action-content">
                <div class="action-title">管理套餐</div>
                <div class="action-desc">编辑或删除现有套餐</div>
              </div>
              <el-icon class="action-arrow"><ArrowRight /></el-icon>
            </div>
            <div class="action-item" @click="$router.push('/fallback')">
              <div class="action-icon">
                <el-icon :size="24"><Refresh /></el-icon>
              </div>
              <div class="action-content">
                <div class="action-title">配置降级策略</div>
                <div class="action-desc">设置自动故障转移</div>
              </div>
              <el-icon class="action-arrow"><ArrowRight /></el-icon>
            </div>
          </div>
        </div>
      </el-col>
    </el-row>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'

const stats = ref({
  plans: 2,
  quotaUsed: 45,
  healthStatus: 'healthy',
  lastCheck: '2分钟前',
  requestsToday: 156
})

const recentRequests = ref([
  { time: '14:32', plan: 'Alaya Plus', agent: 'Claude Code', model: 'GLM-5', status: 200, latency: 120 },
  { time: '14:31', plan: 'Anthropic', agent: 'Claude Code', model: 'Sonnet 4.5', status: 200, latency: 85 },
  { time: '14:30', plan: 'Alaya Plus', agent: 'Kimi CLI', model: 'GLM-5', status: 429, latency: 15 },
  { time: '14:28', plan: 'Kimi', agent: 'Kimi Code', model: 'Moonshot', status: 200, latency: 95 },
])

const getRowClass = ({ row }: { row: { status: number } }) => {
  return row.status >= 200 && row.status < 300 ? 'success-row' : 'error-row'
}

onMounted(() => {
  // TODO: 加载实际数据
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

/* Stats Cards */
.stats-row {
  margin-bottom: 24px;
}

.stat-card {
  position: relative;
  background: var(--el-bg-color);
  border-radius: 16px;
  padding: 24px;
  display: flex;
  align-items: flex-start;
  gap: 16px;
  overflow: hidden;
  transition: all 0.3s ease;
  border: 1px solid var(--el-border-color-light);
}

.stat-card:hover {
  transform: translateY(-4px);
  box-shadow: 0 12px 40px rgba(0, 0, 0, 0.08);
}

.stat-icon {
  width: 56px;
  height: 56px;
  border-radius: 14px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.stat-card-primary .stat-icon {
  background: linear-gradient(135deg, rgba(103, 126, 234, 0.15), rgba(118, 75, 162, 0.15));
  color: #667eea;
}

.stat-card-success .stat-icon {
  background: linear-gradient(135deg, rgba(103, 232, 138, 0.15), rgba(82, 196, 26, 0.15));
  color: #52c41a;
}

.stat-card-warning .stat-icon {
  background: linear-gradient(135deg, rgba(250, 173, 20, 0.15), rgba(255, 153, 0, 0.15));
  color: #faad14;
}

.stat-card-info .stat-icon {
  background: linear-gradient(135deg, rgba(64, 158, 255, 0.15), rgba(19, 111, 222, 0.15));
  color: #409eff;
}

.stat-content {
  flex: 1;
  min-width: 0;
}

.stat-value {
  font-size: 28px;
  font-weight: 700;
  color: var(--el-text-color-primary);
  line-height: 1.2;
}

.stat-label {
  font-size: 13px;
  color: var(--el-text-color-secondary);
  margin-top: 4px;
}

.stat-progress {
  margin-top: 12px;
}

.stat-glow {
  position: absolute;
  top: -50%;
  right: -50%;
  width: 100%;
  height: 100%;
  border-radius: 50%;
  opacity: 0.1;
  pointer-events: none;
}

.stat-card-primary .stat-glow { background: #667eea; }
.stat-card-success .stat-glow { background: #52c41a; }
.stat-card-warning .stat-glow { background: #faad14; }
.stat-card-info .stat-glow { background: #409eff; }

/* Content Cards */
.content-row {
  margin-bottom: 24px;
}

.content-card {
  background: var(--el-bg-color);
  border-radius: 16px;
  padding: 24px;
  border: 1px solid var(--el-border-color-light);
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.card-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--el-text-color-primary);
}

/* Dashboard Table */
.dashboard-table {
  --el-table-header-bg-color: transparent;
}

.dashboard-table :deep(th) {
  font-weight: 600;
  color: var(--el-text-color-secondary);
  font-size: 12px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.dashboard-table :deep(td) {
  font-size: 14px;
}

.latency-value {
  font-family: 'SF Mono', Monaco, monospace;
  font-size: 13px;
  color: var(--el-text-color-secondary);
}

.latency-value.latency-high {
  color: #f56c6c;
}

/* Quick Actions */
.quick-actions {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.action-item {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 16px;
  border-radius: 12px;
  background: var(--el-fill-color-light);
  cursor: pointer;
  transition: all 0.3s ease;
}

.action-item:hover {
  background: var(--el-fill-color);
  transform: translateX(4px);
}

.action-item.action-primary {
  background: linear-gradient(135deg, rgba(103, 126, 234, 0.15), rgba(118, 75, 162, 0.15));
  border: 1px solid rgba(103, 126, 234, 0.2);
}

.action-item.action-primary:hover {
  background: linear-gradient(135deg, rgba(103, 126, 234, 0.25), rgba(118, 75, 162, 0.25));
}

.action-icon {
  width: 44px;
  height: 44px;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--el-bg-color);
  color: var(--el-color-primary);
  flex-shrink: 0;
}

.action-item.action-primary .action-icon {
  background: linear-gradient(135deg, #667eea, #764ba2);
  color: white;
}

.action-content {
  flex: 1;
  min-width: 0;
}

.action-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--el-text-color-primary);
}

.action-desc {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  margin-top: 2px;
}

.action-arrow {
  color: var(--el-text-color-secondary);
  transition: transform 0.3s ease;
}

.action-item:hover .action-arrow {
  transform: translateX(4px);
  color: var(--el-color-primary);
}
</style>