<template>
  <div class="agents-view">
    <div class="agents-header agw-stagger">
      <div class="header-info">
        <h2 class="section-title">Agent 工具管理</h2>
        <p class="section-desc">管理 AI 编码工具的绑定与配置</p>
      </div>
      <el-button type="primary" size="large" class="refresh-btn" :loading="isLoading" @click="loadData">
        <el-icon>
          <Refresh />
        </el-icon>
        刷新
      </el-button>
    </div>

    <!-- Loading State -->
    <div v-if="isLoading && agents.length === 0" class="loading-state">
      <el-skeleton :rows="4" animated />
    </div>

    <!-- Empty State -->
    <el-empty v-else-if="agents.length === 0" description="没有可用的 Agent 工具" class="empty-state">
      <template #image>
        <el-icon :size="80" class="empty-icon">
          <Robot />
        </el-icon>
      </template>
      <template #description>
        <p class="empty-desc">请先配置服务商以获取可用 Agent</p>
      </template>
      <el-button type="primary" size="large" @click="$router.push('/providers')">
        配置服务商
      </el-button>
    </el-empty>

    <!-- Agents Grid -->
    <div v-else class="agents-grid agw-stagger">
      <div v-for="agent in agents" :key="agent.agent_id" class="agent-card">
        <div class="agent-header">
          <div class="agent-icon">
            <img v-if="agent.logo_url" :src="agent.logo_url" :alt="agent.name" class="logo-img" />
            <el-icon v-else :size="32">
              <Robot />
            </el-icon>
          </div>
          <div class="agent-title">
            <h3 class="agent-name">{{ agent.name }}</h3>
            <span class="agent-id">{{ agent.agent_id }}</span>
          </div>
        </div>

        <div class="agent-desc" v-if="agent.description">
          {{ agent.description }}
        </div>

        <div class="agent-meta">
          <div class="meta-item" v-if="agent.supported_formats?.length">
            <span class="meta-label">支持协议:</span>
            <span class="meta-value">{{ agent.supported_formats.join(', ') }}</span>
          </div>
          <div class="meta-item" v-if="agent.config_methods?.length">
            <span class="meta-label">配置方式:</span>
            <span class="meta-value">{{ agent.config_methods.join(', ') }}</span>
          </div>
        </div>

        <div class="agent-links" v-if="agent.homepage || agent.install_url">
          <el-link v-if="agent.homepage" :href="agent.homepage" target="_blank" type="primary" :underline="false">
            <el-icon><Link /></el-icon>
            官网
          </el-link>
          <el-link v-if="agent.install_url" :href="agent.install_url" target="_blank" type="primary" :underline="false">
            <el-icon><Download /></el-icon>
            安装
          </el-link>
        </div>

        <el-divider />

        <!-- Binding Section -->
        <div class="binding-section">
          <div class="binding-header">
            <span class="binding-title">绑定状态</span>
            <el-tag v-if="getPlansWithAgent(agent.agent_id).length === 0" type="info" size="small">
              未绑定
            </el-tag>
            <el-tag v-else type="success" size="small">
              已绑定 {{ getPlansWithAgent(agent.agent_id).length }} 个套餐
            </el-tag>
          </div>

          <!-- Bound Plans List -->
          <div v-if="getPlansWithAgent(agent.agent_id).length > 0" class="bound-plans">
            <div v-for="plan in getPlansWithAgent(agent.agent_id)" :key="plan.id" class="bound-plan-item">
              <div class="plan-info">
                <span class="plan-name">{{ plan.name }}</span>
                <el-tag :type="getConfigStatusType(plan, agent.agent_id)" size="small">
                  {{ getConfigStatusLabel(plan, agent.agent_id) }}
                </el-tag>
              </div>
              <div class="plan-actions">
                <el-button
                  v-if="getAgentBinding(plan.id, agent.agent_id)?.config_status === 'not_configured'"
                  type="primary"
                  size="small"
                  @click="handleAutoConfig(plan.id, agent.agent_id)"
                >
                  一键配置
                </el-button>
                <el-button type="danger" size="small" plain @click="handleUnbind(plan.id, agent.agent_id)">
                  解绑
                </el-button>
              </div>
            </div>
          </div>

          <!-- Bind New Plan -->
          <div class="bind-new">
            <el-select
              v-model="selectedPlanForAgent[agent.agent_id]"
              placeholder="选择套餐绑定"
              size="small"
              style="width: 180px"
            >
              <el-option
                v-for="plan in unboundPlansForAgent(agent.agent_id)"
                :key="plan.id"
                :label="plan.name"
                :value="plan.id"
              />
            </el-select>
            <el-button
              type="primary"
              size="small"
              :disabled="!selectedPlanForAgent[agent.agent_id]"
              @click="handleBind(agent.agent_id)"
            >
              绑定
            </el-button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { ElMessage } from 'element-plus'
import { useAgents } from '@/composables/useAgents'
import { statusLabels } from '@/constants/HealthLabel'

const {
  agents,
  plans,
  isLoading,
  loadAgents,
  loadPlans,
  bind,
  unbind,
  autoConfig,
  getAgentBinding,
  getPlansWithAgent
} = useAgents()

const selectedPlanForAgent = reactive<Record<string, string>>({})

const loadData = async () => {
  await Promise.all([loadAgents(), loadPlans()])
}

