<template>
  <div class="plan-wizard">
    <div class="wizard-header">
      <h2 class="wizard-title">添加新套餐</h2>
      <p class="wizard-desc">按照步骤配置您的 AI 服务</p>
    </div>

    <div class="wizard-steps">
      <el-steps :active="currentStep" finish-status="success" align-center>
        <el-step title="选择服务商" />
        <el-step title="选择套餐" />
        <el-step title="选择工具" />
        <el-step title="配置密钥" />
        <el-step title="完成" />
      </el-steps>
    </div>

    <div class="wizard-content">
      <!-- Step 1: 选择 Provider -->
      <div v-show="currentStep === 0" class="step-content">
        <h3 class="step-title">选择 AI 服务商</h3>
        <ProviderGrid :providers="providers" :selected="selectedProvider" @select="handleProviderSelect" />
      </div>

      <!-- Step 2: 选择 Plan -->
      <div v-show="currentStep === 1" class="step-content">
        <h3 class="step-title">选择 {{ selectedProvider?.name }} 的套餐</h3>
        <PlanSelector v-if="selectedProvider" :plans="selectedProvider.codingPlans" :selected="selectedPlanId"
          @select="handlePlanSelect" />
      </div>

      <!-- Step 3: 选择 Agent -->
      <div v-show="currentStep === 2" class="step-content">
        <h3 class="step-title">选择要绑定的 Agent 工具</h3>
        <AgentSelector v-if="selectedPlan" :agents="availableAgents" :selected="selectedAgents"
          @select="handleAgentSelect" />
      </div>

      <!-- Step 4: 配置 API Key -->
      <div v-show="currentStep === 3" class="step-content">
        <h3 class="step-title">配置 API Key</h3>
        <ApiKeyInput v-if="selectedProvider" :provider="selectedProvider" v-model="apiKey"
          @open-page="handleOpenKeyPage" />
      </div>

      <!-- Step 5: 完成 -->
      <div v-show="currentStep === 4" class="step-content step-complete">
        <div class="success-animation">
          <el-icon :size="80" class="success-icon">
            <CircleCheck />
          </el-icon>
        </div>
        <h3 class="complete-title">配置完成</h3>
        <p class="complete-desc">套餐 "{{ planName }}" 已成功配置</p>
        <div class="complete-actions">
          <el-button type="primary" size="large" @click="$router.push('/plans')">
            <el-icon>
              <HomeFilled />
            </el-icon>
            返回套餐列表
          </el-button>
          <el-button size="large" @click="resetWizard">
            <el-icon>
              <Plus />
            </el-icon>
            添加另一个
          </el-button>
        </div>
      </div>
    </div>

    <div class="wizard-footer" v-if="currentStep < 4">
      <el-button v-if="currentStep > 0" size="large" @click="prevStep">
        <el-icon>
          <ArrowLeft />
        </el-icon>
        上一步
      </el-button>
      <el-button v-if="currentStep < 3" type="primary" size="large" :disabled="!canNext" @click="nextStep">
        下一步
        <el-icon>
          <ArrowRight />
        </el-icon>
      </el-button>
      <el-button v-if="currentStep === 3" type="primary" size="large" :disabled="!apiKey" @click="finishWizard">
        完成配置
        <el-icon>
          <Check />
        </el-icon>
      </el-button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import ProviderGrid from '@/components/ProviderGrid.vue'
import PlanSelector from '@/components/PlanSelector.vue'
import AgentSelector from '@/components/AgentSelector.vue'
import ApiKeyInput from '@/components/ApiKeyInput.vue'
import type { Provider, CodingPlan } from '@/types'
import { fetchProviders, createPlan } from '@/api'

const router = useRouter()

const currentStep = ref(0)
const providers = ref<Provider[]>([])
const selectedProvider = ref<Provider | null>(null)
const selectedPlanId = ref('')
const selectedAgents = ref<string[]>([])
const apiKey = ref('')
const planName = ref('')

const selectedPlan = computed(() => {
  if (!selectedProvider.value?.codingPlans) return null
  return selectedProvider.value.codingPlans.find(p => p.planId === selectedPlanId.value)
})

