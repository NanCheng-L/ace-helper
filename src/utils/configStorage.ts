import { BaseDirectory, readTextFile, writeTextFile, exists, mkdir } from '@tauri-apps/plugin-fs'

const CONFIG_FILE = 'config.json'

export interface OptimizationSettings {
  enabledProcesses: string[]
  priority: string
  affinity: number[]
  ioPriority: string
}

export interface AppSettings {
  autoStart: boolean
  minimizeToTray: boolean
  autoMonitor: boolean
  enableNotifications: boolean
}

export interface AppConfig {
  optimizationSettings?: OptimizationSettings
  appSettings?: AppSettings
}

const defaultOptimizationSettings: OptimizationSettings = {
  enabledProcesses: ['SGuardSvc64.exe', 'SGuard64.exe', 'ACE-Tray.exe', 'ace-helper.exe'],
  priority: 'Idle',
  affinity: [],
  ioPriority: 'VeryLow'
}

const defaultAppSettings: AppSettings = {
  autoStart: true,
  minimizeToTray: true,
  autoMonitor: true,
  enableNotifications: true
}

const defaultConfig: AppConfig = {
  optimizationSettings: { ...defaultOptimizationSettings },
  appSettings: { ...defaultAppSettings }
}

// 确保配置目录存在
async function ensureConfigDir(): Promise<void> {
  try {
    const dirExists = await exists('', { baseDir: BaseDirectory.AppData })
    if (!dirExists) {
      await mkdir('', { baseDir: BaseDirectory.AppData, recursive: true })
    }
  } catch (e) {
    console.error('[ACE Helper] 创建配置目录失败:', e)
  }
}

// 获取默认 CPU 亲和性（最后一个核心）
function getDefaultAffinity(): number[] {
  // 在浏览器环境中使用 navigator.hardwareConcurrency
  const coreCount = typeof navigator !== 'undefined' ? navigator.hardwareConcurrency : 8
  return [coreCount - 1]
}

// 读取配置
export async function loadConfig(): Promise<AppConfig> {
  try {
    await ensureConfigDir()
    const fileExists = await exists(CONFIG_FILE, { baseDir: BaseDirectory.AppData })

    if (!fileExists) {
      console.log('[ACE Helper] 配置文件不存在，使用默认配置')
      // 创建带有正确默认 affinity 的配置
      const configWithDefaultAffinity: AppConfig = {
        optimizationSettings: {
          ...defaultOptimizationSettings,
          affinity: getDefaultAffinity()
        },
        appSettings: { ...defaultAppSettings }
      }
      await saveConfig(configWithDefaultAffinity)
      return { ...configWithDefaultAffinity }
    }

    const content = await readTextFile(CONFIG_FILE, { baseDir: BaseDirectory.AppData })
    const parsed = JSON.parse(content) as AppConfig

    // 合并默认配置，确保新字段存在
    // 处理 affinity 为空数组的情况
    const parsedAffinity = parsed.optimizationSettings?.affinity
    const affinity = (parsedAffinity && parsedAffinity.length > 0)
      ? parsedAffinity
      : getDefaultAffinity()

    return {
      optimizationSettings: {
        enabledProcesses: parsed.optimizationSettings?.enabledProcesses ?? defaultOptimizationSettings.enabledProcesses,
        priority: parsed.optimizationSettings?.priority ?? defaultOptimizationSettings.priority,
        affinity,
        ioPriority: parsed.optimizationSettings?.ioPriority ?? defaultOptimizationSettings.ioPriority
      },
      appSettings: {
        autoStart: parsed.appSettings?.autoStart ?? defaultAppSettings.autoStart,
        minimizeToTray: parsed.appSettings?.minimizeToTray ?? defaultAppSettings.minimizeToTray,
        autoMonitor: parsed.appSettings?.autoMonitor ?? defaultAppSettings.autoMonitor,
        enableNotifications: parsed.appSettings?.enableNotifications ?? defaultAppSettings.enableNotifications
      }
    }
  } catch (e) {
    console.error('[ACE Helper] 加载配置失败:', e)
    return { ...defaultConfig }
  }
}

// 保存配置
export async function saveConfig(config: AppConfig): Promise<void> {
  try {
    await ensureConfigDir()
    const content = JSON.stringify(config, null, 2)
    await writeTextFile(CONFIG_FILE, content, { baseDir: BaseDirectory.AppData })
    console.log('[ACE Helper] 配置已保存到文件')
  } catch (e) {
    console.error('[ACE Helper] 保存配置失败:', e)
    throw e
  }
}

// 更新部分配置
export async function updateConfig(partial: Partial<AppConfig>): Promise<void> {
  const current = await loadConfig()
  const updated: AppConfig = {
    optimizationSettings: {
      enabledProcesses: partial.optimizationSettings?.enabledProcesses ?? current.optimizationSettings!.enabledProcesses,
      priority: partial.optimizationSettings?.priority ?? current.optimizationSettings!.priority,
      affinity: partial.optimizationSettings?.affinity ?? current.optimizationSettings!.affinity,
      ioPriority: partial.optimizationSettings?.ioPriority ?? current.optimizationSettings!.ioPriority ?? 'VeryLow'
    },
    appSettings: {
      autoStart: partial.appSettings?.autoStart ?? current.appSettings!.autoStart,
      minimizeToTray: partial.appSettings?.minimizeToTray ?? current.appSettings!.minimizeToTray,
      autoMonitor: partial.appSettings?.autoMonitor ?? current.appSettings!.autoMonitor,
      enableNotifications: partial.appSettings?.enableNotifications ?? current.appSettings!.enableNotifications
    }
  }
  await saveConfig(updated)
}

// 仅更新优化设置
export async function updateOptimizationSettings(settings: Partial<OptimizationSettings>): Promise<void> {
  const current = await loadConfig()
  const updated: OptimizationSettings = {
    ...current.optimizationSettings!,
    ...settings
  }
  await updateConfig({ optimizationSettings: updated })
}

// 仅更新应用设置
export async function updateAppSettings(settings: Partial<AppSettings>): Promise<void> {
  const current = await loadConfig()
  const updated: AppSettings = {
    ...current.appSettings!,
    ...settings
  }
  await updateConfig({ appSettings: updated })
}
