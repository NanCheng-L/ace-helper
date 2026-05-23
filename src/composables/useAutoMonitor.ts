import { ref, onUnmounted } from 'vue'
import { loadConfig, updateAppSettings } from '../utils/configStorage'

export function useAutoMonitor() {
  // autoStartMonitor: 持久化设置，控制"启动时是否自动开启监听"
  const autoStartMonitor = ref(true)
  // isAutoMonitoring: 当前会话的临时状态
  const isAutoMonitoring = ref(false)
  let autoMonitorTimer: ReturnType<typeof setTimeout> | null = null

  // 加载自动监听设置（从配置文件读取 autoStartMonitor）
  const loadAutoMonitorSetting = async () => {
    try {
      const config = await loadConfig()
      if (typeof config.appSettings?.autoMonitor === 'boolean') {
        autoStartMonitor.value = config.appSettings.autoMonitor
      }
    } catch (e) {
      // 忽略
    }
  }

  // 保存自动监听设置（保存到配置文件的 autoStartMonitor）
  const saveAutoMonitorSetting = async (val: boolean) => {
    try {
      await updateAppSettings({ autoMonitor: val })
      autoStartMonitor.value = val
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

  // 切换当前会话的自动监听状态（不影响持久化设置）
  const toggleAutoMonitor = (
    onStart: () => void,
    onStop: () => void
  ) => {
    if (isAutoMonitoring.value) {
      stopAutoMonitor()
      isAutoMonitoring.value = false
      onStop()
    } else {
      isAutoMonitoring.value = true
      onStart()
    }
  }

  // 根据持久化设置启动自动监听（仅在应用启动时调用）
  const startAutoMonitorIfEnabled = (onStart: () => void) => {
    if (autoStartMonitor.value) {
      isAutoMonitoring.value = true
      onStart()
    }
  }

  // 组件卸载时清理定时器
  onUnmounted(() => {
    stopAutoMonitor()
  })

  return {
    autoStartMonitor,
    isAutoMonitoring,
    loadAutoMonitorSetting,
    saveAutoMonitorSetting,
    startAutoMonitor,
    stopAutoMonitor,
    toggleAutoMonitor,
    startAutoMonitorIfEnabled
  }
}
