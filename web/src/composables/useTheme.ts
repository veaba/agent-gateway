import { ref, watch, onMounted } from 'vue'

export type ThemeMode = 'dark' | 'light' | 'auto'

const STORAGE_KEY = 'agw-settings'

/**
 * Theme composable for managing dark/light/auto mode
 *
 * The theme is applied by toggling the 'dark' class on the HTML element.
 * Element Plus dark theme CSS vars are already imported in main.ts.
 */
export function useTheme() {
  const theme = ref<ThemeMode>('dark')
  const isInitialized = ref(false)

  /**
   * Apply theme to DOM by toggling 'dark' class on html element
   */
  const applyTheme = (mode: ThemeMode) => {
    const html = document.documentElement

    if (mode === 'auto') {
      // Follow system preference
      const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches
      html.classList.toggle('dark', prefersDark)
    } else {
      html.classList.toggle('dark', mode === 'dark')
    }
  }

  /**
   * Load theme from localStorage and apply it
   */
  const loadTheme = () => {
    try {
      const saved = localStorage.getItem(STORAGE_KEY)
      if (saved) {
        const parsed = JSON.parse(saved)
        if (parsed.theme) {
          theme.value = parsed.theme as ThemeMode
        }
      }
    } catch {
      // Ignore errors
    }

    applyTheme(theme.value)
    isInitialized.value = true
  }

  /**
   * Save theme to localStorage and apply it
   */
  const setTheme = (mode: ThemeMode) => {
    theme.value = mode
    applyTheme(mode)

    // Save to localStorage (merge with existing settings)
    try {
      const saved = localStorage.getItem(STORAGE_KEY)
      const existing = saved ? JSON.parse(saved) : {}
      existing.theme = mode
      localStorage.setItem(STORAGE_KEY, JSON.stringify(existing))
    } catch {
      // Ignore errors
    }
  }

  // Watch for system preference changes when in auto mode
  const setupAutoListener = () => {
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')

    mediaQuery.addEventListener('change', (e) => {
      if (theme.value === 'auto') {
        document.documentElement.classList.toggle('dark', e.matches)
      }
    })
  }

  onMounted(() => {
    loadTheme()
    setupAutoListener()
  })

  // Also watch theme ref changes (for reactive updates from Settings.vue)
  watch(theme, (newTheme) => {
    if (isInitialized.value) {
      applyTheme(newTheme)
    }
  })

  return {
    theme,
    setTheme,
    loadTheme,
    applyTheme
  }
}