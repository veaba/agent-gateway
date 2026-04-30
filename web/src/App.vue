<template>
  <div class="app-container">
    <el-container class="app-layout">
      <el-aside width="240px" class="sidebar">
        <div class="logo-container">
          <div class="logo-icon">
            <el-icon :size="28"><Connection /></el-icon>
          </div>
          <span class="logo-text">Agent Gateway</span>
        </div>
        <el-menu
          :default-active="$route.path"
          router
          class="sidebar-menu"
          :popper-class="'sidebar-popper'"
        >
          <el-menu-item index="/">
            <el-icon><HomeFilled /></el-icon>
            <span>仪表盘</span>
          </el-menu-item>
          <el-menu-item index="/plans">
            <el-icon><Connection /></el-icon>
            <span>我的套餐</span>
          </el-menu-item>
          <el-menu-item index="/fallback">
            <el-icon><Refresh /></el-icon>
            <span>降级策略</span>
          </el-menu-item>
          <el-menu-item index="/quota">
            <el-icon><DataLine /></el-icon>
            <span>配额使用</span>
          </el-menu-item>
          <el-menu-item index="/logs">
            <el-icon><Document /></el-icon>
            <span>请求日志</span>
          </el-menu-item>
          <el-menu-item index="/plugins">
            <el-icon><Box /></el-icon>
            <span>插件管理</span>
          </el-menu-item>
          <el-menu-item index="/settings">
            <el-icon><Setting /></el-icon>
            <span>设置</span>
          </el-menu-item>
        </el-menu>
        <div class="sidebar-footer">
          <div class="version-info">v0.1.0</div>
        </div>
      </el-aside>

      <el-container class="main-container">
        <el-header class="app-header">
          <div class="header-left">
            <h1 class="page-title">{{ pageTitle }}</h1>
          </div>
          <div class="header-right">
            <el-button type="primary" class="add-btn" @click="$router.push('/plans/add')">
              <el-icon><Plus /></el-icon>
              添加套餐
            </el-button>
          </div>
        </el-header>

        <el-main class="app-main">
          <router-view v-slot="{ Component }">
            <transition name="fade-slide" mode="out-in">
              <component :is="Component" />
            </transition>
          </router-view>
        </el-main>
      </el-container>
    </el-container>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'

const route = useRoute()

const pageTitle = computed(() => {
  const titles: Record<string, string> = {
    '/': '仪表盘',
    '/plans': '我的套餐',
    '/plans/add': '添加套餐',
    '/fallback': '降级策略',
    '/quota': '配额使用',
    '/logs': '请求日志',
    '/plugins': '插件管理',
    '/settings': '设置'
  }
  return titles[route.path] || 'Agent Gateway'
})
</script>

<style scoped>
.app-container {
  width: 100vw;
  height: 100vh;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
}

.app-layout {
  height: 100%;
  background: var(--el-bg-color);
}

/* Sidebar Styles */
.sidebar {
  background: linear-gradient(180deg, #1a1a2e 0%, #16213e 100%);
  display: flex;
  flex-direction: column;
  position: relative;
  overflow: hidden;
}

.sidebar::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: radial-gradient(ellipse at top, rgba(102, 126, 234, 0.15) 0%, transparent 60%);
  pointer-events: none;
}

.logo-container {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 24px 20px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
}

.logo-icon {
  width: 44px;
  height: 44px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  box-shadow: 0 4px 15px rgba(102, 126, 234, 0.4);
}

.logo-text {
  font-size: 18px;
  font-weight: 700;
  color: white;
  letter-spacing: -0.5px;
}

.sidebar-menu {
  flex: 1;
  background: transparent;
  border: none;
  padding: 16px 12px;
}

.sidebar-menu :deep(.el-menu-item) {
  height: 48px;
  line-height: 48px;
  margin: 4px 0;
  border-radius: 12px;
  color: rgba(255, 255, 255, 0.7);
  transition: all 0.3s ease;
}

.sidebar-menu :deep(.el-menu-item:hover) {
  background: rgba(102, 126, 234, 0.2);
  color: white;
}

.sidebar-menu :deep(.el-menu-item.is-active) {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  box-shadow: 0 4px 15px rgba(102, 126, 234, 0.4);
}

.sidebar-menu :deep(.el-menu-item .el-icon) {
  font-size: 18px;
  margin-right: 12px;
}

.sidebar-footer {
  padding: 16px 20px;
  border-top: 1px solid rgba(255, 255, 255, 0.1);
}

.version-info {
  font-size: 12px;
  color: rgba(255, 255, 255, 0.4);
  text-align: center;
}

/* Main Container */
.main-container {
  display: flex;
  flex-direction: column;
  background: var(--el-bg-color-page);
}

/* Header Styles */
.app-header {
  background: var(--el-bg-color);
  border-bottom: 1px solid var(--el-border-color-light);
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 32px;
  height: 72px;
}

.page-title {
  font-size: 24px;
  font-weight: 600;
  color: var(--el-text-color-primary);
  margin: 0;
  letter-spacing: -0.5px;
}

.add-btn {
  height: 40px;
  padding: 0 20px;
  border-radius: 10px;
  font-weight: 500;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border: none;
  box-shadow: 0 4px 15px rgba(102, 126, 234, 0.3);
  transition: all 0.3s ease;
}

.add-btn:hover {
  transform: translateY(-2px);
  box-shadow: 0 6px 20px rgba(102, 126, 234, 0.4);
}

/* Main Content */
.app-main {
  padding: 24px 32px;
  overflow-y: auto;
}

/* Page Transitions */
.fade-slide-enter-active,
.fade-slide-leave-active {
  transition: all 0.3s ease;
}

.fade-slide-enter-from {
  opacity: 0;
  transform: translateX(20px);
}

.fade-slide-leave-to {
  opacity: 0;
  transform: translateX(-20px);
}
</style>