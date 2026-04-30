<template>
  <div class="plugins-view">
    <div class="plugins-header">
      <el-button type="primary" @click="handleInstall">
        <el-icon><Plus /></el-icon>
        安装插件
      </el-button>
    </div>

    <el-empty v-if="plugins.length === 0" description="还没有安装任何插件" />

    <div v-else class="plugins-grid">
      <PluginCard
        v-for="plugin in plugins"
        :key="plugin.id"
        :plugin="plugin"
        @enable="handleEnable"
        @disable="handleDisable"
        @uninstall="handleUninstall"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { ElMessage } from 'element-plus'
import PluginCard from '@/components/PluginCard.vue'

const plugins = ref<any[]>([])

const handleInstall = () => {
  ElMessage.info('插件安装功能开发中')
}

const handleEnable = (id: string) => {
  ElMessage.success('插件已启用')
}

const handleDisable = (id: string) => {
  ElMessage.info('插件已禁用')
}

const handleUninstall = (id: string) => {
  ElMessage.info('插件已卸载')
}
</script>

<style scoped>
.plugins-header {
  margin-bottom: 20px;
}

.plugins-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 20px;
}
</style>