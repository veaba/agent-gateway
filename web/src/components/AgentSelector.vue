<template>
  <div class="agent-selector">
    <div
      v-for="agent in agents"
      :key="agent.agent_id"
      class="agent-item"
      :class="{ selected: isSelected(agent.agent_id) }"
      @click="handleToggle(agent.agent_id)"
    >
      <el-checkbox
        :model-value="isSelected(agent.agent_id)"
        size="large"
      />
      <div class="agent-icon">
        <el-icon :size="24"><Robot /></el-icon>
      </div>
      <div class="agent-info">
        <div class="agent-name">{{ agent.name }}</div>
        <div class="agent-id">{{ agent.agent_id }}</div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { AgentRef } from '@/types'

const props = defineProps<{
  agents: AgentRef[]
  selected?: string[]
}>()

const emit = defineEmits<{
  select: [agentIds: string[]]
}>()

const isSelected = (agentId: string) => {
  return props.selected?.includes(agentId) || false
}

const handleToggle = (agentId: string) => {
  const newSelected = isSelected(agentId)
    ? props.selected?.filter(id => id !== agentId)
    : [...(props.selected || []), agentId]

  emit('select', newSelected || [])
}
</script>

<style scoped>
.agent-selector {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: 16px;
}

.agent-item {
  display: flex;
  align-items: center;
  gap: 12px;
  border: 2px solid var(--el-border-color);
  border-radius: 8px;
  padding: 16px;
  cursor: pointer;
  transition: all 0.3s;
}

.agent-item:hover {
  border-color: var(--el-color-primary);
}

.agent-item.selected {
  border-color: var(--el-color-primary);
  background: var(--el-color-primary-light-9);
}

.agent-icon {
  color: var(--el-color-primary);
}

.agent-name {
  font-weight: 600;
}

.agent-id {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}
</style>