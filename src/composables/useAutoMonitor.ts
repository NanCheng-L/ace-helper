import { ref, onUnmounted } from 'vue'

const APP_SETTINGS_KEY = 'ace-app-settings'

export function useAutoMonitor() {
  const isAutoMonitoring = ref(true)
  let autoMonitorTimer: ReturnType<typeof setTimeout> | null = null

  // 加载自动监听设置
  const loadAutoMonitorSetting = () => {
    try {
      const saved = localStorage.getItem(APP_SETTINGS_KEY)
      if (saved) {
        const parsed = JSON.parse(saved)
        if (typeof parsed.autoMonitor === 'boolean') {
          isAutoMonitoring.value = parsed.autoMonitor
        }
      }
    } catch (e) {
      // 忽略
    }
  }

  // 保存自动监听设置
  const saveAutoMonitorSetting = (val: boolean) => {
    try {
      const saved = localStorage.getItem(APP_SETTINGS_KEY)
      const parsed = saved ? JSON.parse(saved) : {}
      parsed.autoMonitor = val
      localStorage.setItem(APP_SETTINGS_KEY, JSON.stringify(parsed))
    } catch (e) {
      // 忽略
    }
  }

  // 启动自动监听定时器
  const startAutoMonitor = (callback: () => void, interval = 3000) => {
    if (autoMonitorTimer) {
      clearTimeout(autoMonitorTimer)
    }
    autoMonitorTimer = setTimeout(callback, interval)
  }

  // 停止自动监听定时器
  const stopAutoMonitor = () => {
    if (autoMonitorTimer) {
      clearTimeout(autoMonitorTimer)
      autoMonitorTimer = null
    }
  }

  // 切换自动监听模式
  const toggleAutoMonitor = (
    onStart: () => void,
    onStop: () => void
  ) => {
    if (isAutoMonitoring.value) {
      stopAutoMonitor()
      isAutoMonitoring.value = false
      saveAutoMonitorSetting(false)
      onStop()
    } else {
      isAutoMonitoring.value = true
      saveAutoMonitorSetting(true)
      onStart()
    }
  }

  // 组件卸载时清理定时器
  onUnmounted(() => {
    stopAutoMonitor()
  })

  return {
    isAutoMonitoring,
    loadAutoMonitorSetting,
    saveAutoMonitorSetting,
    startAutoMonitor,
    stopAutoMonitor,
    toggleAutoMonitor
  }
}
