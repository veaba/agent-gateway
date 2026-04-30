<template>
  <div class="quota-view">
    <el-card>
      <template #header>
        <div class="card-header">
          <span>配额使用情况</span>
        </div>
      </template>

      <el-table :data="quotas" style="width: 100%">
        <el-table-column prop="plan_id" label="套餐" width="150" />
        <el-table-column label="日配额">
          <template #default="{ row }">
            <el-progress :percentage="getPercent(row.daily_used, row.daily_limit)" :stroke-width="10" />
            <span>{{ row.daily_used }} / {{ row.daily_limit }}</span>
          </template>
        </el-table-column>
        <el-table-column label="月配额">
          <template #default="{ row }">
            <el-progress :percentage="getPercent(row.monthly_used, row.monthly_limit)" :stroke-width="10" />
            <span>{{ row.monthly_used }} / {{ row.monthly_limit }}</span>
          </template>
        </el-table-column>
        <el-table-column label="RPM">
          <template #default="{ row }">
            <el-progress :percentage="getPercent(row.rpm_used, row.rpm_limit)" :stroke-width="10" />
            <span>{{ row.rpm_used }} / {{ row.rpm_limit }}</span>
          </template>
        </el-table-column>
        <el-table-column label="操作" width="150">
          <template #default="{ row }">
            <el-button size="small" @click="handleReset(row.plan_id)">重置</el-button>
          </template>
        </el-table-column>
      </el-table>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { ElMessage } from 'element-plus'
import type { QuotaStatus } from '@/types'
import { fetchQuotaStatus } from '@/api'

const quotas = ref<QuotaStatus[]>([])

const getPercent = (used: number, limit: number) => {
  if (!limit) return 0
  return Math.floor((used / limit) * 100)
}

const loadQuotas = async () => {
  try {
    quotas.value = await fetchQuotaStatus()
  } catch (error) {
    ElMessage.error('加载配额失败')
  }
}

const handleReset = async (planId: string) => {
  ElMessage.info('重置功能开发中')
}

onMounted(() => {
  loadQuotas()
})
</script>