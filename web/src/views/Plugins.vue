<template>
  <div class="plugins-view">
    <div class="plugins-header agw-stagger">
      <div class="header-info">
        <h2 class="section-title">插件管理</h2>
        <p class="section-desc">扩展 Agent Gateway 的功能</p>
      </div>
      <el-button type="primary" size="large" class="add-btn" @click="showInstallDialog = true">
        <el-icon><Plus /></el-icon>
        安装插件
      </el-button>
    </div>

    <!-- Loading State -->
    <div v-if="loading && plugins.length === 0" class="loading-state">
      <el-skeleton :rows="3" animated />
    </div>

    <!-- Empty State -->
    <el-empty v-else-if="plugins.length === 0" description="还没有安装任何插件" class="empty-state">
      <template #image>
        <el-icon :size="80" class="empty-icon"><Box /></el-icon>
      </template>
      <template #description>
        <p class="empty-desc">安装插件来扩展 Agent Gateway 的功能</p>
      </template>
      <el-button type="primary" size="large" @click="showInstallDialog = true">
        <el-icon><Plus /></el-icon>
        安装第一个插件
      </el-button>
    </el-empty>

    <!-- Plugins Grid -->
    <div v-else class="plugins-grid agw-stagger">
      <PluginCard
        v-for="plugin in plugins"
        :key="plugin.id"
        :plugin="plugin"
        :loading="actionLoading[plugin.id]"
        @enable="handleEnable"
        @disable="handleDisable"
        @uninstall="handleUninstall"
      />
    </div>

    <!-- Install Dialog -->
    <el-dialog
      v-model="showInstallDialog"
      title="安装插件"
      width="480px"
      :close-on-click-modal="false"
      class="install-dialog"
    >
      <el-form label-position="top">
        <el-form-item label="插件来源">
          <el-input
            v-model="installSource"
            placeholder="本地路径、GitHub URL 或远程 URL"
            clearable
          >
            <template #prepend>
              <el-icon><Link /></el-icon>
            </template>
          </el-input>
          <div class="form-tip">
            支持：本地 .wasm 文件路径、GitHub 仓库 URL、远程 WASM URL
          </div>
        </el-form-item>
      </el-form>

      <template #footer>
        <el-button @click="showInstallDialog = false">取消</el-button>
        <el-button
          type="primary"
          :loading="installing"
          :disabled="!installSource.trim()"
          @click="handleInstall"
        >
          安装
        </el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import PluginCard from '@/components/PluginCard.vue'
import { usePlugins, type PluginInfo } from '@/composables/usePlugins'

const {
  plugins,
  isLoading: loading,
  error,
  loadPlugins,
  install,
  uninstall,
  enable,
  disable
} = usePlugins()

// UI State
const showInstallDialog = ref(false)
const installSource = ref('')
const installing = ref(false)
const actionLoading = reactive<Record<string, boolean>>({})

// Load plugins on mount
onMounted(() => {
  loadPlugins()
})

// Install plugin
const handleInstall = async () => {
  if (!installSource.value.trim()) return

  installing.value = true
  try {
    await install(installSource.value.trim())
    ElMessage.success('插件安装成功')
    showInstallDialog.value = false
    installSource.value = ''
  } catch (e: any) {
    ElMessage.error(e?.message || '插件安装失败')
  } finally {
    installing.value = false
  }
}

// Enable plugin
const handleEnable = async (id: string) => {
  actionLoading[id] = true
  try {
    await enable(id)
    ElMessage.success('插件已启用')
  } catch (e) {
    ElMessage.error('启用插件失败')
  } finally {
    actionLoading[id] = false
  }
}

// Disable plugin
const handleDisable = async (id: string) => {
  actionLoading[id] = true
  try {
    await disable(id)
    ElMessage.success('插件已禁用')
  } catch (e) {
    ElMessage.error('禁用插件失败')
  } finally {
    actionLoading[id] = false
  }
}

// Uninstall plugin
const handleUninstall = async (id: string) => {
  const plugin = plugins.value.find(p => p.id === id)
  try {
    await ElMessageBox.confirm(
      `确定要卸载插件 "${plugin?.name || id}" 吗？`,
      '确认卸载',
      {
        confirmButtonText: '卸载',
        cancelButtonText: '取消',
        type: 'warning'
      }
    )
    actionLoading[id] = true
    await uninstall(id)
    ElMessage.success('插件已卸载')
  } catch (e: any) {
    if (e !== 'cancel') {
      ElMessage.error('卸载插件失败')
    }
  } finally {
    actionLoading[id] = false
  }
}
</script>

<style scoped>
.plugins-view {
  animation: fadeIn 0.5s ease;
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(10px); }
  to { opacity: 1; transform: translateY(0); }
}

.plugins-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
  padding: 20px 24px;
  background: var(--agw-bg-card);
  backdrop-filter: blur(20px);
  border: 1px solid var(--agw-border-default);
  border-radius: 14px;
}

.header-info {
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.section-title {
  font-size: 20px;
  font-weight: 700;
  color: var(--agw-text-primary);
  margin: 0;
}

.section-desc {
  font-size: 13px;
  color: var(--agw-text-secondary);
  margin: 0;
}

.add-btn {
  height: 40px;
  padding: 0 20px;
  border-radius: 10px;
  font-weight: 600;
  background: linear-gradient(135deg, #0ea5e9 0%, #06b6d4 100%);
  border: none;
  box-shadow: 0 4px 14px rgba(14, 165, 233, 0.3);
}

.add-btn:hover {
  transform: translateY(-1px);
  box-shadow: 0 6px 20px rgba(14, 165, 233, 0.4);
}

/* Loading State */
.loading-state {
  padding: 40px;
  background: var(--agw-bg-card);
  border-radius: 14px;
}

/* Empty State */
.empty-state {
  padding: 60px 20px;
  background: var(--agw-bg-card);
  border-radius: 14px;
  border: 1px solid var(--agw-border-subtle);
}

.empty-icon {
  color: var(--agw-text-muted);
  opacity: 0.5;
}

.empty-desc {
  color: var(--agw-text-secondary);
  font-size: 14px;
}

/* Plugins Grid */
.plugins-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
  gap: 16px;
}

/* Form Tip */
.form-tip {
  font-size: 12px;
  color: var(--agw-text-secondary);
  margin-top: 6px;
}
</style>