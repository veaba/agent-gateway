<template>
  <div class="settings-view">
    <el-card>
      <template #header>
        <span>基本设置</span>
      </template>

      <el-form label-width="120px">
        <el-form-item label="监听地址">
          <el-input v-model="settings.listen_address" />
        </el-form-item>
        <el-form-item label="日志级别">
          <el-select v-model="settings.log_level">
            <el-option label="Debug" value="debug" />
            <el-option label="Info" value="info" />
            <el-option label="Warning" value="warn" />
            <el-option label="Error" value="error" />
          </el-select>
        </el-form-item>
        <el-form-item>
          <el-button type="primary" @click="handleSave">保存设置</el-button>
        </el-form-item>
      </el-form>
    </el-card>

    <el-card style="margin-top: 20px">
      <template #header>
        <span>关于</span>
      </template>
      <el-descriptions :column="1" border>
        <el-descriptions-item label="版本">0.1.0</el-descriptions-item>
        <el-descriptions-item label="构建时间">{{ buildTime }}</el-descriptions-item>
        <el-descriptions-item label="Rust 版本">{{ rustVersion }}</el-descriptions-item>
      </el-descriptions>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { ElMessage } from 'element-plus'

const settings = ref({
  listen_address: '127.0.0.1:8080',
  log_level: 'info'
})

const buildTime = '2024-01-15'
const rustVersion = '1.75'

const handleSave = () => {
  ElMessage.success('设置已保存')
}
</script>