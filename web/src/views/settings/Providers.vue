<template>
  <div class="providers-view">
    <div class="providers-header agw-stagger">
      <div class="header-info">
        <h2 class="section-title">服务商管理</h2>
        <p class="section-desc">查看内置服务商模板与自定义服务商</p>
      </div>
      <div class="header-actions">
        <el-button type="success" size="large" class="add-btn" @click="showCreateDialog">
          <el-icon>
            <Plus />
          </el-icon>
          添加服务商
        </el-button>
        <el-button type="primary" size="large" class="refresh-btn" :loading="isUpdating" @click="handleUpdate">
          <el-icon>
            <Refresh />
          </el-icon>
          更新定义
        </el-button>
        <el-button type="default" size="large" class="reload-btn" :loading="isLoading" @click="loadAllProviders">
          <el-icon>
            <RefreshRight />
          </el-icon>
          刷新
        </el-button>
      </div>
    </div>

    <!-- Loading State -->
    <div v-if="isLoading && allProviders.length === 0" class="loading-state">
      <el-skeleton :rows="4" animated />
    </div>

    <!-- Empty State -->
    <el-empty v-else-if="allProviders.length === 0" description="没有可用的服务商" class="empty-state">
      <template #image>
        <el-icon :size="80" class="empty-icon">
          <Cloudy />
        </el-icon>
      </template>
      <template #description>
        <p class="empty-desc">请检查配置文件或添加自定义服务商</p>
      </template>
      <div class="empty-actions">
        <el-button type="success" size="large" @click="showCreateDialog">
          添加自定义服务商
        </el-button>
        <el-button type="primary" size="large" @click="handleUpdate">
          更新服务商定义
        </el-button>
      </div>
    </el-empty>

    <!-- Providers Grid -->
    <div v-else class="providers-grid agw-stagger">
      <div v-for="provider in allProviders" :key="provider.providerId" class="provider-card" :class="{ 'custom-provider': provider.isCustom }" @click="showProviderDetail(provider)">
        <div class="provider-header">
          <div class="provider-icon">
            <img v-if="provider.logoUrl" :src="provider.logoUrl" :alt="provider.name" class="logo-img" />
            <el-icon v-else :size="32">
              <Cloudy />
            </el-icon>
          </div>
          <div class="provider-title">
            <h3 class="provider-name">{{ provider.name }}</h3>
            <span class="provider-id">{{ provider.providerId }}</span>
          </div>
          <el-tag :type="getApiFormatType(provider.apiFormat)" size="small" class="format-tag">
            {{ provider.apiFormat }}
          </el-tag>
          <!-- Custom provider badge -->
          <el-tag v-if="provider.isCustom" type="success" size="small" class="custom-badge">
            自定义
          </el-tag>
        </div>

        <div class="provider-desc" v-if="provider.description">
          {{ provider.description }}
        </div>

        <!-- Base URL for custom providers -->
        <div class="provider-base-url" v-if="provider.isCustom && provider.baseUrl">
          <span class="base-url-label">Base URL:</span>
          <span class="base-url-value">{{ provider.baseUrl }}</span>
        </div>

        <div class="provider-stats">
          <div class="stat-item">
            <el-icon><ListModel /></el-icon>
            <span class="stat-value">{{ provider.codingPlans?.length || 0 }}</span>
            <span class="stat-label">套餐</span>
          </div>
          <div class="stat-item">
            <el-icon><Cpu /></el-icon>
            <span class="stat-value">{{ provider.models?.length || 0 }}</span>
            <span class="stat-label">模型</span>
          </div>
          <div class="stat-item">
            <el-icon><Robot /></el-icon>
            <span class="stat-value">{{ provider.supportedAgents?.length || 0 }}</span>
            <span class="stat-label">工具</span>
          </div>
        </div>

        <div class="provider-links" v-if="!provider.isCustom">
          <el-link v-if="provider.homepage" :href="provider.homepage" target="_blank" type="primary" :underline="false">
            <el-icon><Link /></el-icon>
            官网
          </el-link>
          <el-link v-if="provider.docsUrl" :href="provider.docsUrl" target="_blank" type="primary" :underline="false">
            <el-icon><Document /></el-icon>
            文档
          </el-link>
          <el-link v-if="provider.getApiKeyUrl" :href="provider.getApiKeyUrl" target="_blank" type="success" :underline="false">
            <el-icon><Key /></el-icon>
            获取密钥
          </el-link>
        </div>

        <!-- Custom provider actions -->
        <div class="custom-actions" v-if="provider.isCustom">
          <el-button type="primary" size="small" plain @click.stop="showEditDialog(provider)">
            <el-icon><Edit /></el-icon>
            编辑
          </el-button>
          <el-button type="danger" size="small" plain @click.stop="handleDeleteCustom(provider)">
            <el-icon><Delete /></el-icon>
            删除
          </el-button>
        </div>

        <div class="view-detail-hint" v-if="!provider.isCustom">
          <el-icon><ArrowRight /></el-icon>
          点击查看详情
        </div>
      </div>
    </div>

    <!-- Provider Detail Dialog -->
    <el-dialog
      v-model="detailDialogVisible"
      :title="selectedProvider?.name + ' - 详情'"
      width="800px"
      :close-on-click-modal="true"
      class="provider-detail-dialog"
    >
      <div v-if="selectedProvider" class="provider-detail-content">
        <!-- Basic Info -->
        <div class="detail-section">
          <h4 class="detail-title">基本信息</h4>
          <div class="detail-grid">
            <div class="detail-item">
              <span class="detail-label">服务商 ID</span>
              <span class="detail-value">{{ selectedProvider.providerId }}</span>
            </div>
            <div class="detail-item">
              <span class="detail-label">API 格式</span>
              <el-tag :type="getApiFormatType(selectedProvider.apiFormat)" size="small">
                {{ selectedProvider.apiFormat }}
              </el-tag>
            </div>
            <div class="detail-item">
              <span class="detail-label">需要 API Key</span>
              <el-tag :type="selectedProvider.requiresApiKey ? 'warning' : 'success'" size="small">
                {{ selectedProvider.requiresApiKey ? '是' : '否' }}
              </el-tag>
            </div>
            <div class="detail-item" v-if="selectedProvider.homepage">
              <span class="detail-label">官网</span>
              <el-link :href="selectedProvider.homepage" target="_blank" type="primary">
                {{ selectedProvider.homepage }}
              </el-link>
            </div>
            <div class="detail-item" v-if="selectedProvider.docsUrl">
              <span class="detail-label">文档</span>
              <el-link :href="selectedProvider.docsUrl" target="_blank" type="primary">
                {{ selectedProvider.docsUrl }}
              </el-link>
            </div>
            <div class="detail-item" v-if="selectedProvider.getApiKeyUrl">
              <span class="detail-label">获取密钥</span>
              <el-link :href="selectedProvider.getApiKeyUrl" target="_blank" type="success">
                {{ selectedProvider.getApiKeyUrl }}
              </el-link>
            </div>
          </div>
          <div class="detail-desc" v-if="selectedProvider.description">
            {{ selectedProvider.description }}
          </div>
        </div>

        <!-- Coding Plans -->
        <div class="detail-section" v-if="selectedProvider.codingPlans?.length">
          <h4 class="detail-title">
            <el-icon><ListModel /></el-icon>
            套餐方案 ({{ selectedProvider.codingPlans.length }})
          </h4>
          <div class="plans-list">
            <div v-for="plan in selectedProvider.codingPlans" :key="plan.planId" class="plan-item">
              <div class="plan-header">
                <span class="plan-name">{{ plan.name }}</span>
                <el-tag :type="getTierType(plan.tier)" size="small">{{ plan.tier }}</el-tag>
                <span class="plan-price" v-if="plan.price">{{ plan.price }}</span>
              </div>
              <div class="plan-desc" v-if="plan.description">{{ plan.description }}</div>
              <div class="plan-meta">
                <span v-if="plan.quotaDaily">日配额: {{ plan.quotaDaily }}</span>
                <span v-if="plan.quotaMonthly">月配额: {{ plan.quotaMonthly }}</span>
                <span v-if="plan.rpmLimit">RPM: {{ plan.rpmLimit }}</span>
              </div>
              <div class="plan-models" v-if="plan.supportedModelIds?.length">
                <span class="meta-label">支持模型:</span>
                <span class="meta-value">{{ plan.supportedModelIds.join(', ') }}</span>
              </div>
              <div class="plan-features" v-if="plan.features?.length">
                <el-tag v-for="feature in plan.features" :key="feature" size="small" type="info" class="feature-tag">
                  {{ feature }}
                </el-tag>
              </div>
            </div>
          </div>
        </div>

        <!-- Models -->
        <div class="detail-section" v-if="selectedProvider.models?.length">
          <h4 class="detail-title">
            <el-icon><Cpu /></el-icon>
            支持模型 ({{ selectedProvider.models.length }})
          </h4>
          <el-table :data="selectedProvider.models" size="small" stripe>
            <el-table-column prop="modelId" label="模型 ID" width="200">
              <template #default="{ row }">
                <span class="model-id">{{ row.modelId }}</span>
              </template>
            </el-table-column>
            <el-table-column prop="name" label="名称" width="180" />
            <el-table-column prop="contextLength" label="上下文长度" width="120">
              <template #default="{ row }">
                <span v-if="row.contextLength">{{ formatNumber(row.contextLength) }}</span>
                <span v-else>-</span>
              </template>
            </el-table-column>
            <el-table-column prop="capabilities" label="能力">
              <template #default="{ row }">
                <el-tag v-for="cap in row.capabilities" :key="cap" size="small" type="info" class="cap-tag">
                  {{ cap }}
                </el-tag>
              </template>
            </el-table-column>
          </el-table>
        </div>

        <!-- Supported Agents -->
        <div class="detail-section" v-if="selectedProvider.supportedAgents?.length">
          <h4 class="detail-title">
            <el-icon><Robot /></el-icon>
            支持工具 ({{ selectedProvider.supportedAgents.length }})
          </h4>
          <div class="agents-list">
            <el-tag v-for="agent in selectedProvider.supportedAgents" :key="agent.agentId" size="default" class="agent-tag">
              {{ agent.name }} ({{ agent.agentId }})
            </el-tag>
          </div>
        </div>

        <!-- Actions -->
        <div class="detail-actions">
          <el-button type="primary" @click="$router.push('/plans/add')">
            <el-icon><Plus /></el-icon>
            创建套餐
          </el-button>
          <el-button v-if="selectedProvider.getApiKeyUrl" type="success" @click="openApiKeyPage(selectedProvider.getApiKeyUrl)">
            <el-icon><Key /></el-icon>
            获取 API Key
          </el-button>
        </div>
      </div>
      <template #footer>
        <el-button @click="detailDialogVisible = false">关闭</el-button>
      </template>
    </el-dialog>

    <!-- Create/Edit Custom Provider Dialog -->
    <el-dialog
      v-model="dialogVisible"
      :title="editingProvider ? '编辑自定义服务商' : '添加自定义服务商'"
      width="600px"
      :close-on-click-modal="false"
    >
      <el-form :model="dialogForm" label-width="120px" class="dialog-form">
        <el-form-item label="服务商名称" required>
          <el-input v-model="dialogForm.name" placeholder="例如：My Custom Provider" />
        </el-form-item>
        <el-form-item label="服务商 ID" required>
          <el-input
            v-model="dialogForm.providerId"
            placeholder="例如：my-custom-provider"
            :disabled="!!editingProvider"
          />
          <div class="form-tip">唯一标识符，创建后不可修改</div>
        </el-form-item>
        <el-form-item label="API 格式" required>
          <el-select v-model="dialogForm.apiFormat" style="width: 100%">
            <el-option label="OpenAI" value="openai" />
            <el-option label="Anthropic" value="anthropic" />
            <el-option label="Custom" value="custom" />
          </el-select>
        </el-form-item>
        <el-form-item label="Base URL" required>
          <el-input v-model="dialogForm.baseUrl" placeholder="https://api.example.com/v1" />
        </el-form-item>
        <el-form-item label="需要 API Key">
          <el-switch v-model="dialogForm.requiresApiKey" />
        </el-form-item>
        <el-form-item label="描述">
          <el-input v-model="dialogForm.description" type="textarea" :rows="2" placeholder="服务商描述..." />
        </el-form-item>
        <el-form-item label="Logo URL">
          <el-input v-model="dialogForm.logoUrl" placeholder="https://..." />
        </el-form-item>
        <el-form-item label="官网">
          <el-input v-model="dialogForm.homepage" placeholder="https://..." />
        </el-form-item>
        <el-form-item label="文档 URL">
          <el-input v-model="dialogForm.docsUrl" placeholder="https://..." />
        </el-form-item>
        <el-form-item label="获取密钥 URL">
          <el-input v-model="dialogForm.getApiKeyUrl" placeholder="https://..." />
        </el-form-item>

        <!-- Models Section -->
        <el-divider content-position="left">模型列表</el-divider>
        <div class="models-section">
          <div v-for="(model, index) in dialogForm.models" :key="model.modelId" class="model-item">
            <div class="model-info">
              <span class="model-id">{{ model.modelId }}</span>
              <span class="model-name">{{ model.name }}</span>
            </div>
            <el-button type="danger" size="small" plain @click="removeModel(index)">
              <el-icon><Delete /></el-icon>
            </el-button>
          </div>
          <el-button type="primary" size="small" plain @click="addModel">
            <el-icon><Plus /></el-icon>
            添加模型
          </el-button>
        </div>
      </el-form>
      <template #footer>
        <el-button @click="dialogVisible = false">取消</el-button>
        <el-button type="primary" @click="handleSaveCustom" :loading="dialogLoading">
          {{ editingProvider ? '保存' : '创建' }}
        </el-button>
      </template>
    </el-dialog>

    <!-- Add Model Dialog -->
    <el-dialog
      v-model="modelDialogVisible"
      title="添加模型"
      width="400px"
      :close-on-click-modal="false"
    >
      <el-form :model="modelForm" label-width="100px" class="dialog-form">
        <el-form-item label="模型 ID" required>
          <el-input v-model="modelForm.modelId" placeholder="例如：gpt-4" />
        </el-form-item>
        <el-form-item label="模型名称" required>
          <el-input v-model="modelForm.name" placeholder="例如：GPT-4" />
        </el-form-item>
        <el-form-item label="描述">
          <el-input v-model="modelForm.description" placeholder="模型描述..." />
        </el-form-item>
        <el-form-item label="上下文长度">
          <el-input-number v-model="modelForm.contextLength" :min="0" :step="1000" style="width: 100%" />
        </el-form-item>
        <el-form-item label="能力标签">
          <el-input v-model="modelForm.capabilities" placeholder="chat,completion,streaming (逗号分隔)" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="modelDialogVisible = false">取消</el-button>
        <el-button type="primary" @click="saveModel">添加</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { useProviders } from '@/composables/useProviders'
