<template>
  <div class="plans-view">
    <div class="plans-header agw-stagger">
      <div class="header-info">
        <h2 class="section-title">我的套餐</h2>
        <p class="section-desc">管理您的 AI 服务配置</p>
      </div>
      <el-button type="primary" size="large" class="add-btn" @click="$router.push('/plans/add')">
        <el-icon>
          <Plus />
        </el-icon>
        添加套餐
      </el-button>
    </div>

    <!-- Loading State -->
    <div v-if="loading && plans.length === 0" class="loading-state">
      <el-skeleton :rows="4" animated />
    </div>

    <!-- Empty State -->
    <el-empty v-else-if="plans.length === 0" description="还没有配置任何套餐" class="empty-state">
      <template #image>
        <el-icon :size="80" class="empty-icon">
          <Connection />
        </el-icon>
      </template>
      <template #description>
        <p class="empty-desc">添加第一个 AI 套餐开始使用</p>
      </template>
      <el-button type="primary" size="large" @click="$router.push('/plans/add')">
        <el-icon>
          <Plus />
        </el-icon>
        添加第一个套餐
      </el-button>
    </el-empty>

    <!-- Plans Grid -->
    <div v-else class="plans-grid agw-stagger">
      <PlanCard v-for="plan in plans" :key="plan.id" :plan="plan" :default-plan-id="defaultPlanId"
        :provider="getProviderForPlan(plan)" @edit="handleEdit" @delete="handleDelete" @test="handleTest"
        @set-default="handleSetDefault" @bind-agent="handleBindAgent" @auto-config="handleAutoConfig"
        @test-agent="handleTestAgent" @view-detail="handleViewDetail" />
    </div>

    <!-- Detail Drawer -->
    <el-drawer v-model="drawerVisible" :title="selectedPlan?.name + ' 详情'" direction="rtl" size="480px"
      :close-on-click-modal="true" class="detail-drawer">
      <template #header>
        <div class="drawer-header">
          <span class="drawer-title">{{ selectedPlan?.name }}</span>
          <el-tag v-if="selectedPlan?.id === defaultPlanId" type="primary" size="small" effect="dark" round>默认</el-tag>
        </div>
      </template>

      <div v-if="selectedPlan" class="drawer-content">
        <!-- Provider Info Section -->
        <div class="detail-section">
          <div class="section-header">
            <el-icon>
              <OfficeBuilding />
            </el-icon>
            <span>Provider 信息</span>
          </div>
          <div class="section-body">
            <div class="detail-row">
              <span class="detail-label">名称</span>
              <span class="detail-value">{{ selectedProviderInfo.name || selectedPlan.providerId }}</span>
            </div>
            <div v-if="selectedProviderInfo.homepage" class="detail-row">
              <span class="detail-label">官网</span>
              <a class="detail-link" :href="selectedProviderInfo.homepage" target="_blank">
                {{ selectedProviderInfo.homepage }}
                <el-icon size="12">
                  <Link />
                </el-icon>
              </a>
            </div>
            <div v-if="selectedProviderInfo.docsUrl" class="detail-row">
              <span class="detail-label">文档</span>
              <a class="detail-link" :href="selectedProviderInfo.docsUrl" target="_blank">
                {{ selectedProviderInfo.docsUrl }}
                <el-icon size="12">
                  <Link />
                </el-icon>
              </a>
            </div>
            <div v-if="selectedProviderInfo.getApiKeyUrl" class="detail-row">
              <span class="detail-label">API Key</span>
              <a class="detail-link" :href="selectedProviderInfo.getApiKeyUrl" target="_blank">
                获取 API Key
                <el-icon size="12">
                  <Link />
                </el-icon>
              </a>
            </div>
            <div class="detail-row">
              <span class="detail-label">API 格式</span>
              <el-tag size="small" effect="plain" round>{{ selectedProviderInfo.apiFormat || selectedPlan.providerId }}</el-tag>
            </div>
          </div>
        </div>

        <!-- Coding Plan Info Section -->
        <div class="detail-section">
          <div class="section-header">
            <el-icon>
              <Tickets />
            </el-icon>
            <span>Coding Plan 信息</span>
          </div>
          <div class="section-body">
            <div class="detail-row">
              <span class="detail-label">方案</span>
              <span class="detail-value">{{ selectedPlanTemplate?.name || selectedPlan.planId }}</span>
            </div>
            <div v-if="selectedPlanTemplate" class="detail-row">
              <span class="detail-label">等级</span>
              <el-tag size="small" :type="getTierType(selectedPlanTemplate.tier)" effect="plain" round>
                {{ getTierLabel(selectedPlanTemplate.tier) }}
              </el-tag>
            </div>
            <div v-if="selectedPlanTemplate?.description" class="detail-row">
              <span class="detail-label">描述</span>
              <span class="detail-value description-value">{{ selectedPlanTemplate.description }}</span>
            </div>
            <div v-if="selectedPlanTemplate?.price" class="detail-row">
              <span class="detail-label">订阅</span>
              <span class="detail-value price-value">{{ selectedPlanTemplate.price }}</span>
            </div>
            <div v-if="selectedPlanTemplate?.features?.length" class="detail-row">
              <span class="detail-label">特性</span>
              <div class="feature-tags">
                <el-tag v-for="f in selectedPlanTemplate.features" :key="f" size="small" effect="plain" round
                  type="info">
                  {{ f }}
                </el-tag>
              </div>
            </div>
          </div>
        </div>

        <!-- Model Config Section -->
        <div class="detail-section">
          <div class="section-header">
            <el-icon>
              <Cpu />
            </el-icon>
            <span>模型配置</span>
          </div>
          <div class="section-body">
            <div class="detail-row">
              <span class="detail-label">当前</span>
              <el-tag size="default" effect="dark" round type="primary">{{ selectedPlan.selectedModelId }}</el-tag>
            </div>
            <div v-if="selectedAvailableModels.length" class="detail-row">
              <span class="detail-label">可选</span>
              <div class="model-chips">
                <el-tag v-for="m in selectedAvailableModels" :key="m.modelId" size="small"
                  :type="m.modelId === selectedPlan.selectedModelId ? 'primary' : 'info'"
                  :effect="m.modelId === selectedPlan.selectedModelId ? 'dark' : 'plain'" round class="model-chip">
                  {{ m.name }}
                </el-tag>
              </div>
            </div>
            <div v-if="selectedModelInfo" class="detail-row">
              <span class="detail-label">能力</span>
              <div class="capability-tags">
                <el-tag v-for="cap in selectedModelInfo.capabilities" :key="cap" size="small" effect="plain" round
                  :type="getCapabilityType(cap)">
                  {{ getCapabilityLabel(cap) }}
                </el-tag>
              </div>
            </div>
            <div v-if="selectedModelInfo?.contextLength" class="detail-row">
              <span class="detail-label">上下文</span>
              <span class="detail-value mono">{{ formatContextLength(selectedModelInfo.contextLength) }}</span>
            </div>
          </div>
        </div>

        <!-- Agent Tools Binding Section -->
        <div class="detail-section">
          <div class="section-header">
            <el-icon>
              <Connection />
            </el-icon>
            <span>Agent 工具绑定</span>
          </div>
          <div class="section-body">
            <div v-if="!selectedPlan.boundAgents?.length" class="empty-agents">
              <span class="muted">未绑定任何 Agent 工具</span>
            </div>
            <div v-for="agent in selectedPlan.boundAgents" :key="agent.agentId" class="agent-binding-row">
              <div class="agent-binding-header">
                <span class="agent-name">{{ getAgentName(agent.agentId, selectedProvider) }}</span>
                <el-tag size="small" :type="agent.configured ? 'success' : 'danger'" effect="dark" round>
                  {{ getAgentStatusLabel(agent) }}
                </el-tag>
              </div>
              <div class="agent-binding-detail">
                <div v-if="getAgentSetupGuide(agent.agentId, selectedProvider)" class="agent-env-vars">
                  <div v-for="ev in getAgentEnvVars(agent.agentId, selectedProvider)" :key="ev.name" class="env-var-row">
                    <span class="mono env-var-name">{{ ev.name }}</span>
                    <span class="env-var-value">{{ ev.value }}</span>
                  </div>
                  <div v-if="getAgentConfigPaths(agent.agentId, selectedProvider)" class="config-path">
                    <el-icon size="12">
                      <Document />
                    </el-icon>
                    {{ getAgentConfigPaths(agent.agentId, selectedProvider) }}
                  </div>
                </div>
                <div class="agent-actions">
                  <el-button v-if="!agent.configured" size="small" type="primary"
                    @click="handleAutoConfig(selectedPlan.id, agent.agentId)">
                    <el-icon>
                      <SetUp />
                    </el-icon>
                    一键配置
                  </el-button>
                  <el-button size="small" @click="handleTestAgent(selectedPlan.id, agent.agentId)">
                    <el-icon>
                      <Connection />
                    </el-icon>
                    测试连接
                  </el-button>
                </div>
              </div>
            </div>
            <!-- Unbound agents supported by provider -->
            <div v-if="selectedUnboundAgents.length" class="unbound-section">
              <div class="unbound-header">可绑定的 Agent 工具</div>
              <div class="unbound-agents">
                <el-tag v-for="ua in selectedUnboundAgents" :key="ua.agentId" size="small" effect="plain" round type="info"
                  class="unbound-tag" @click="handleBindAgent(selectedPlan.id, ua.agentId)">
                  + {{ ua.name }}
                </el-tag>
              </div>
            </div>
          </div>
        </div>

        <!-- API Key Section -->
        <div class="detail-section">
          <div class="section-header">
            <el-icon>
              <Key />
            </el-icon>
            <span>API Key</span>
          </div>
          <div class="section-body">
            <div class="detail-row">
              <span class="detail-label">状态</span>
              <el-tag :type="selectedPlan.apiKeyMasked ? 'success' : 'danger'" size="small" effect="dark" round>
                {{ selectedPlan.apiKeyMasked ? '已配置' : '未配置' }}
              </el-tag>
            </div>
            <div v-if="selectedPlan.apiKeyMasked" class="detail-row">
              <span class="detail-label">Key</span>
              <span class="detail-value mono api-key-masked">{{ selectedPlan.apiKeyMasked }}</span>
            </div>
            <div class="api-key-actions">
              <el-button size="small" @click="handleEdit(selectedPlan)">
                <el-icon>
                  <Edit />
                </el-icon>
                更新 Key
              </el-button>
              <el-button v-if="selectedProviderInfo.getApiKeyUrl" size="small" type="primary"
                @click="openUrl(selectedProviderInfo.getApiKeyUrl)">
                <el-icon>
                  <Link />
                </el-icon>
                获取 Key
              </el-button>
            </div>
          </div>
        </div>

        <!-- Quota & Limits Section -->
        <div class="detail-section">
          <div class="section-header">
            <el-icon>
              <Odometer />
            </el-icon>
            <span>配额与限制</span>
          </div>
          <div class="section-body">
            <div class="quota-grid">
              <div class="quota-item">
                <span class="quota-item-label">日配额</span>
                <span class="quota-item-value">
                  {{ selectedPlan.customQuotaDaily ?? selectedPlanTemplate?.quotaDaily ?? '无限制' }}
                  <template v-if="selectedPlan.customQuotaDaily || selectedPlanTemplate?.quotaDaily">
                    {{ selectedQuotaUsed }} 已用
                  </template>
                </span>
                <el-progress v-if="selectedQuotaPercent > 0" :percentage="selectedQuotaPercent" :stroke-width="6"
                  :color="getQuotaColor(selectedQuotaPercent)" :show-text="false" class="quota-progress" />
              </div>
              <div class="quota-item">
                <span class="quota-item-label">月配额</span>
                <span class="quota-item-value">
                  {{ selectedPlan.customQuotaMonthly ?? selectedPlanTemplate?.quotaMonthly ?? '无限制' }}
                </span>
              </div>
              <div class="quota-item">
                <span class="quota-item-label">RPM 限制</span>
                <span class="quota-item-value">{{ selectedPlan.customRpmLimit ?? selectedPlanTemplate?.rpmLimit ?? '无限制' }}</span>
              </div>
              <div class="quota-item">
                <span class="quota-item-label">Fallback 优先级</span>
                <span class="quota-item-value">{{ selectedPlan.priority }}</span>
              </div>
            </div>
          </div>
        </div>

        <!-- Action Buttons -->
        <div class="detail-actions">
          <el-button size="default" @click="handleEdit(selectedPlan)">
            <el-icon>
              <Edit />
            </el-icon>
            编辑
          </el-button>
          <el-button size="default" type="success" @click="handleTest(selectedPlan)">
            <el-icon>
              <Connection />
            </el-icon>
            测试连接
          </el-button>
          <el-button v-if="selectedPlan.id !== defaultPlanId" size="default" type="warning" @click="handleSetDefault(selectedPlan)">
            <el-icon>
              <Star />
            </el-icon>
            设为默认
          </el-button>
          <el-button size="default" type="danger" @click="handleDelete(selectedPlan)">
            <el-icon>
              <Delete />
            </el-icon>
            删除
          </el-button>
        </div>
      </div>
    </el-drawer>

    <!-- Edit Dialog -->
    <el-dialog v-model="editDialogVisible" title="编辑套餐" width="560px" :close-on-click-modal="false" class="edit-dialog">
      <el-form ref="editFormRef" :model="editForm" :rules="editRules" label-position="top" class="edit-form">
        <el-form-item label="套餐名称" prop="name">
          <el-input v-model="editForm.name" placeholder="给套餐起个名字" clearable />
        </el-form-item>

        <el-form-item label="API Key" prop="apiKey">
          <el-input v-model="editForm.apiKey" type="password" show-password placeholder="留空则保持原值，填写则更新" />
          <div class="form-tip">留空则保持原 API Key 不变</div>
        </el-form-item>

        <el-form-item label="模型" prop="selectedModelId">
          <el-select v-model="editForm.selectedModelId" placeholder="选择模型" clearable style="width: 100%">
            <el-option v-for="model in availableModels" :key="model.modelId" :label="model.name" :value="model.modelId">
              <div class="model-option">
                <span class="model-name">{{ model.name }}</span>
                <span class="model-id">{{ model.modelId }}</span>
              </div>
            </el-option>
          </el-select>
        </el-form-item>

        <el-row :gutter="16">
          <el-col :span="12">
            <el-form-item label="启用状态">
              <el-switch v-model="editForm.enabled" />
            </el-form-item>
          </el-col>
          <el-col :span="12">
            <el-form-item label="优先级" prop="priority">
              <el-input-number v-model="editForm.priority" :min="1" :max="100" controls-position="right"
                style="width: 100%" />
            </el-form-item>
          </el-col>
        </el-row>

        <el-divider content-position="left">自定义配额</el-divider>

        <el-row :gutter="16">
          <el-col :span="8">
            <el-form-item label="日配额上限">
              <el-input-number v-model="editForm.customQuotaDaily" :min="0" controls-position="right"
                style="width: 100%" placeholder="无限制" />
            </el-form-item>
          </el-col>
          <el-col :span="8">
            <el-form-item label="月配额上限">
              <el-input-number v-model="editForm.customQuotaMonthly" :min="0" controls-position="right"
                style="width: 100%" placeholder="无限制" />
            </el-form-item>
          </el-col>
          <el-col :span="8">
            <el-form-item label="RPM 限制">
              <el-input-number v-model="editForm.customRpmLimit" :min="0" controls-position="right" style="width: 100%"
                placeholder="无限制" />
            </el-form-item>
          </el-col>
        </el-row>

        <el-form-item label="备注" prop="notes">
          <el-input v-model="editForm.notes" type="textarea" :rows="2" placeholder="可选备注信息" />
        </el-form-item>
      </el-form>

      <template #footer>
        <el-button @click="editDialogVisible = false">取消</el-button>
        <el-button type="primary" :loading="saving" @click="handleSaveEdit">
          保存更改
        </el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, computed } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import type { FormInstance, FormRules } from 'element-plus'
