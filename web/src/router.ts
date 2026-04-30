import { createRouter, createWebHistory } from 'vue-router'
import type { RouteRecordRaw } from 'vue-router'

const routes: RouteRecordRaw[] = [
  {
    path: '/',
    name: 'Dashboard',
    component: () => import('@/views/Dashboard.vue')
  },
  {
    path: '/plans',
    name: 'Plans',
    component: () => import('@/views/Plans.vue')
  },
  {
    path: '/plans/add',
    name: 'PlanWizard',
    component: () => import('@/views/PlanWizard.vue')
  },
  {
    path: '/fallback',
    name: 'Fallback',
    component: () => import('@/views/Fallback.vue')
  },
  {
    path: '/quota',
    name: 'Quota',
    component: () => import('@/views/Quota.vue')
  },
  {
    path: '/logs',
    name: 'Logs',
    component: () => import('@/views/Logs.vue')
  },
  {
    path: '/plugins',
    name: 'Plugins',
    component: () => import('@/views/Plugins.vue')
  },
  {
    path: '/settings',
    name: 'Settings',
    component: () => import('@/views/Settings.vue')
  }
]

const router = createRouter({
  history: createWebHistory(),
  routes
})

export default router