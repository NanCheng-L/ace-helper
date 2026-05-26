import { ref, onMounted } from 'vue'
import type { SettingsState } from '../types'
import { loadConfig, updateOptimizationSettings } from '../utils/configStorage'

// 默认进程列表
export const DEFAULT_PROCESSES = [
  { name: 'SGuardSvc64.exe', doodle: '🧸' },
  { name: 'SGuard64.exe', doodle: '🐾' },
  { name: 'ACE-Tray.exe', doodle: '💫' },
  { name: 'ace-helper.exe', doodle: '🛠️' }
]

// 默认启用的进程
const DEFAULT_ENABLED_PROCESSES = ['SGuardSvc64.exe', 'SGuard64.exe', 'ACE-Tray.exe', 'ace-helper.exe']

export function useSettings() {
  const settings = ref<SettingsState>({
    enabledProcesses: [...DEFAULT_ENABLED_PROCESSES],
    priority: 'Idle',
    affinity: [(navigator.hardwareConcurrency || 8) - 1],
    ioPriority: 'VeryLow',
    efficiencyMode: true
  })
  const isLoading = ref(false)

  // 加载设置
  const loadSettings = async () => {
    isLoading.value = true
    try {
      const config = await loadConfig()
      if (config.optimizationSettings) {
        // 处理空数组情况：如果 affinity 为空数组，使用默认值
        const affinity = (config.optimizationSettings.affinity && config.optimizationSettings.affinity.length > 0)
          ? config.optimizationSettings.affinity
          : [(navigator.hardwareConcurrency || 8) - 1]
        settings.value = {
          enabledProcesses: config.optimizationSettings.enabledProcesses || [...DEFAULT_ENABLED_PROCESSES],
          priority: config.optimizationSettings.priority || 'Idle',
          affinity,
          ioPriority: config.optimizationSettings.ioPriority || 'VeryLow',
          efficiencyMode: config.optimizationSettings.efficiencyMode ?? true
        }
      }
    } catch (e) {
      console.error('[ACE Helper] 加载设置失败:', e)
    } finally {
      isLoading.value = false
    }
  }

  // 保存设置
  const saveSettings = async (newSettings: SettingsState) => {
    try {
      await updateOptimizationSettings({
        enabledProcesses: newSettings.enabledProcesses,
        priority: newSettings.priority,
        affinity: newSettings.affinity,
        ioPriority: newSettings.ioPriority,
        efficiencyMode: newSettings.efficiencyMode
      })
      settings.value = newSettings
    } catch (e) {
      console.error('[ACE Helper] 保存设置失败:', e)
    }
  }

  // 获取启用的进程列表
  const getEnabledProcesses = () => {
    return DEFAULT_PROCESSES.filter(p => settings.value.enabledProcesses.includes(p.name))
  }

  // 组件挂载时加载设置
  onMounted(() => {
    loadSettings()
  })

  return {
    settings,
    isLoading,
    loadSettings,
    saveSettings,
    getEnabledProcesses,
    DEFAULT_PROCESSES
  }
}