import PlanCard from '@/components/PlanCard.vue'
import type { UserPlan, Provider, Model, CodingPlan, AgentBinding } from '@/types'
import { fetchPlans, deletePlan, testPlan, updatePlan, fetchProvider, fetchProviders } from '@/api'
import { providerNames, statusLabels } from '@/constants/HealthLabel'

const plans = ref<UserPlan[]>([])
const providers = ref<Provider[]>([])
const loading = ref(false)
const editDialogVisible = ref(false)
const saving = ref(false)
const editFormRef = ref<FormInstance>()
const editingPlan = ref<UserPlan | null>(null)
const availableModels = ref<Model[]>([])

// Drawer state
const drawerVisible = ref(false)
const selectedPlan = ref<UserPlan | null>(null)

const editForm = reactive({
  name: '',
  apiKey: '',
  selectedModelId: '',
  enabled: true,
  priority: 1,
  customQuotaDaily: 0,
  customQuotaMonthly: 0,
  customRpmLimit: 0,
  notes: ''
})

const editRules: FormRules = {
  name: [{ required: true, message: '请输入套餐名称', trigger: 'blur' }]
}

const defaultPlanId = computed(() => {
  // The first enabled plan is the default (or could be from config)
  const enabled = plans.value.filter(p => p.enabled)
  return enabled.length > 0 ? enabled[0].id : ''
})

