<template>
  <div class="app-container agw-grid-bg">
    <el-container class="app-layout">
      <el-aside width="248px" class="sidebar">
        <div class="logo-container">
          <div class="logo-icon">
            <el-icon :size="26">
              <DataBoard />
            </el-icon>
          </div>
          <div class="logo-text-group">
            <span class="logo-text">Agent Gateway</span>
            <span class="logo-sub">AI Proxy Hub</span>
          </div>
        </div>

        <el-menu :default-active="$route.path" router class="sidebar-menu" :popper-class="'sidebar-popper'">
          <el-menu-item index="/">
            <el-icon>
              <HomeFilled />
            </el-icon>
            <span>仪表盘</span>
          </el-menu-item>
          <el-menu-item index="/plans">
            <el-icon>
              <Connection />
            </el-icon>
            <span>我的套餐</span>
          </el-menu-item>
          <el-menu-item index="/fallback">
            <el-icon>
              <RefreshLeft />
            </el-icon>
            <span>降级策略</span>
          </el-menu-item>
          <el-menu-item index="/quota">
            <el-icon>
              <DataLine />
            </el-icon>
            <span>配额使用</span>
          </el-menu-item>
          <el-menu-item index="/stats">
            <el-icon>
              <TrendCharts />
            </el-icon>
            <span>统计数据</span>
          </el-menu-item>
          <el-menu-item index="/logs">
            <el-icon>
              <Tickets />
            </el-icon>
            <span>请求日志</span>
          </el-menu-item>
          <el-menu-item index="/plugins">
            <el-icon>
              <Box />
            </el-icon>
            <span>插件管理</span>
          </el-menu-item>
          <div class="menu-divider"></div>
          <el-menu-item index="/guide">
            <el-icon>
              <Guide />
            </el-icon>
            <span>配置引导</span>
          </el-menu-item>
          <el-sum-menu index=''>
            <template #title>
              <el-icon>
                <location />
              </el-icon>
              <span>Navigator One</span>
            </template>
          </el-sum-menu>
          <el-sub-menu index="1">
            <template #title>
              <el-icon>
                <Setting />
              </el-icon>
              <span>设置</span>
            </template>
            <el-menu-item-group>
              <el-menu-item index="/settings">通用配置</el-menu-item>
              <el-menu-item index="/settings/agents">Agents</el-menu-item>
              <el-menu-item index="/settings/providers">Providers</el-menu-item>
            </el-menu-item-group>
          </el-sub-menu>
        </el-menu>

        <div class="sidebar-footer">
          <div class="gateway-status" :class="{ 'is-online': isConnected }">
            <span class="status-dot"></span>
            <span class="status-text">{{ isConnected ? '服务在线' : '服务离线' }}</span>
          </div>
          <div class="version-info">v0.1.0</div>
        </div>

        <div class="sidebar-glow"></div>
      </el-aside>

      <el-container class="main-container">
        <el-header class="app-header">
          <div class="header-left">
            <h1 class="page-title">{{ pageTitle }}</h1>
            <span v-if="breadcrumb" class="page-breadcrumb">{{ breadcrumb }}</span>
          </div>
          <div class="header-right">
            <el-badge :value="notificationCount" :hidden="notificationCount === 0" class="header-badge">
              <el-button circle :icon="Bell" class="icon-btn" />
            </el-badge>
            <el-button type="primary" class="add-btn" @click="$router.push('/plans/add')">
              <el-icon>
                <Plus />
              </el-icon>
              添加套餐
            </el-button>
          </div>
        </el-header>

        <el-main class="app-main">
          <router-view v-slot="{ Component }">
            <transition name="fade-slide" mode="out-in">
              <component :is="Component" v-if="Component" :key="$route.path" />

            </transition>
          </router-view>
        </el-main>
        <p class='copyright'>copyright © 2026 Agent Gateway. All rights reserved.</p>
      </el-container>
    </el-container>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import { Bell } from '@element-plus/icons-vue'
import { healthCheck } from '@/api'
import { useTheme } from '@/composables/useTheme'

// Initialize theme globally (handles auto mode system preference listener)
useTheme()

const route = useRoute()
const isConnected = ref(false)
const notificationCount = ref(0)

const pageTitle = computed(() => {
  const titles: Record<string, string> = {
    '/': '仪表盘',
    '/plans': '我的套餐',
    '/plans/add': '添加套餐',
    '/fallback': '降级策略',
    '/quota': '配额使用',
    '/stats': '统计数据',
    '/logs': '请求日志',
    '/plugins': '插件管理',
    '/settings': '设置'
  }
  return titles[route.path] || 'Agent Gateway'
})

const breadcrumb = computed(() => {
  const crumbs: Record<string, string> = {
    '/plans/add': '创建新的 AI 服务配置'
  }
  return crumbs[route.path] || ''
})

const checkHealth = async () => {
  isConnected.value = await healthCheck()
}

onMounted(() => {
  checkHealth()
  // Check health every 30 seconds
  setInterval(checkHealth, 30000)
})
</script>

<style scoped>
.app-container {
  width: 100vw;
  height: 100vh;
  overflow: hidden;
}

.app-layout {
  height: 100%;
}

