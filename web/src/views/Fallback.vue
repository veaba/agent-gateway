<template>
  <div class="fallback-view">
    <el-card>
      <template #header>
        <div class="card-header">
          <span>降级策略配置</span>
        </div>
      </template>

      <el-form label-width="120px">
        <el-form-item label="启用自动降级">
          <el-switch v-model="config.enabled" />
        </el-form-item>

        <el-form-item label="最大重试次数">
          <el-input-number v-model="config.max_attempts" :min="1" :max="5" />
        </el-form-item>

        <el-form-item label="优先级顺序">
          <el-select v-model="config.priority_order" multiple placeholder="选择套餐顺序">
            <el-option v-for="plan in plans" :key="plan.id" :label="plan.name" :value="plan.id" />
          </el-select>
        </el-form-item>

        <el-form-item>
          <el-button type="primary" @click="handleSave">保存配置</el-button>
        </el-form-item>
      </el-form>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { ElMessage } from 'element-plus'
import type { FallbackConfig, UserPlan } from '@/types'
import { fetchFallbackConfig, updateFallbackConfig, fetchPlans } from '@/api'

const config = ref<FallbackConfig>({
  enabled: true,
  max_attempts: 3,
  priority_order: []
})

const plans = ref<UserPlan[]>([])

const loadData = async () => {
  try {
    const [fallbackConfig, planList] = await Promise.all([
      fetchFallbackConfig(),
      fetchPlans()
    ])
    config.value = fallbackConfig
    plans.value = planList
  } catch (error) {
    ElMessage.error('加载配置失败')
  }
}

const handleSave = async () => {
  try {
    await updateFallbackConfig(config.value)
    ElMessage.success('保存成功')
  } catch (error) {
    ElMessage.error('保存失败')
  }
}

onMounted(() => {
  loadData()
})
</script>

<style scoped>
.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}
</style>