import type { Provider, CustomProvider, CustomModel } from '@/types'

const {
  allProviders,
  isLoading,
  isUpdating,
  loadAllProviders,
  updateProviderDefs,
  createCustom,
  updateCustom,
  deleteCustom
} = useProviders()

const detailDialogVisible = ref(false)
const selectedProvider = ref<Provider | null>(null)

// Dialog state for create/edit
const dialogVisible = ref(false)
const dialogLoading = ref(false)
const editingProvider = ref<CustomProvider | null>(null)
const dialogForm = reactive({
  providerId: '',
  name: '',
  apiFormat: 'openai',
  baseUrl: '',
  requiresApiKey: true,
  description: '',
  logoUrl: '',
  homepage: '',
  docsUrl: '',
  getApiKeyUrl: '',
  models: [] as CustomModel[]
})

// Model form for adding models
const modelDialogVisible = ref(false)
const modelForm = reactive({
  modelId: '',
  name: '',
  description: '',
  contextLength: 0,
  capabilities: ''
})

const loadData = async () => {
  await loadAllProviders()
}

const handleUpdate = async () => {
  const success = await updateProviderDefs()
  if (success) {
    ElMessage.success('服务商定义已更新')
  } else {
    ElMessage.error('更新失败')
  }
}

const showProviderDetail = (provider: Provider) => {
  // 自定义 Provider 不显示详情弹窗，而是编辑
  if (provider.isCustom) {
    showEditDialog(provider as unknown as CustomProvider)
  } else {
    selectedProvider.value = provider
    detailDialogVisible.value = true
  }
}

