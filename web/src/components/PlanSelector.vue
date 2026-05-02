<template>
  <div class="plan-selector">
    <div v-for="plan in plans" :key="plan.planId" class="plan-item" :class="{ selected: isSelected(plan.planId) }"
      @click="handleSelect(plan.planId)">
      <div class="plan-header">
        <span class="plan-name">{{ plan.name }}</span>
        <el-tag v-if="plan.tier === 'pro'" type="warning" size="small">推荐</el-tag>
      </div>
      <div class="plan-price">{{ plan.price || '自定义' }}</div>
      <div class="plan-desc">{{ plan.description }}</div>
      <div class="plan-features">
        <div v-if="plan.quotaDaily" class="feature">
          <el-icon>
            <Calendar />
          </el-icon>
          {{ plan.quotaDaily }}次/日
        </div>
        <div v-if="plan.rpmLimit" class="feature">
          <el-icon>
            <Timer />
          </el-icon>
          {{ plan.rpmLimit }} RPM
        </div>
      </div>
      <div class="plan-models">
        <span class="label">支持模型:</span>
        {{ plan.supportedModelIds.join(', ') }}
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { CodingPlan } from '@/types'

const props = defineProps<{
  plans: CodingPlan[]
  selected?: string
}>()

const emit = defineEmits<{
  select: [planId: string]
}>()

const isSelected = (planId: string) => {
  return props.selected === planId
}

const handleSelect = (planId: string) => {
  emit('select', planId)
}
</script>

<style scoped>
.plan-selector {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
  gap: 16px;
}

.plan-item {
  border: 2px solid var(--el-border-color);
  border-radius: 12px;
  padding: 20px;
  cursor: pointer;
  transition: all 0.3s;
}

.plan-item:hover {
  border-color: var(--el-color-primary);
}

.plan-item.selected {
  border-color: var(--el-color-primary);
  background: var(--el-color-primary-light-9);
}

.plan-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.plan-name {
  font-weight: 600;
  font-size: 18px;
}

.plan-price {
  font-size: 24px;
  font-weight: bold;
  color: var(--el-color-primary);
  margin-bottom: 8px;
}

.plan-desc {
  font-size: 14px;
  color: var(--el-text-color-secondary);
  margin-bottom: 12px;
}

.plan-features {
  display: flex;
  gap: 16px;
  margin-bottom: 12px;
}

.feature {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

.plan-models {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

.plan-models .label {
  font-weight: 500;
}
</style>