const getProviderForPlan = (plan: UserPlan): Provider | undefined => {
  return providers.value.find(p => p.providerId === plan.providerId)
}

// Selected plan's provider info (for drawer)
const selectedProvider = computed(() => {
  if (!selectedPlan.value) return undefined
  return providers.value.find(p => p.providerId === selectedPlan.value!.providerId)
})

const selectedProviderInfo = computed(() => selectedProvider.value || {
  providerId: selectedPlan.value?.providerId || '',
  name: providerNames[selectedPlan.value?.providerId || ''] || selectedPlan.value?.providerId,
  apiFormat: 'anthropic',
  homepage: '',
  docsUrl: '',
} as Provider)

const selectedPlanTemplate = computed((): CodingPlan | undefined => {
  if (!selectedProvider.value?.codingPlans || !selectedPlan.value) return undefined
  return selectedProvider.value.codingPlans.find(cp => cp.planId === selectedPlan.value!.planId)
})

const selectedAvailableModels = computed((): Model[] => {
  if (!selectedProvider.value?.models || !selectedPlan.value) return []
  const template = selectedPlanTemplate.value
  if (!template) return selectedProvider.value.models || []
  return selectedProvider.value.models.filter(m => template.supportedModelIds.includes(m.modelId))
})

const selectedModelInfo = computed((): Model | undefined => {
  if (!selectedPlan.value) return undefined
  return selectedAvailableModels.value.find(m => m.modelId === selectedPlan.value!.selectedModelId)
})

