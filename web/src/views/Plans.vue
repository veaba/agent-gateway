<template>
  <div class="plans-view">
    <div class="plans-header">
      <div class="header-info">
        <h2 class="section-title">我的套餐</h2>
        <p class="section-desc">管理您的 AI 服务配置</p>
      </div>
      <el-button type="primary" size="large" class="add-btn" @click="$router.push('/plans/add')">
        <el-icon><Plus /></el-icon>
        添加套餐
      </el-button>
    </div>

    <el-empty v-if="plans.length === 0" description="还没有配置任何套餐" class="empty-state">
      <template #image>
        <el-icon :size="80" class="empty-icon"><Connection /></el-icon>
      </template>
      <el-button type="primary" size="large" @click="$router.push('/plans/add')">添加第一个套餐</el-button>
    </el-empty>

    <div v-else class="plans-grid">
      <PlanCard
        v-for="plan in plans"
        :key="plan.id"
        :plan="plan"
        @edit="handleEdit"
        @delete="handleDelete"
        @test="handleTest"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import PlanCard from '@/components/PlanCard.vue'
import type { UserPlan } from '@/types'
import { fetchPlans, deletePlan, testPlan } from '@/api'

const plans = ref<UserPlan[]>([])

const loadPlans = async () => {
  try {
    plans.value = await fetchPlans()
  } catch {
    ElMessage.error('加载套餐失败')
  }
}

const handleEdit = (plan: UserPlan) => {
  // TODO: 跳转到编辑页面
  console.log('Edit plan:', plan.id)
}

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
    ElMessage.success('删除成功')
    loadPlans()
  } catch {
    // user cancelled or error
  }
}

const handleTest = async (plan: UserPlan) => {
  try {
    const success = await testPlan(plan.id)
    if (success) {
      ElMessage.success('连接测试成功')
    } else {
      ElMessage.error('连接测试失败')
    }
  } catch {
    ElMessage.error('连接测试失败')
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
  margin-bottom: 32px;
  padding: 24px;
  background: var(--el-bg-color);
  border-radius: 16px;
  border: 1px solid var(--el-border-color-light);
}

.header-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.section-title {
  font-size: 24px;
  font-weight: 600;
  color: var(--el-text-color-primary);
  margin: 0;
}

.section-desc {
  font-size: 14px;
  color: var(--el-text-color-secondary);
  margin: 0;
}

.add-btn {
  height: 44px;
  padding: 0 24px;
  border-radius: 10px;
  font-weight: 500;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border: none;
  box-shadow: 0 4px 15px rgba(102, 126, 234, 0.3);
  transition: all 0.3s ease;
}

.add-btn:hover {
  transform: translateY(-2px);
  box-shadow: 0 6px 20px rgba(102, 126, 234, 0.4);
}

.plans-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(380px, 1fr));
  gap: 24px;
}

.empty-state {
  padding: 80px 20px;
  background: var(--el-bg-color);
  border-radius: 16px;
  border: 1px solid var(--el-border-color-light);
}

.empty-icon {
  color: var(--el-text-color-secondary);
  opacity: 0.3;
}
</style>