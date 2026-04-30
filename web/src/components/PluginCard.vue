<template>
  <div class="plugin-card" :class="{ 'is-enabled': plugin.status === 'enabled' }">
    <div class="plugin-glow" :class="plugin.status"></div>

    <div class="plugin-header">
      <div class="plugin-info">
        <div class="plugin-icon">
          <el-icon :size="20"><Box /></el-icon>
        </div>
        <div class="plugin-titles">
          <span class="plugin-name">{{ plugin.name }}</span>
          <span class="plugin-version">v{{ plugin.version }}</span>
        </div>
      </div>
      <el-tag
        :type="getStatusType(plugin.status)"
        size="small"
        effect="dark"
        round
        class="status-tag"
      >
        {{ getStatusLabel(plugin.status) }}
      </el-tag>
    </div>

    <div class="plugin-body">
      <p class="plugin-description">{{ plugin.description }}</p>
      <div class="plugin-meta">
        <span class="plugin-author">
          <el-icon><User /></el-icon>
          {{ plugin.author }}
        </span>
      </div>
    </div>

    <div class="plugin-actions">
      <el-button
        v-if="plugin.status === 'enabled'"
        size="default"
        class="action-btn"
        :loading="loading"
        @click="$emit('disable', plugin.id)"
      >
        <el-icon><VideoPause /></el-icon>
        禁用
      </el-button>
      <el-button
        v-else
        size="default"
        type="primary"
        class="action-btn"
        :loading="loading"
        @click="$emit('enable', plugin.id)"
      >
        <el-icon><VideoPlay /></el-icon>
        启用
      </el-button>
      <el-button
        size="default"
        type="danger"
        plain
        class="action-btn"
        :loading="loading"
        @click="$emit('uninstall', plugin.id)"
      >
        <el-icon><Delete /></el-icon>
        卸载
      </el-button>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { PluginInfo } from '@/composables/usePlugins'

defineProps<{
  plugin: PluginInfo
  loading?: boolean
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
  position: relative;
  background: rgba(20, 23, 34, 0.7);
  backdrop-filter: blur(20px);
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 14px;
  padding: 18px;
  overflow: hidden;
  transition: all 0.3s ease;
}

.plugin-card:hover {
  border-color: rgba(255, 255, 255, 0.1);
  transform: translateY(-2px);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
}

.plugin-card.is-enabled {
  border-color: rgba(16, 185, 129, 0.2);
}

.plugin-glow {
  position: absolute;
  top: -50%;
  right: -30%;
  width: 100px;
  height: 100px;
  border-radius: 50%;
  opacity: 0.1;
  pointer-events: none;
}

.plugin-glow.enabled { background: #10b981; }
.plugin-glow.disabled { background: #6b7280; }
.plugin-glow.error { background: #f43f5e; }

.plugin-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 14px;
}

.plugin-info {
  display: flex;
  align-items: center;
  gap: 12px;
}

.plugin-icon {
  width: 40px;
  height: 40px;
  border-radius: 10px;
  background: linear-gradient(135deg, rgba(14, 165, 233, 0.2), rgba(6, 182, 212, 0.15));
  color: #38bdf8;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.plugin-titles {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.plugin-name {
  font-weight: 600;
  font-size: 15px;
  color: #e8eaf0;
}

.plugin-version {
  font-size: 12px;
  color: #6b7280;
  font-family: var(--agw-font-mono, monospace);
}

.status-tag {
  font-weight: 600;
  font-size: 11px;
}

.plugin-body {
  margin-bottom: 16px;
}

.plugin-description {
  font-size: 13px;
  color: #94a3b8;
  margin: 0 0 12px 0;
  line-height: 1.5;
}

.plugin-meta {
  display: flex;
  gap: 12px;
}

.plugin-author {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  color: #6b7280;
}

.plugin-actions {
  display: flex;
  gap: 8px;
  padding-top: 14px;
  border-top: 1px solid rgba(255, 255, 255, 0.05);
}

.action-btn {
  flex: 1;
  border-radius: 8px;
  font-size: 13px;
}
</style>