const selectedUnboundAgents = computed(() => {
  if (!selectedProvider.value?.supportedAgents || !selectedPlan.value) return []
  const boundIds = selectedPlan.value.boundAgents.map(a => a.agentId)
  if (selectedPlanTemplate.value?.supportedAgentIds) {
    return selectedProvider.value.supportedAgents.filter(
      a => selectedPlanTemplate.value!.supportedAgentIds.includes(a.agentId) && !boundIds.includes(a.agentId)
    )
  }
  return selectedProvider.value.supportedAgents.filter(a => !boundIds.includes(a.agentId))
})

const selectedQuotaUsed = computed(() => selectedPlan.value?.quotaUsed ?? 0)
const selectedQuotaLimit = computed(() => selectedPlan.value?.quotaLimit ?? (selectedPlanTemplate.value?.quotaDaily ?? 500))
const selectedQuotaPercent = computed(() => {
  if (!selectedQuotaLimit.value) return 0
  return Math.min(100, Math.floor((selectedQuotaUsed.value / selectedQuotaLimit.value) * 100))
})

// Load plans & providers
const loadPlans = async () => {
  loading.value = true
  try {
    const [plansData, providersData] = await Promise.all([
      fetchPlans(),
      fetchProviders()
    ])
    plans.value = plansData
    providers.value = providersData
  } catch {
    ElMessage.error('加载数据失败')
  } finally {
    loading.value = false
  }
}

