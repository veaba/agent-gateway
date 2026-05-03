<template>
  <div class="agents-view">
    <div class="agents-header agw-stagger">
      <div class="header-info">
        <h2 class="section-title">Agent 工具管理</h2>
        <p class="section-desc">管理 AI 编码工具的绑定与配置</p>
      </div>
      <div class="header-actions">
        <el-button type="success" size="large" class="add-btn" @click="showCreateDialog">
          <el-icon>
            <Plus />
          </el-icon>
          添加工具
        </el-button>
        <el-button type="primary" size="large" class="refresh-btn" :loading="isLoading" @click="loadData">
          <el-icon>
            <Refresh />
          </el-icon>
          刷新
        </el-button>
      </div>
    </div>

    <!-- Loading State -->
    <div v-if="isLoading && allAgents.length === 0" class="loading-state">
      <el-skeleton :rows="4" animated />
    </div>

    <!-- Empty State -->
    <el-empty v-else-if="allAgents.length === 0" description="没有可用的 Agent 工具" class="empty-state">
      <template #image>
        <el-icon :size="80" class="empty-icon">
          <Robot />
        </el-icon>
      </template>
      <template #description>
        <p class="empty-desc">请先配置服务商以获取可用 Agent，或添加自定义工具</p>
      </template>
      <div class="empty-actions">
        <el-button type="success" size="large" @click="showCreateDialog">
          添加自定义工具
        </el-button>
        <el-button type="primary" size="large" @click="$router.push('/providers')">
          配置服务商
        </el-button>
      </div>
    </el-empty>

    <!-- Agents Grid -->
    <div v-else class="agents-grid agw-stagger">
      <div v-for="agent in allAgents" :key="agent.agent_id" class="agent-card" :class="{ 'custom-agent': agent.isCustom }">
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
          <!-- Custom agent badge -->
          <el-tag v-if="agent.isCustom" type="success" size="small" class="custom-badge">
            自定义
          </el-tag>
        </div>

        <div class="agent-desc" v-if="agent.description">
          {{ agent.description }}
        </div>

        <!-- Version for custom agents -->
        <div class="agent-version" v-if="agent.isCustom && agent.version">
          <el-tag type="info" size="small">v{{ agent.version }}</el-tag>
        </div>

        <div class="agent-meta" v-if="!agent.isCustom">
          <div class="meta-item" v-if="agent.supported_formats?.length">
            <span class="meta-label">支持协议:</span>
            <span class="meta-value">{{ agent.supported_formats.join(', ') }}</span>
          </div>
          <div class="meta-item" v-if="agent.config_methods?.length">
            <span class="meta-label">配置方式:</span>
            <span class="meta-value">{{ agent.config_methods.join(', ') }}</span>
          </div>
        </div>

        <div class="agent-links" v-if="!agent.isCustom && (agent.homepage || agent.install_url)">
          <el-link v-if="agent.homepage" :href="agent.homepage" target="_blank" type="primary" :underline="false">
            <el-icon><Link /></el-icon>
            官网
          </el-link>
          <el-link v-if="agent.install_url" :href="agent.install_url" target="_blank" type="primary" :underline="false">
            <el-icon><Download /></el-icon>
            安装
          </el-link>
        </div>

        <!-- Custom agent actions -->
        <div class="custom-actions" v-if="agent.isCustom">
          <el-button type="primary" size="small" plain @click="showEditDialog(agent)">
            <el-icon><Edit /></el-icon>
            编辑
          </el-button>
          <el-button type="danger" size="small" plain @click="handleDeleteCustom(agent)">
            <el-icon><Delete /></el-icon>
            删除
          </el-button>
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
                  v-if="!agent.isCustom && getAgentBinding(plan.id, agent.agent_id)?.config_status === 'not_configured'"
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

    <!-- Create/Edit Custom Agent Dialog -->
    <el-dialog
      v-model="dialogVisible"
      :title="editingAgent ? '编辑自定义工具' : '添加自定义工具'"
      width="500px"
      :close-on-click-modal="false"
    >
      <el-form :model="dialogForm" label-width="100px" class="dialog-form">
        <el-form-item label="工具名称" required>
          <el-input v-model="dialogForm.name" placeholder="例如：My Custom Tool" />
        </el-form-item>
        <el-form-item label="工具代码" required>
          <el-input
            v-model="dialogForm.agentId"
            placeholder="例如：my-custom-tool"
            :disabled="!!editingAgent"
          />
          <div class="form-tip">唯一标识符，创建后不可修改</div>
        </el-form-item>
        <el-form-item label="版本号" required>
          <el-input v-model="dialogForm.version" placeholder="例如：1.0.0" />
        </el-form-item>
        <el-form-item label="图标 URL">
          <el-input v-model="dialogForm.logoUrl" placeholder="https://..." />
        </el-form-item>
        <el-form-item label="描述">
          <el-input v-model="dialogForm.description" type="textarea" :rows="3" placeholder="工具描述..." />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="dialogVisible = false">取消</el-button>
        <el-button type="primary" @click="handleSaveCustom" :loading="dialogLoading">
          {{ editingAgent ? '保存' : '创建' }}
        </el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { useAgents } from '@/composables/useAgents'
