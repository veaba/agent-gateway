<template>
  <div class="plans-view">
    <div class="plans-header agw-stagger">
      <div class="header-info">
        <h2 class="section-title">我的套餐</h2>
        <p class="section-desc">管理您的 AI 服务配置</p>
      </div>
      <el-button type="primary" size="large" class="add-btn" @click="$router.push('/plans/add')">
        <el-icon><Plus /></el-icon>
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
        <el-icon :size="80" class="empty-icon"><Connection /></el-icon>
      </template>
      <template #description>
        <p class="empty-desc">添加第一个 AI 套餐开始使用</p>
      </template>
      <el-button type="primary" size="large" @click="$router.push('/plans/add')">
        <el-icon><Plus /></el-icon>
        添加第一个套餐
      </el-button>
    </el-empty>

    <!-- Plans Grid -->
    <div v-else class="plans-grid agw-stagger">
      <PlanCard
        v-for="plan in plans"
        :key="plan.id"
        :plan="plan"
        :default-plan-id="defaultPlanId"
        :provider="getProviderForPlan(plan)"
        @edit="handleEdit"
        @delete="handleDelete"
        @test="handleTest"
        @set-default="handleSetDefault"
        @bind-agent="handleBindAgent"
        @auto-config="handleAutoConfig"
        @test-agent="handleTestAgent"
      />
    </div>

    <!-- Edit Dialog -->
    <el-dialog
      v-model="editDialogVisible"
      title="编辑套餐"
      width="560px"
      :close-on-click-modal="false"
      class="edit-dialog"
    >
      <el-form
        ref="editFormRef"
        :model="editForm"
        :rules="editRules"
        label-position="top"
        class="edit-form"
      >
        <el-form-item label="套餐名称" prop="name">
          <el-input v-model="editForm.name" placeholder="给套餐起个名字" clearable />
        </el-form-item>

        <el-form-item label="API Key" prop="api_key">
          <el-input
            v-model="editForm.api_key"
            type="password"
            show-password
            placeholder="留空则保持原值，填写则更新"
          />
          <div class="form-tip">留空则保持原 API Key 不变</div>
        </el-form-item>

        <el-form-item label="模型" prop="selected_model_id">
          <el-select v-model="editForm.selected_model_id" placeholder="选择模型" clearable style="width: 100%">
            <el-option
              v-for="model in availableModels"
              :key="model.model_id"
              :label="model.name"
              :value="model.model_id"
            >
              <div class="model-option">
                <span class="model-name">{{ model.name }}</span>
                <span class="model-id">{{ model.model_id }}</span>
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
              <el-input-number
                v-model="editForm.priority"
                :min="1"
                :max="100"
                controls-position="right"
                style="width: 100%"
              />
            </el-form-item>
          </el-col>
        </el-row>

        <el-divider content-position="left">自定义配额</el-divider>

        <el-row :gutter="16">
          <el-col :span="8">
            <el-form-item label="日配额上限">
              <el-input-number
                v-model="editForm.custom_quota_daily"
                :min="0"
                controls-position="right"
                style="width: 100%"
                placeholder="无限制"
              />
            </el-form-item>
          </el-col>
          <el-col :span="8">
            <el-form-item label="月配额上限">
              <el-input-number
                v-model="editForm.custom_quota_monthly"
                :min="0"
                controls-position="right"
                style="width: 100%"
                placeholder="无限制"
              />
            </el-form-item>
          </el-col>
          <el-col :span="8">
            <el-form-item label="RPM 限制">
              <el-input-number
                v-model="editForm.custom_rpm_limit"
                :min="0"
                controls-position="right"
                style="width: 100%"
                placeholder="无限制"
              />
            </el-form-item>
          </el-col>
        </el-row>

        <el-form-item label="备注" prop="notes">
          <el-input
            v-model="editForm.notes"
            type="textarea"
            :rows="2"
            placeholder="可选备注信息"
          />
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
import type { UserPlan, Provider, Model } from '@/types'
import { fetchPlans, deletePlan, testPlan, updatePlan, fetchProvider, fetchProviders } from '@/api'

const plans = ref<UserPlan[]>([])
const providers = ref<Provider[]>([])
const loading = ref(false)
const editDialogVisible = ref(false)
const saving = ref(false)
const editFormRef = ref<FormInstance>()
const editingPlan = ref<UserPlan | null>(null)
const availableModels = ref<Model[]>([])

const editForm = reactive({
  name: '',
  api_key: '',
  selected_model_id: '',
  enabled: true,
  priority: 1,
  custom_quota_daily: 0,
  custom_quota_monthly: 0,
  custom_rpm_limit: 0,
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
  return providers.value.find(p => p.provider_id === plan.provider_id)
}

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
  await loadProviderModels(plan.provider_id)

  editForm.name = plan.name
  editForm.api_key = ''
  editForm.selected_model_id = plan.selected_model_id || ''
  editForm.enabled = plan.enabled
  editForm.priority = plan.priority || 1
  editForm.custom_quota_daily = plan.custom_quota_daily || 0
  editForm.custom_quota_monthly = plan.custom_quota_monthly || 0
  editForm.custom_rpm_limit = plan.custom_rpm_limit || 0
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

      if (editForm.api_key.trim()) {
        updateData.api_key = editForm.api_key.trim()
      }

      if (editForm.selected_model_id) {
        updateData.selected_model_id = editForm.selected_model_id
      }

      if (editForm.custom_quota_daily > 0) {
        updateData.custom_quota_daily = editForm.custom_quota_daily
      }

      if (editForm.custom_quota_monthly > 0) {
        updateData.custom_quota_monthly = editForm.custom_quota_monthly
      }

      if (editForm.custom_rpm_limit > 0) {
        updateData.custom_rpm_limit = editForm.custom_rpm_limit
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
    await api.post(`/plans/${planId}/agents/${agentId}/bind`, { auto_config: false })
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

onMounted(() => {
  loadPlans()
})
</script>

<style scoped>
.plans-view {
  animation: fadeIn 0.5s ease;
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(10px); }
  to { opacity: 1; transform: translateY(0); }
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
</style>