// Load models for provider
const loadProviderModels = async (providerId: string) => {
  try {
    const provider = await fetchProvider(providerId)
    availableModels.value = provider.models || []
  } catch {
    availableModels.value = []
  }
}

// Open edit dialog
const handleEdit = async (plan: UserPlan) => {
  editingPlan.value = plan
  await loadProviderModels(plan.providerId)

  editForm.name = plan.name
  editForm.apiKey = ''
  editForm.selectedModelId = plan.selectedModelId || ''
  editForm.enabled = plan.enabled
  editForm.priority = plan.priority || 1
  editForm.customQuotaDaily = plan.customQuotaDaily || 0
  editForm.customQuotaMonthly = plan.customQuotaMonthly || 0
  editForm.customRpmLimit = plan.customRpmLimit || 0
  editForm.notes = plan.notes || ''

  editDialogVisible.value = true
}

// Save edit
const handleSaveEdit = async () => {
  if (!editFormRef.value || !editingPlan.value) return

  await editFormRef.value.validate(async (valid) => {
    if (!valid) return

    saving.value = true
    try {
      const updateData: Partial<UserPlan> = {
        name: editForm.name,
        enabled: editForm.enabled,
        priority: editForm.priority,
        notes: editForm.notes || undefined
      }

      if (editForm.apiKey.trim()) {
        updateData.apiKey = editForm.apiKey.trim()
      }

      if (editForm.selectedModelId) {
        updateData.selectedModelId = editForm.selectedModelId
      }

      if (editForm.customQuotaDaily > 0) {
        updateData.customQuotaDaily = editForm.customQuotaDaily
      }

      if (editForm.customQuotaMonthly > 0) {
        updateData.customQuotaMonthly = editForm.customQuotaMonthly
      }

      if (editForm.customRpmLimit > 0) {
        updateData.customRpmLimit = editForm.customRpmLimit
      }

      const updated = await updatePlan(editingPlan.value.id, updateData)

      const index = plans.value.findIndex(p => p.id === editingPlan.value!.id)
      if (index !== -1) {
        plans.value[index] = updated
      }

      ElMessage.success('套餐已更新')
      editDialogVisible.value = false
    } catch (e: any) {
      ElMessage.error(e?.message || '更新套餐失败')
    } finally {
      saving.value = false
    }
  })
}