// Create/Edit Custom Provider Dialog
const showCreateDialog = () => {
  editingProvider.value = null
  dialogForm.providerId = ''
  dialogForm.name = ''
  dialogForm.apiFormat = 'openai'
  dialogForm.baseUrl = ''
  dialogForm.requiresApiKey = true
  dialogForm.description = ''
  dialogForm.logoUrl = ''
  dialogForm.homepage = ''
  dialogForm.docsUrl = ''
  dialogForm.getApiKeyUrl = ''
  dialogForm.models = []
  dialogVisible.value = true
}

const showEditDialog = (provider: CustomProvider) => {
  editingProvider.value = provider
  dialogForm.providerId = provider.providerId
  dialogForm.name = provider.name
  dialogForm.apiFormat = provider.apiFormat
  dialogForm.baseUrl = provider.baseUrl
  dialogForm.requiresApiKey = provider.requiresApiKey
  dialogForm.description = provider.description || ''
  dialogForm.logoUrl = provider.logoUrl || ''
  dialogForm.homepage = provider.homepage || ''
  dialogForm.docsUrl = provider.docsUrl || ''
  dialogForm.getApiKeyUrl = provider.getApiKeyUrl || ''
  dialogForm.models = provider.models ? [...provider.models] : []
  dialogVisible.value = true
}

