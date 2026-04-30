<template>
  <div class="provider-grid">
    <div
      v-for="provider in providers"
      :key="provider.provider_id"
      class="provider-card"
      :class="{ selected: isSelected(provider) }"
      @click="handleSelect(provider)"
    >
      <div class="provider-icon">
        <el-icon :size="32"><Connection /></el-icon>
      </div>
      <div class="provider-name">{{ provider.name }}</div>
      <div class="provider-desc">{{ provider.description }}</div>
      <div class="provider-plans">
        <el-tag v-for="plan in provider.coding_plans" :key="plan.plan_id" size="small">
          {{ plan.name }}
        </el-tag>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Provider } from '@/types'

const props = defineProps<{
  providers: Provider[]
  selected?: Provider | null
}>()

const emit = defineEmits<{
  select: [provider: Provider]
}>()

const isSelected = (provider: Provider) => {
  return props.selected?.provider_id === provider.provider_id
}

const handleSelect = (provider: Provider) => {
  emit('select', provider)
}
</script>

<style scoped>
.provider-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  gap: 16px;
}

.provider-card {
  border: 2px solid var(--el-border-color);
  border-radius: 12px;
  padding: 20px;
  text-align: center;
  cursor: pointer;
  transition: all 0.3s;
}

.provider-card:hover {
  border-color: var(--el-color-primary);
  transform: translateY(-2px);
}

.provider-card.selected {
  border-color: var(--el-color-primary);
  background: var(--el-color-primary-light-9);
}

.provider-icon {
  margin-bottom: 12px;
  color: var(--el-color-primary);
}

.provider-name {
  font-weight: 600;
  font-size: 16px;
  margin-bottom: 8px;
}

.provider-desc {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  margin-bottom: 12px;
}

.provider-plans {
  display: flex;
  flex-wrap: wrap;
  justify-content: center;
  gap: 4px;
}
</style>