// Delete plan
const handleDelete = async (plan: UserPlan) => {
  try {
    await ElMessageBox.confirm(
      `确定要删除套餐 "${plan.name}" 吗？此操作不可撤销。`,
      '确认删除',
      {
        confirmButtonText: '删除',
        cancelButtonText: '取消',
        type: 'warning',
        confirmButtonClass: 'el-button--danger'
      }
    )
    await deletePlan(plan.id)
    plans.value = plans.value.filter(p => p.id !== plan.id)
    ElMessage.success('删除成功')
  } catch (e: any) {
    if (e !== 'cancel') {
      ElMessage.error('删除失败')
    }
  }
}

// Test plan
const handleTest = async (plan: UserPlan) => {
  try {
    const success = await testPlan(plan.id)
    if (success) {
      ElMessage.success('连接测试成功')
    } else {
      ElMessage.warning('连接测试失败，请检查 API Key 和网络')
    }
  } catch {
    ElMessage.error('连接测试失败')
  }
}

// Set default plan
const handleSetDefault = async (plan: UserPlan) => {
  try {
    await updatePlan(plan.id, { priority: 0 } as Partial<UserPlan>)
    // Reload plans to reflect change
    plans.value = await fetchPlans()
    ElMessage.success('已设为默认套餐')
  } catch {
    ElMessage.error('设置默认套餐失败')
  }
}

// Bind agent
const handleBindAgent = async (planId: string, agentId: string) => {
  try {
    const { default: api } = await import('@/api')
    await api.post(`/plans/${planId}/agents/${agentId}/bind`, { autoConfig: false })
    plans.value = await fetchPlans()
    ElMessage.success(`Agent ${agentId} 已绑定`)
  } catch {
    ElMessage.error('绑定 Agent 失败')
  }
}

// Auto config agent
const handleAutoConfig = async (planId: string, agentId: string) => {
  try {
    const { default: api } = await import('@/api')
    await api.post(`/plans/${planId}/agents/${agentId}/auto-config`)
    plans.value = await fetchPlans()
    ElMessage.success(`Agent ${agentId} 自动配置成功`)
  } catch {
    ElMessage.error('自动配置失败')
  }
}

// Test agent connection
const handleTestAgent = async (planId: string, agentId: string) => {
  try {
    const { default: api } = await import('@/api')
    const { data } = await api.post(`/plans/${planId}/test`)
    if (data.data.success) {
      ElMessage.success('连接测试成功')
    } else {
      ElMessage.warning('连接测试失败')
    }
  } catch {
    ElMessage.error('测试连接失败')
  }
}

// Open detail drawer
const handleViewDetail = (plan: UserPlan) => {
  selectedPlan.value = plan
  drawerVisible.value = true
}

// Helper functions for drawer
const getTierType = (tier: string) => {
  const types: Record<string, string> = {
    free: 'success',
    pro: 'warning',
    enterprise: 'danger',
    custom: 'info'
  }
  return types[tier] || 'info'
}

const getTierLabel = (tier: string) => {
  const labels: Record<string, string> = {
    free: '免费版',
    pro: '专业版',
    enterprise: '企业版',
    custom: '自定义'
  }
  return labels[tier] || tier
}

