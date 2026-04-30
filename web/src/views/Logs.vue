<template>
  <div class="logs-view">
    <el-card>
      <template #header>
        <div class="card-header">
          <span>请求日志</span>
          <el-button size="small" @click="loadLogs">刷新</el-button>
        </div>
      </template>

      <el-table :data="logs" style="width: 100%">
        <el-table-column prop="created_at" label="时间" width="180" />
        <el-table-column prop="plan_id" label="套餐" width="150" />
        <el-table-column prop="agent_id" label="Agent" width="120" />
        <el-table-column prop="model_id" label="模型" width="150" />
        <el-table-column prop="status_code" label="状态" width="100">
          <template #default="{ row }">
            <el-tag :type="row.status_code >= 200 && row.status_code < 300 ? 'success' : 'danger'">
              {{ row.status_code || 'N/A' }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="latency_ms" label="延迟" width="100">
          <template #default="{ row }">
            {{ row.latency_ms ? `${row.latency_ms}ms` : 'N/A' }}
          </template>
        </el-table-column>
      </el-table>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { ElMessage } from 'element-plus'
import type { RequestLog } from '@/types'
import { fetchLogs } from '@/api'

const logs = ref<RequestLog[]>([])

const loadLogs = async () => {
  try {
    logs.value = await fetchLogs(100)
  } catch (error) {
    ElMessage.error('加载日志失败')
  }
}

onMounted(() => {
  loadLogs()
})
</script>

<style scoped>
.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}
</style>