// Get plans that don't have this agent bound
const unboundPlansForAgent = (agentId: string) => {
  return plans.value.filter(p =>
    !p.bound_agents?.some(b => b.agent_id === agentId) && p.enabled
  )
}

const getConfigStatusType = (plan: any, agentId: string) => {
  const binding = getAgentBinding(plan.id, agentId)
  if (!binding) return 'info'
  const status = binding.config_status
  switch (status) {
    case 'auto_configured': return 'success'
    case 'manually_configured': return 'success'
    case 'config_error': return 'danger'
    case 'needs_update': return 'warning'
    default: return 'info'
  }
}

const getConfigStatusLabel = (plan: any, agentId: string) => {
  const binding = getAgentBinding(plan.id, agentId)
  if (!binding) return '未知'
  return statusLabels[binding.config_status] || binding.config_status
}

const handleBind = async (agentId: string) => {
  const planId = selectedPlanForAgent[agentId]
  if (!planId) return

  const success = await bind(planId, agentId, false)
  if (success) {
    ElMessage.success('Agent 已绑定')
    selectedPlanForAgent[agentId] = ''
  } else {
    ElMessage.error('绑定失败')
  }
}

const handleUnbind = async (planId: string, agentId: string) => {
  const success = await unbind(planId, agentId)
  if (success) {
    ElMessage.success('Agent 已解绑')
  } else {
    ElMessage.error('解绑失败')
  }
}

const handleAutoConfig = async (planId: string, agentId: string) => {
  const success = await autoConfig(planId, agentId)
  if (success) {
    ElMessage.success('自动配置成功')
  } else {
    ElMessage.error('自动配置失败')
  }
}

onMounted(() => {
  loadData()
})
</script>

<style scoped>
.agents-view {
  animation: fadeIn 0.5s ease;
}

@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.agents-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
  padding: 20px 24px;
  background: var(--agw-bg-card);
  backdrop-filter: blur(20px);
  border: 1px solid var(--agw-border-default);
  border-radius: 14px;
}

.header-info {
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.section-title {
  font-size: 20px;
  font-weight: 700;
  color: var(--agw-text-primary);
  margin: 0;
}

.section-desc {
  font-size: 13px;
  color: var(--agw-text-secondary);
  margin: 0;
}

.refresh-btn {
  height: 40px;
  padding: 0 20px;
  border-radius: 10px;
  font-weight: 600;
  background: linear-gradient(135deg, #0ea5e9 0%, #06b6d4 100%);
  border: none;
  box-shadow: 0 4px 14px rgba(14, 165, 233, 0.3);
}

.refresh-btn:hover {
  transform: translateY(-1px);
  box-shadow: 0 6px 20px rgba(14, 165, 233, 0.4);
}

/* Loading State */
.loading-state {
  padding: 40px;
  background: var(--agw-bg-card);
  border-radius: 14px;
}

/* Empty State */
.empty-state {
  padding: 60px 20px;
  background: var(--agw-bg-card);
  border-radius: 14px;
  border: 1px solid var(--agw-border-subtle);
}

.empty-icon {
  color: var(--agw-text-muted);
  opacity: 0.5;
}

.empty-desc {
  color: var(--agw-text-secondary);
  font-size: 14px;
}

/* Agents Grid */
.agents-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(380px, 1fr));
  gap: 20px;
}

.agent-card {
  background: var(--agw-bg-card);
  border: 1px solid var(--agw-border-default);
  border-radius: 14px;
  padding: 20px;
  transition: all 0.3s ease;
}

.agent-card:hover {
  border-color: var(--agw-border-hover);
  transform: translateY(-2px);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.1);
}

.agent-header {
  display: flex;
  align-items: center;
  gap: 16px;
  margin-bottom: 12px;
}

.agent-icon {
  width: 48px;
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, #0ea5e9 0%, #06b6d4 100%);
  border-radius: 12px;
  color: white;
}

.logo-img {
  width: 32px;
  height: 32px;
  object-fit: contain;
}

.agent-title {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.agent-name {
  font-size: 16px;
  font-weight: 600;
  color: var(--agw-text-primary);
  margin: 0;
}

.agent-id {
  font-size: 12px;
  font-family: var(--agw-font-mono, monospace);
  color: var(--agw-text-secondary);
}

.agent-desc {
  font-size: 13px;
  color: var(--agw-text-secondary);
  line-height: 1.5;
  margin-bottom: 12px;
}

.agent-meta {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-bottom: 12px;
}

.meta-item {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
}

.meta-label {
  color: var(--agw-text-muted);
}

.meta-value {
  color: var(--agw-text-secondary);
}

.agent-links {
  display: flex;
  gap: 16px;
}

.agent-links .el-link {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 13px;
}

/* Binding Section */
.binding-section {
  margin-top: 8px;
}

.binding-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
}

.binding-title {
  font-size: 14px;
  font-weight: 500;
  color: var(--agw-text-primary);
}

.bound-plans {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: 12px;
}

.bound-plan-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 12px;
  background: var(--agw-bg-subtle);
  border-radius: 8px;
}

.plan-info {
  display: flex;
  align-items: center;
  gap: 8px;
}

.plan-name {
  font-size: 13px;
  font-weight: 500;
  color: var(--agw-text-primary);
}

.plan-actions {
  display: flex;
  gap: 8px;
}

.bind-new {
  display: flex;
  align-items: center;
  gap: 8px;
}

/* Divider */
:deep(.el-divider) {
  margin: 16px 0;
}
</style>