const getQuotaColor = (percent: number) => {
  if (percent >= 90) return '#f56c6c'
  if (percent >= 70) return '#e6a23c'
  return '#67c23a'
}

const getAgentName = (agentId: string, provider?: Provider) => {
  if (provider?.supportedAgents) {
    const found = provider.supportedAgents.find(a => a.agentId === agentId)
    if (found) return found.name
  }
  const names: Record<string, string> = {
    'claude-code': 'Claude Code',
    'kimi-cli': 'Kimi CLI',
    'opencode': 'OpenCode',
    'kilo-cli': 'Kilo CLI'
  }
  return names[agentId] || agentId
}

const getAgentStatusLabel = (agent: AgentBinding) => {
  if (agent.configured) return '已配置'
  return statusLabels[agent.configStatus] || '未配置'
}

const getAgentSetupGuide = (agentId: string, provider?: Provider) => {
  if (!provider?.onboarding) return null
  return provider.onboarding.agentSetupGuides.find(g => g.agentId === agentId)
}

const getAgentEnvVars = (agentId: string, provider?: Provider) => {
  const guide = getAgentSetupGuide(agentId, provider)
  return guide?.envVars || []
}

const getAgentConfigPaths = (agentId: string, provider?: Provider) => {
  const guide = getAgentSetupGuide(agentId, provider)
  if (!guide?.configFilePaths) return ''
  const paths = guide.configFilePaths
  return paths.linux || paths.macos || paths.windows || ''
}

const getCapabilityType = (cap: string) => {
  const types: Record<string, string> = {
    code: 'success',
    reasoning: 'primary',
    'long-context': 'warning',
    'math': 'danger',
    'chinese-optimized': 'info',
    'multimodal': 'warning'
  }
  return types[cap] || 'info'
}

const getCapabilityLabel = (cap: string) => {
  const labels: Record<string, string> = {
    code: '代码生成',
    reasoning: '推理',
    'long-context': '长上下文',
    'math': '数学',
    'chinese-optimized': '中文优化',
    'multimodal': '多模态'
  }
  return labels[cap] || cap
}

const formatContextLength = (len: number) => {
  if (len >= 1000000) return `${(len / 1000000).toFixed(1)}M`
  if (len >= 1000) return `${(len / 1000).toFixed(0)}K`
  return `${len}`
}

const openUrl = (url: string) => {
  window.open(url, '_blank')
}

onMounted(() => {
  loadPlans()
})
</script>

