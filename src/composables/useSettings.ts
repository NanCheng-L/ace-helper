import { ref } from 'vue'
import type { SettingsState } from '../types'

const SETTINGS_KEY = 'ace-helper-settings'

// 默认进程列表
export const DEFAULT_PROCESSES = [
  { name: 'SGuardSvc64.exe', doodle: '🧸' },
  { name: 'SGuard64.exe', doodle: '🐾' },
  { name: 'ACE-Tray.exe', doodle: '💫' }
]

export function useSettings() {
  const settings = ref<SettingsState>({
    enabledProcesses: DEFAULT_PROCESSES.map(p => p.name),
    priority: 'Idle',
    affinity: [(navigator.hardwareConcurrency || 8) - 1]
  })

  // 加载设置
  const loadSettings = () => {
    try {
      const saved = localStorage.getItem(SETTINGS_KEY)
      if (saved) {
        const parsed = JSON.parse(saved)
        settings.value = {
          enabledProcesses: parsed.enabledProcesses || DEFAULT_PROCESSES.map(p => p.name),
          priority: parsed.priority || 'Idle',
          affinity: parsed.affinity || []
        }
      }
    } catch (e) {
      console.error('[ACE Helper] 加载设置失败:', e)
    }
  }

  // 保存设置
  const saveSettings = (newSettings: SettingsState) => {
    try {
      localStorage.setItem(SETTINGS_KEY, JSON.stringify(newSettings))
      settings.value = newSettings
    } catch (e) {
      console.error('[ACE Helper] 保存设置失败:', e)
    }
  }

  // 获取启用的进程列表
  const getEnabledProcesses = () => {
    return DEFAULT_PROCESSES.filter(p => settings.value.enabledProcesses.includes(p.name))
  }

  return {
    settings,
    loadSettings,
    saveSettings,
    getEnabledProcesses,
    DEFAULT_PROCESSES
  }
}
