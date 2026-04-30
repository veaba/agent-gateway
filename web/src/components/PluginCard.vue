<template>
  <div class="plugin-card">
    <div class="plugin-header">
      <span class="plugin-name">{{ plugin.name }}</span>
      <el-tag :type="getStatusType(plugin.status)" size="small">
        {{ getStatusLabel(plugin.status) }}
      </el-tag>
    </div>
    <div class="plugin-body">
      <div class="plugin-info">
        <span class="version">v{{ plugin.version }}</span>
        <span class="author">by {{ plugin.author }}</span>
      </div>
      <p class="description">{{ plugin.description }}</p>
    </div>
    <div class="plugin-actions">
      <el-button v-if="plugin.status === 'enabled'" size="small" @click="$emit('disable', plugin.id)">
        禁用
      </el-button>
      <el-button v-else size="small" type="primary" @click="$emit('enable', plugin.id)">
        启用
      </el-button>
      <el-button size="small" type="danger" @click="$emit('uninstall', plugin.id)">
        卸载
      </el-button>
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  plugin: any
}>()

defineEmits<{
  enable: [id: string]
  disable: [id: string]
  uninstall: [id: string]
}>()

const getStatusType = (status: string) => {
  const types: Record<string, string> = {
    enabled: 'success',
    disabled: 'info',
    error: 'danger'
  }
  return types[status] || 'info'
}

const getStatusLabel = (status: string) => {
  const labels: Record<string, string> = {
    enabled: '已启用',
    disabled: '已禁用',
    error: '错误'
  }
  return labels[status] || status
}
</script>

<style scoped>
.plugin-card {
  background: var(--el-bg-color);
  border: 1px solid var(--el-border-color);
  border-radius: 8px;
  padding: 16px;
}

.plugin-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.plugin-name {
  font-weight: 600;
  font-size: 16px;
}

.plugin-body {
  margin-bottom: 12px;
}

.plugin-info {
  display: flex;
  gap: 12px;
  font-size: 12px;
  color: var(--el-text-color-secondary);
  margin-bottom: 8px;
}

.description {
  font-size: 14px;
  color: var(--el-text-color-secondary);
  margin: 0;
}

.plugin-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}
</style>