const addModel = () => {
  modelForm.modelId = ''
  modelForm.name = ''
  modelForm.description = ''
  modelForm.contextLength = 0
  modelForm.capabilities = ''
  modelDialogVisible.value = true
}

const saveModel = () => {
  if (!modelForm.modelId.trim() || !modelForm.name.trim()) {
    ElMessage.warning('模型 ID 和名称不能为空')
    return
  }
  dialogForm.models.push({
    modelId: modelForm.modelId.trim(),
    name: modelForm.name.trim(),
    description: modelForm.description.trim() || undefined,
    contextLength: modelForm.contextLength || undefined,
    capabilities: modelForm.capabilities.split(',').map(c => c.trim()).filter(c => c)
  })
  modelDialogVisible.value = false
}

const removeModel = (index: number) => {
  dialogForm.models.splice(index, 1)
}

const handleSaveCustom = async () => {
  if (!dialogForm.providerId.trim()) {
    ElMessage.warning('请输入服务商 ID')
    return
  }
  if (!dialogForm.name.trim()) {
    ElMessage.warning('请输入服务商名称')
    return
  }
  if (!dialogForm.baseUrl.trim()) {
    ElMessage.warning('请输入 Base URL')
    return
  }

  dialogLoading.value = true
  try {
    if (editingProvider.value) {
      // Update existing
      const result = await updateCustom(editingProvider.value.id, {
        name: dialogForm.name.trim(),
        apiFormat: dialogForm.apiFormat,
        baseUrl: dialogForm.baseUrl.trim(),
        requiresApiKey: dialogForm.requiresApiKey,
        description: dialogForm.description.trim() || undefined,
        logoUrl: dialogForm.logoUrl.trim() || undefined,
        homepage: dialogForm.homepage.trim() || undefined,
        docsUrl: dialogForm.docsUrl.trim() || undefined,
        getApiKeyUrl: dialogForm.getApiKeyUrl.trim() || undefined,
        models: dialogForm.models
      })
      if (result) {
        ElMessage.success('自定义服务商已更新')
        dialogVisible.value = false
      }
    } else {
      // Create new
      const result = await createCustom({
        providerId: dialogForm.providerId.trim(),
        name: dialogForm.name.trim(),
        apiFormat: dialogForm.apiFormat,
        baseUrl: dialogForm.baseUrl.trim(),
        requiresApiKey: dialogForm.requiresApiKey,
        description: dialogForm.description.trim() || undefined,
        logoUrl: dialogForm.logoUrl.trim() || undefined,
        homepage: dialogForm.homepage.trim() || undefined,
        docsUrl: dialogForm.docsUrl.trim() || undefined,
        getApiKeyUrl: dialogForm.getApiKeyUrl.trim() || undefined,
        models: dialogForm.models
      })
      if (result) {
        ElMessage.success('自定义服务商已创建')
        dialogVisible.value = false
      }
    }
  } catch (e: any) {
    ElMessage.error(e.message || '操作失败')
  } finally {
    dialogLoading.value = false
  }
}

