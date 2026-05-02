import { createApp } from 'vue'
import ElementPlus from 'element-plus'
import 'element-plus/dist/index.css'
import 'element-plus/theme-chalk/dark/css-vars.css'
import '@/styles/theme.css'
import * as ElementPlusIconsVue from '@element-plus/icons-vue'
import App from './App.vue'
import router from './router'

// Initialize theme before app mounts to prevent flash
const initTheme = () => {
  const STORAGE_KEY = 'agw-settings'
  try {
    const saved = localStorage.getItem(STORAGE_KEY)
    if (saved) {
      const parsed = JSON.parse(saved)
      const theme = parsed.theme || 'dark'
      const html = document.documentElement

      if (theme === 'auto') {
        const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches
        html.classList.toggle('dark', prefersDark)
      } else {
        html.classList.toggle('dark', theme === 'dark')
      }
    } else {
      // Default to dark theme
      document.documentElement.classList.add('dark')
    }
  } catch {
    // Default to dark on error
    document.documentElement.classList.add('dark')
  }
}

// Apply theme immediately
initTheme()

const app = createApp(App)

// 注册所有图标
for (const [key, component] of Object.entries(ElementPlusIconsVue)) {
  app.component(key, component)
}

app.use(ElementPlus)
app.use(router)
app.mount('#app')