<style scoped>
.plans-view {
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

.plans-header {
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

.add-btn {
  height: 40px;
  padding: 0 20px;
  border-radius: 10px;
  font-weight: 600;
  background: linear-gradient(135deg, #0ea5e9 0%, #06b6d4 100%);
  border: none;
  box-shadow: 0 4px 14px rgba(14, 165, 233, 0.3);
}

.add-btn:hover {
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

/* Plans Grid */
.plans-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(420px, 1fr));
  gap: 20px;
}

/* Form */
.edit-form {
  padding: 4px 0;
}

.form-tip {
  font-size: 12px;
  color: var(--agw-text-secondary);
  margin-top: 4px;
}

.model-option {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 2px 0;
}

.model-name {
  font-weight: 500;
}

.model-id {
  font-size: 11px;
  font-family: var(--agw-font-mono, monospace);
  color: var(--agw-text-secondary);
}

/* Dialog overrides */
:deep(.el-dialog) {
  border-radius: 16px;
}

:deep(.el-dialog__header) {
  border-bottom: 1px solid var(--agw-border-subtle);
  padding: 18px 24px;
  margin-right: 0;
}

:deep(.el-dialog__body) {
  padding: 24px;
}

:deep(.el-dialog__footer) {
  border-top: 1px solid var(--agw-border-subtle);
  padding: 16px 24px;
}

/* Drawer styles */
:deep(.el-drawer) {
  border-radius: 16px 0 0 16px;
}

:deep(.el-drawer__header) {
  padding: 18px 24px;
  margin-bottom: 0;
  border-bottom: 1px solid var(--agw-border-subtle);
}

:deep(.el-drawer__body) {
  padding: 0;
}

.drawer-header {
  display: flex;
  align-items: center;
  gap: 10px;
}

.drawer-title {
  font-weight: 600;
  font-size: 18px;
  color: var(--el-text-color-primary);
}

.drawer-content {
  padding: 24px;
}

/* Detail section styles */
.detail-section {
  margin-bottom: 16px;
  background: var(--el-bg-color);
  border: 1px solid var(--el-border-color-light);
  border-radius: 12px;
  overflow: hidden;
}

.section-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 16px;
  background: var(--el-fill-color-light);
  font-weight: 600;
  font-size: 13px;
  color: var(--el-text-color-primary);
  border-bottom: 1px solid var(--el-border-color-lighter);
}

.section-body {
  padding: 14px 16px;
}

.detail-row {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  margin-bottom: 10px;
  min-height: 28px;
}

.detail-row:last-child {
  margin-bottom: 0;
}

.detail-label {
  color: var(--el-text-color-secondary);
  font-size: 13px;
  min-width: 70px;
  flex-shrink: 0;
  padding-top: 3px;
}

.detail-value {
  color: var(--el-text-color-primary);
  font-size: 14px;
  flex: 1;
}

.detail-value.mono {
  font-family: var(--agw-font-mono, monospace);
}

.detail-value.price-value {
  font-weight: 600;
  color: var(--el-color-primary);
}

.detail-value.description-value {
  color: var(--el-text-color-regular);
  line-height: 1.5;
}

.detail-value.api-key-masked {
  font-family: var(--agw-font-mono, monospace);
  font-size: 13px;
  background: var(--el-fill-color-light);
  padding: 2px 8px;
  border-radius: 4px;
  letter-spacing: 0.5px;
}

.detail-link {
  color: var(--el-color-primary);
  text-decoration: none;
  font-size: 13px;
  display: inline-flex;
  align-items: center;
  gap: 4px;
  transition: color 0.2s;
}

.detail-link:hover {
  color: var(--el-color-primary-light-3);
  text-decoration: underline;
}

.feature-tags,
.capability-tags,
.model-chips {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.model-chip {
  cursor: default;
}

/* Agent binding section */
.agent-binding-row {
  padding: 12px 0;
  border-bottom: 1px solid var(--el-border-color-lighter);
}

.agent-binding-row:last-child {
  border-bottom: none;
}

.agent-binding-header {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 8px;
}

.agent-name {
  font-weight: 600;
  font-size: 14px;
  color: var(--el-text-color-primary);
}

.agent-binding-detail {
  padding-left: 4px;
}

.agent-env-vars {
  margin-bottom: 8px;
}

.env-var-row {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 4px;
}

.env-var-name {
  font-size: 12px;
  color: var(--agw-cyan, #00d4aa);
  font-family: var(--agw-font-mono, monospace);
  min-width: 220px;
}

.env-var-value {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  font-family: var(--agw-font-mono, monospace);
}

.config-path {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  color: var(--el-text-color-secondary);
  margin-top: 4px;
}

.agent-actions {
  display: flex;
  gap: 8px;
  margin-top: 8px;
}

.empty-agents {
  padding: 12px 0;
}

.muted {
  color: var(--el-text-color-muted);
  font-size: 13px;
}

.unbound-section {
  margin-top: 12px;
  padding-top: 12px;
  border-top: 1px dashed var(--el-border-color-light);
}

.unbound-header {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  margin-bottom: 8px;
}

.unbound-agents {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.unbound-tag {
  cursor: pointer;
  transition: all 0.2s ease;
}

.unbound-tag:hover {
  opacity: 0.8;
}

/* API Key section */
.api-key-actions {
  display: flex;
  gap: 8px;
  margin-top: 8px;
}

/* Quota grid */
.quota-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
}

.quota-item {
  padding: 10px 12px;
  background: var(--el-fill-color-light);
  border-radius: 8px;
}

.quota-item-label {
  display: block;
  font-size: 12px;
  color: var(--el-text-color-secondary);
  margin-bottom: 4px;
}

.quota-item-value {
  display: block;
  font-size: 14px;
  font-weight: 600;
  color: var(--el-text-color-primary);
  font-family: var(--agw-font-mono, monospace);
}

.quota-progress {
  margin-top: 6px;
}

/* Detail actions */
.detail-actions {
  display: flex;
  gap: 8px;
  padding-top: 16px;
  border-top: 1px solid var(--el-border-color-lighter);
  margin-top: 4px;
}
</style>