/* ── Sidebar ── */
.sidebar {
  background: var(--agw-bg-card);
  display: flex;
  flex-direction: column;
  position: relative;
  overflow: hidden;
  border-right: 1px solid var(--agw-border-subtle);
}

.sidebar-glow {
  position: absolute;
  top: -100px;
  left: 50%;
  transform: translateX(-50%);
  width: 200px;
  height: 200px;
  background: radial-gradient(ellipse, rgba(14, 165, 233, 0.08) 0%, transparent 70%);
  pointer-events: none;
}

.logo-container {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 20px 18px;
  border-bottom: 1px solid var(--agw-border-subtle);
}

.logo-icon {
  width: 40px;
  height: 40px;
  background: linear-gradient(135deg, #0ea5e9 0%, #06b6d4 100%);
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  box-shadow: 0 4px 16px rgba(14, 165, 233, 0.3);
  flex-shrink: 0;
}

.logo-text-group {
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.logo-text {
  font-size: 15px;
  font-weight: 700;
  color: var(--agw-text-primary);
  letter-spacing: -0.3px;
  line-height: 1.2;
}

.logo-sub {
  font-size: 10px;
  font-weight: 500;
  color: var(--agw-text-muted);
  text-transform: uppercase;
  letter-spacing: 0.08em;
}

.sidebar-menu {
  flex: 1;
  background: transparent;
  border: none;
  padding: 12px 8px;
  overflow-y: auto;
}

.menu-divider {
  height: 1px;
  background: var(--agw-border-subtle);
  margin: 8px 12px;
}

.sidebar-menu :deep(.el-menu-item) {
  height: 44px;
  line-height: 44px;
  margin: 2px 0;
  border-radius: 10px;
  color: var(--agw-text-secondary);
  font-size: 14px;
  transition: all 0.25s ease;
  position: relative;
}

.sidebar-menu :deep(.el-menu-item:hover) {
  background: var(--agw-bg-hover);
  color: var(--agw-text-primary);
}

.sidebar-menu :deep(.el-menu-item.is-active) {
  background: var(--agw-sky-dim);
  color: var(--agw-sky);
  font-weight: 600;
}

.sidebar-menu :deep(.el-menu-item .el-icon) {
  font-size: 17px;
  margin-right: 10px;
  width: 20px;
}

/* ── Sidebar Footer ── */
.sidebar-footer {
  padding: 14px 18px;
  border-top: 1px solid var(--agw-border-subtle);
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.gateway-status {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  color: var(--agw-text-muted);
}

.status-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: var(--agw-text-muted);
  transition: all 0.3s ease;
}

.gateway-status.is-online .status-dot {
  background: #10b981;
  box-shadow: 0 0 8px rgba(16, 185, 129, 0.5);
}

.status-text {
  font-family: var(--agw-font-mono, monospace);
}

.version-info {
  font-size: 11px;
  color: var(--agw-text-muted);
  font-family: var(--agw-font-mono, monospace);
}

/* ── Main Container ── */
.main-container {
  display: flex;
  flex-direction: column;
  background: var(--agw-bg-page);
}

/* ── Header ── */
.app-header {
  background: var(--agw-bg-card);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  border-bottom: 1px solid var(--agw-border-subtle);
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 28px;
  height: 64px;
  position: sticky;
  top: 0;
  z-index: 10;
}

.header-left {
  display: flex;
  align-items: baseline;
  gap: 12px;
}

.page-title {
  font-size: 20px;
  font-weight: 700;
  color: var(--agw-text-primary);
  margin: 0;
  letter-spacing: -0.5px;
}

.page-breadcrumb {
  font-size: 13px;
  color: var(--agw-text-muted);
}

.header-right {
  display: flex;
  align-items: center;
  gap: 10px;
}

.icon-btn {
  background: var(--agw-bg-hover);
  border: 1px solid var(--agw-border-default);
  color: var(--agw-text-secondary);
  width: 36px;
  height: 36px;
}

.icon-btn:hover {
  background: var(--agw-bg-elevated);
  color: var(--agw-text-primary);
  border-color: var(--agw-border-active);
}

.header-badge :deep(.el-badge__content) {
  background: #0ea5e9;
  border: none;
}

.add-btn {
  height: 36px;
  padding: 0 18px;
  border-radius: 8px;
  font-weight: 600;
  font-size: 13px;
  background: linear-gradient(135deg, #0ea5e9 0%, #06b6d4 100%);
  border: none;
  box-shadow: 0 4px 14px rgba(14, 165, 233, 0.3);
  transition: all 0.3s ease;
}

.add-btn:hover {
  background: linear-gradient(135deg, #38bdf8 0%, #22d3ee 100%);
  box-shadow: 0 6px 20px rgba(14, 165, 233, 0.4);
  transform: translateY(-1px);
}

/* ── Main Content ── */
.app-main {
  padding: 24px 28px;
  overflow-y: auto;
  flex: 1;
}

/* ── Page Transitions ── */
.fade-slide-enter-active,
.fade-slide-leave-active {
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.fade-slide-enter-from {
  opacity: 0;
  transform: translateX(16px);
}

.fade-slide-leave-to {
  opacity: 0;
  transform: translateX(-16px);
}

.copyright {
  text-align: center;
  font-size: 12px;
  color: var(--agw-text-muted);
  padding: 12px 0;
}
</style>