const handleDeleteCustom = async (provider: any) => {
  try {
    await ElMessageBox.confirm(
      `确定要删除自定义服务商 "${provider.name}" 吗？`,
      '删除确认',
      {
        confirmButtonText: '删除',
        cancelButtonText: '取消',
        type: 'warning',
      }
    )
    const success = await deleteCustom(provider.id)
    if (success) {
      ElMessage.success('自定义服务商已删除')
    }
  } catch {
    // User cancelled
  }
}

const getApiFormatType = (format: string): 'success' | 'warning' | 'info' | 'primary' => {
  switch (format) {
    case 'anthropic': return 'primary'
    case 'openai': return 'success'
    default: return 'info'
  }
}

const getTierType = (tier: string): 'success' | 'warning' | 'info' | 'danger' => {
  switch (tier) {
    case 'free': return 'success'
    case 'pro': return 'warning'
    case 'enterprise': return 'primary'
    default: return 'info'
  }
}

const formatNumber = (num: number): string => {
  if (num >= 1000000) return (num / 1000000).toFixed(1) + 'M'
  if (num >= 1000) return (num / 1000).toFixed(1) + 'K'
  return num.toString()
}

const openApiKeyPage = (url: string) => {
  window.open(url, '_blank')
}

onMounted(() => {
  loadData()
})
</script>