import { statusLabels } from '@/constants/HealthLabel'
import type { CustomAgent } from '@/types'

const {
  agents,
  customAgents,
  plans,
  isLoading,
  loadAgents,
  loadCustomAgents,
  loadPlans,
  bind,
  unbind,
  autoConfig,
  createCustom,
  updateCustom,
  deleteCustom,
  getAgentBinding,
  getPlansWithAgent,
  getAllAgents,
} = useAgents()

const selectedPlanForAgent = reactive<Record<string, string>>({})

// Dialog state
const dialogVisible = ref(false)
const dialogLoading = ref(false)
const editingAgent = ref<CustomAgent | null>(null)
const dialogForm = reactive({
  name: '',
  agentId: '',
  version: '',
  logoUrl: '',
  description: '',
})

// Combined agents list
const allAgents = computed(() => getAllAgents())

const loadData = async () => {
  await Promise.all([loadAgents(), loadCustomAgents(), loadPlans()])
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

// Custom agent dialog handlers
const showCreateDialog = () => {
  editingAgent.value = null
  dialogForm.name = ''
  dialogForm.agentId = ''
  dialogForm.version = '1.0.0'
  dialogForm.logoUrl = ''
  dialogForm.description = ''
  dialogVisible.value = true
}

const showEditDialog = (agent: any) => {
  editingAgent.value = agent
  dialogForm.name = agent.name
  dialogForm.agentId = agent.agent_id
  dialogForm.version = agent.version || '1.0.0'
  dialogForm.logoUrl = agent.logo_url || ''
  dialogForm.description = agent.description || ''
  dialogVisible.value = true
}

const handleSaveCustom = async () => {
  if (!dialogForm.name.trim()) {
    ElMessage.warning('请输入工具名称')
    return
  }
  if (!dialogForm.agentId.trim()) {
    ElMessage.warning('请输入工具代码')
    return
  }
  if (!dialogForm.version.trim()) {
    ElMessage.warning('请输入版本号')
    return
  }

  dialogLoading.value = true
  try {
    if (editingAgent.value) {
      // Update existing
      const result = await updateCustom(editingAgent.value.customId || editingAgent.value.id, {
        name: dialogForm.name.trim(),
        version: dialogForm.version.trim(),
        logoUrl: dialogForm.logoUrl.trim() || undefined,
        description: dialogForm.description.trim() || undefined,
      })
      if (result) {
        ElMessage.success('自定义工具已更新')
        dialogVisible.value = false
      }
    } else {
      // Create new
      const result = await createCustom({
        agentId: dialogForm.agentId.trim(),
        name: dialogForm.name.trim(),
        version: dialogForm.version.trim(),
        logoUrl: dialogForm.logoUrl.trim() || undefined,
        description: dialogForm.description.trim() || undefined,
      })
      if (result) {
        ElMessage.success('自定义工具已创建')
        dialogVisible.value = false
      }
    }
  } finally {
    dialogLoading.value = false
  }
}

const handleDeleteCustom = async (agent: any) => {
  try {
    await ElMessageBox.confirm(
      `确定要删除自定义工具 "${agent.name}" 吗？`,
      '删除确认',
      {
        confirmButtonText: '删除',
        cancelButtonText: '取消',
        type: 'warning',
      }
    )
    const success = await deleteCustom(agent.customId || agent.id)
    if (success) {
      ElMessage.success('自定义工具已删除')
    }
  } catch {
    // User cancelled
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

.header-actions {
  display: flex;
  gap: 12px;
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

.add-btn {
  height: 40px;
  padding: 0 20px;
  border-radius: 10px;
  font-weight: 600;
  background: linear-gradient(135deg, #22c55e 0%, #16a34a 100%);
  border: none;
  box-shadow: 0 4px 14px rgba(34, 197, 94, 0.3);
}

.add-btn:hover {
  transform: translateY(-1px);
  box-shadow: 0 6px 20px rgba(34, 197, 94, 0.4);
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

.empty-actions {
  display: flex;
  gap: 12px;
  margin-top: 16px;
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

.agent-card.custom-agent {
  border-color: rgba(34, 197, 94, 0.3);
}

.agent-card.custom-agent:hover {
  border-color: rgba(34, 197, 94, 0.5);
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

.custom-agent .agent-icon {
  background: linear-gradient(135deg, #22c55e 0%, #16a34a 100%);
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

.custom-badge {
  margin-left: auto;
}

.agent-desc {
  font-size: 13px;
  color: var(--agw-text-secondary);
  line-height: 1.5;
  margin-bottom: 12px;
}

.agent-version {
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

.custom-actions {
  display: flex;
  gap: 8px;
  margin-bottom: 12px;
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

/* Dialog */
.dialog-form {
  padding: 20px 0;
}

.form-tip {
  font-size: 12px;
  color: var(--agw-text-muted);
  margin-top: 4px;
}

/* Divider */
:deep(.el-divider) {
  margin: 16px 0;
}
</style>