const availableAgents = computed(() => {
  if (!selectedPlan.value || !selectedProvider.value?.supportedAgents) return []
  return selectedProvider.value.supportedAgents.filter(
    a => selectedPlan.value!.supportedAgentIds?.includes(a.agentId)
  )
})

const canNext = computed(() => {
  switch (currentStep.value) {
    case 0: return !!selectedProvider.value
    case 1: return !!selectedPlanId.value
    case 2: return selectedAgents.value.length > 0
    default: return true
  }
})

const handleProviderSelect = (provider: Provider) => {
  selectedProvider.value = provider
}

const handlePlanSelect = (planId: string) => {
  selectedPlanId.value = planId
}

const handleAgentSelect = (agentIds: string[]) => {
  selectedAgents.value = agentIds
}

const handleOpenKeyPage = () => {
  if (selectedProvider.value?.getApiKeyUrl) {
    window.open(selectedProvider.value.getApiKeyUrl, '_blank')
  }
}

const nextStep = () => {
  currentStep.value++
}

const prevStep = () => {
  currentStep.value--
}

const finishWizard = async () => {
  try {
    planName.value = `${selectedProvider.value?.name} ${selectedPlan.value?.name}`
    await createPlan({
      providerId: selectedProvider.value!.providerId,
      planId: selectedPlanId.value,
      name: planName.value,
      apiKey: apiKey.value,
      selectedModelId: selectedPlan.value!.defaultModelId,
      boundAgents: selectedAgents.value.map(id => ({
        agentId: id,
        configured: false,
        configStatus: 'notConfigured'
      }))
    })
    currentStep.value = 4
  } catch {
    ElMessage.error('创建套餐失败')
  }
}

const resetWizard = () => {
  currentStep.value = 0
  selectedProvider.value = null
  selectedPlanId.value = ''
  selectedAgents.value = []
  apiKey.value = ''
  planName.value = ''
}

onMounted(async () => {
  try {
    providers.value = await fetchProviders()
  } catch {
    ElMessage.error('加载服务商列表失败')
  }
})
</script>

<style scoped>
.plan-wizard {
  max-width: 1000px;
  margin: 0 auto;
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

.wizard-header {
  text-align: center;
  margin-bottom: 40px;
}

.wizard-title {
  font-size: 28px;
  font-weight: 600;
  color: var(--el-text-color-primary);
  margin: 0 0 8px;
}

.wizard-desc {
  font-size: 14px;
  color: var(--el-text-color-secondary);
  margin: 0;
}

.wizard-steps {
  background: var(--el-bg-color);
  padding: 32px;
  border-radius: 16px;
  margin-bottom: 24px;
  border: 1px solid var(--el-border-color-light);
}

.wizard-content {
  background: var(--el-bg-color);
  border-radius: 16px;
  padding: 40px;
  min-height: 400px;
  border: 1px solid var(--el-border-color-light);
  margin-bottom: 24px;
}

.step-content {
  animation: slideIn 0.3s ease;
}

@keyframes slideIn {
  from {
    opacity: 0;
    transform: translateX(20px);
  }

  to {
    opacity: 1;
    transform: translateX(0);
  }
}

.step-title {
  font-size: 20px;
  font-weight: 600;
  color: var(--el-text-color-primary);
  margin-bottom: 32px;
  text-align: center;
}

.step-complete {
  text-align: center;
  padding: 40px 0;
}

.success-animation {
  margin-bottom: 24px;
}

.success-icon {
  color: #67c23a;
  animation: scaleIn 0.5s ease;
}

@keyframes scaleIn {
  from {
    transform: scale(0);
    opacity: 0;
  }

  to {
    transform: scale(1);
    opacity: 1;
  }
}

.complete-title {
  font-size: 24px;
  font-weight: 600;
  color: var(--el-text-color-primary);
  margin: 0 0 8px;
}

.complete-desc {
  font-size: 16px;
  color: var(--el-text-color-secondary);
  margin: 0 0 32px;
}

.complete-actions {
  display: flex;
  justify-content: center;
  gap: 16px;
}

.wizard-footer {
  display: flex;
  justify-content: center;
  gap: 16px;
}

.wizard-footer .el-button {
  padding: 12px 32px;
  border-radius: 10px;
}

.wizard-footer .el-button--primary {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border: none;
  box-shadow: 0 4px 15px rgba(102, 126, 234, 0.3);
}
</style>