<style scoped>
.providers-view {
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

.providers-header {
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

.refresh-btn {
  height: 40px;
  padding: 0 20px;
  border-radius: 10px;
  font-weight: 600;
  background: linear-gradient(135deg, #22c55e 0%, #16a34a 100%);
  border: none;
  box-shadow: 0 4px 14px rgba(34, 197, 94, 0.3);
}

.refresh-btn:hover {
  transform: translateY(-1px);
  box-shadow: 0 6px 20px rgba(34, 197, 94, 0.4);
}

.reload-btn {
  height: 40px;
  padding: 0 20px;
  border-radius: 10px;
  font-weight: 600;
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

/* Providers Grid */
.providers-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(340px, 1fr));
  gap: 20px;
}

.provider-card {
  background: var(--agw-bg-card);
  border: 1px solid var(--agw-border-default);
  border-radius: 14px;
  padding: 20px;
  cursor: pointer;
  transition: all 0.3s ease;
}

.provider-card:hover {
  border-color: var(--agw-border-hover);
  transform: translateY(-2px);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.1);
}

.provider-header {
  display: flex;
  align-items: center;
  gap: 16px;
  margin-bottom: 12px;
}

.provider-icon {
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

.provider-title {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.provider-name {
  font-size: 16px;
  font-weight: 600;
  color: var(--agw-text-primary);
  margin: 0;
}

.provider-id {
  font-size: 12px;
  font-family: var(--agw-font-mono, monospace);
  color: var(--agw-text-secondary);
}

.format-tag {
  margin-left: auto;
}

.provider-desc {
  font-size: 13px;
  color: var(--agw-text-secondary);
  line-height: 1.5;
  margin-bottom: 16px;
}

.provider-stats {
  display: flex;
  gap: 20px;
  margin-bottom: 16px;
}

.stat-item {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  color: var(--agw-text-secondary);
}

.stat-item .el-icon {
  font-size: 16px;
  color: var(--agw-text-muted);
}

.stat-value {
  font-weight: 600;
  color: var(--agw-text-primary);
}

.stat-label {
  color: var(--agw-text-muted);
}

.provider-links {
  display: flex;
  gap: 16px;
  margin-bottom: 12px;
}

.provider-links .el-link {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 13px;
}

.view-detail-hint {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 4px;
  font-size: 12px;
  color: var(--agw-text-muted);
  padding-top: 8px;
  border-top: 1px solid var(--agw-border-subtle);
}

/* Provider Detail Dialog */
.provider-detail-content {
  max-height: 600px;
  overflow-y: auto;
}

.detail-section {
  margin-bottom: 24px;
}

.detail-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 16px;
  font-weight: 600;
  color: var(--agw-text-primary);
  margin: 0 0 16px 0;
  padding-bottom: 8px;
  border-bottom: 1px solid var(--agw-border-subtle);
}

.detail-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 12px;
}

.detail-item {
  display: flex;
  align-items: center;
  gap: 8px;
}

.detail-label {
  font-size: 13px;
  color: var(--agw-text-muted);
  min-width: 80px;
}

.detail-value {
  font-size: 13px;
  color: var(--agw-text-primary);
  font-family: var(--agw-font-mono, monospace);
}

.detail-desc {
  font-size: 14px;
  color: var(--agw-text-secondary);
  line-height: 1.6;
  margin-top: 12px;
  padding: 12px;
  background: var(--agw-bg-subtle);
  border-radius: 8px;
}

/* Plans List */
.plans-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.plan-item {
  padding: 16px;
  background: var(--agw-bg-subtle);
  border-radius: 10px;
}

.plan-header {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 8px;
}

.plan-name {
  font-size: 15px;
  font-weight: 600;
  color: var(--agw-text-primary);
}

.plan-price {
  font-size: 13px;
  color: var(--agw-text-secondary);
  margin-left: auto;
}

.plan-desc {
  font-size: 13px;
  color: var(--agw-text-secondary);
  margin-bottom: 8px;
}

.plan-meta {
  display: flex;
  gap: 12px;
  font-size: 12px;
  color: var(--agw-text-muted);
  margin-bottom: 8px;
}

.plan-models {
  font-size: 12px;
  margin-bottom: 8px;
}

.meta-label {
  color: var(--agw-text-muted);
}

.meta-value {
  color: var(--agw-text-secondary);
}

.plan-features {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.feature-tag {
  font-size: 11px;
}

/* Models Table */
.model-id {
  font-family: var(--agw-font-mono, monospace);
  font-size: 12px;
}

.cap-tag {
  font-size: 11px;
  margin-right: 4px;
}

/* Agents List */
.agents-list {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.agent-tag {
  font-size: 13px;
}

/* Detail Actions */
.detail-actions {
  display: flex;
  gap: 12px;
  margin-top: 24px;
  padding-top: 16px;
  border-top: 1px solid var(--agw-border-subtle);
}

/* Dialog */
:deep(.el-dialog__body) {
  padding: 20px;
}

/* Add Button */
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

/* Custom Provider Styles */
.provider-card.custom-provider {
  border-color: rgba(34, 197, 94, 0.3);
}

.provider-card.custom-provider:hover {
  border-color: rgba(34, 197, 94, 0.5);
}

.provider-card.custom-provider .provider-icon {
  background: linear-gradient(135deg, #22c55e 0%, #16a34a 100%);
}

.custom-badge {
  margin-left: 8px;
}

.provider-base-url {
  margin-bottom: 12px;
  font-size: 12px;
}

.base-url-label {
  color: var(--agw-text-muted);
}

.base-url-value {
  color: var(--agw-text-secondary);
  font-family: var(--agw-font-mono, monospace);
}

.custom-actions {
  display: flex;
  gap: 8px;
  margin-bottom: 12px;
}

/* Dialog Form */
.dialog-form {
  padding: 20px 0;
}

.form-tip {
  font-size: 12px;
  color: var(--agw-text-muted);
  margin-top: 4px;
}

/* Models Section */
.models-section {
  margin-top: 16px;
}

.model-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  background: var(--agw-bg-subtle);
  border-radius: 8px;
  margin-bottom: 8px;
}

.model-info {
  display: flex;
  align-items: center;
  gap: 12px;
}

.model-info .model-id {
  font-family: var(--agw-font-mono, monospace);
  font-size: 12px;
  color: var(--agw-text-secondary);
}

.model-info .model-name {
  font-size: 13px;
  color: var(--agw-text-primary);
}

/* Divider */
:deep(.el-divider) {
  margin: 16px 0;
}
</style>