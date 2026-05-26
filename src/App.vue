<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { listen } from '@tauri-apps/api/event'
import { isPermissionGranted, requestPermission } from '@tauri-apps/plugin-notification'
import Sidebar from './components/Sidebar.vue'
import HomePage from './views/home/HomePage.vue'
import OptimizationPage from './views/optimization/OptimizationPage.vue'
import SettingsPage from './views/settings/SettingsPage.vue'
import AboutPage from './views/about/AboutPage.vue'
import { useLogs } from './composables/useLogs'
import { useSettings } from './composables/useSettings'
import { useAutoMonitor } from './composables/useAutoMonitor'
import { useProcess } from './composables/useProcess'
import { loadConfig } from './utils/configStorage'

// 当前激活的标签页
const activeTab = ref('home')

// 底部提示文字
const metaText = ref('点"一键优化"让 ACE 进程乖乖听话吧 ✨')

// 使用 composables
const { logs, addLog, initLog } = useLogs()
const { settings, loadSettings, getEnabledProcesses } = useSettings()
const { isAutoMonitoring, loadAutoMonitorSetting, startAutoMonitor, toggleAutoMonitor, startAutoMonitorIfEnabled } = useAutoMonitor()
const {
  processStates,
  processDetails,
  detailOpenStates,
  checkedCount,
  toggleCard,
  optimizeProcesses,
  getProcessStatus,
  getSystemInfo,
  processOptimizeResults,
  processInfoResults
} = useProcess()

// 根据设置过滤启用的进程
const enabledProcesses = computed(() => getEnabledProcesses())

// 设置提示文字
const setMeta = (text: string) => {
  metaText.value = text
}

// 延迟函数
const sleep = (ms: number) => new Promise(r => setTimeout(r, ms))

// 发送系统通知
const notifyOptimized = async (newlyOptimized: string[]) => {
  if (newlyOptimized.length === 0) return
  try {
    const config = await loadConfig()
    if (!config.appSettings?.enableNotifications) return

    let grantResult = await isPermissionGranted()
    if (grantResult === null || grantResult === false) {
      const permission = await requestPermission()
      grantResult = permission === 'granted'
    }
    if (!grantResult) return

    const names = newlyOptimized.join('、')
    invoke('send_notification', {
      title: 'ACE 小助手',
      body: `已优化：${names}`
    }).catch(() => {})
  } catch {}
}

// 运行优化流程
const runOptimize = async (isAuto = false) => {
  const currentProcesses = enabledProcesses.value
  const startTime = Date.now()

  if (!isAuto) {
    setMeta('正在施展优化魔法，请稍等一下下… 🪄')
  }

  addLog(`开始优化：${currentProcesses.map(p => p.name).join(' / ')}`)

  try {
    const realStatuses = await optimizeProcesses({
      processes: settings.value.enabledProcesses,
      priority: settings.value.priority,
      affinity: settings.value.affinity,
      ioPriority: settings.value.ioPriority,
      efficiencyMode: settings.value.efficiencyMode
    })

    // 保存之前的状态用于判断
    const previousStates = { ...processStates.value }

    const result = processOptimizeResults(realStatuses, currentProcesses, previousStates)

    // 输出日志
    result.results.forEach(r => addLog(r.logMessage))

    // 收集新优化的进程名并发送通知
    const newlyOptimized = result.results
      .filter(r => r.state === 'optimized' && previousStates[r.name] !== 'optimized')
      .map(r => r.name)
    notifyOptimized(newlyOptimized)

    // 确保提示至少显示1秒（仅手动触发时）
    if (!isAuto) {
      const elapsed = Date.now() - startTime
      if (elapsed < 1000) {
        await sleep(1000 - elapsed)
      }
    }

    // 设置底部提示
    if (result.failedCount > 0) {
      addLog(`⚠️ 有 ${result.failedCount} 个进程因权限不足无法优化`)
      setMeta(`有 ${result.failedCount} 个进程需要管理员权限才能优化`)
    } else if (result.optimizedCount > 0) {
      setMeta('全部进程已乖乖排好队，一切正常喵~ ✦')
      if (result.actuallyOptimizedCount > 0) {
        addLog(`✅ 优化完成：${result.actuallyOptimizedCount} 个进程已优化`)
      }
    } else if (result.offlineCount === currentProcesses.length) {
      addLog('没有发现目标进程在运行哦 ( ˘︹˘ )')
      setMeta('没有发现 ACE 进程在运行，它们好像躲猫猫去了 👀')
    }

    // 自动监听模式：设置下一次检测
    if (isAutoMonitoring.value) {
      startAutoMonitor(() => runOptimize(true), 3000)
    }

  } catch (error) {
    console.error('[ACE Helper] 获取进程状态失败:', error)
    const errMsg = error instanceof Error ? error.message : String(error)
    addLog('获取进程状态失败：' + errMsg)
    setMeta('哎呀出错了，获取进程状态遇到了点小麻烦…看看日志？📋')

    // 出错后继续监听
    if (isAutoMonitoring.value) {
      startAutoMonitor(() => runOptimize(true), 5000)
    }
  }
}

// 获取进程信息
const getProcessInfo = async () => {
  try {
    // 使用配置中启用的进程列表
    const config = await loadConfig()
    const processesToCheck = config.optimizationSettings?.enabledProcesses || ['SGuardSvc64.exe', 'SGuard64.exe', 'ACE-Tray.exe']
    
    // 构建优化配置用于判断进程是否已优化
    const optimizationConfig = {
      processes: processesToCheck,
      priority: config.optimizationSettings?.priority || 'Idle',
      affinity: config.optimizationSettings?.affinity || [],
      ioPriority: config.optimizationSettings?.ioPriority || 'VeryLow',
      efficiencyMode: config.optimizationSettings?.efficiencyMode ?? true
    }
    
    const realStatuses = await getProcessStatus(processesToCheck, optimizationConfig)
    addLog('===== 获取进程信息 =====')

    const results = processInfoResults(realStatuses)

    const onlineCount = results.filter(r => r.isOnline).length
    results.forEach(r => addLog(r.info))

    if (onlineCount === 0) {
      addLog('没有发现目标进程在运行')
    }
    addLog('===== 获取完成 =====')

  } catch (error) {
    console.error('[ACE Helper] 获取进程信息失败:', error)
    const errMsg = error instanceof Error ? error.message : String(error)
    addLog('获取进程信息失败：' + errMsg)
  }
}

// 切换自动监听
const handleToggleMonitor = () => {
  toggleAutoMonitor(
    () => {
      addLog('已开启自动监听，每 3 秒检测一次')
      runOptimize(true)
    },
    () => addLog('已关闭自动监听')
  )
}

// 同步关闭最小化设置到后端
const syncMinimizeToTray = async () => {
  try {
    const config = await loadConfig()
    invoke('set_minimize_to_tray', { value: config.appSettings?.minimizeToTray ?? true }).catch(() => {})
  } catch {}
}

// 处理菜单选择
const handleSelect = (id: string) => {
  activeTab.value = id
  if (id === 'settings') {
    loadSettings().then(() => syncMinimizeToTray())
  } else if (id === 'home') {
    loadSettings().then(() => syncMinimizeToTray())
    loadAutoMonitorSetting()
    if (isAutoMonitoring.value) {
      runOptimize(true)
    }
  }
}

// 跳转到设置页面
const goToSettings = () => {
  handleSelect('settings')
}

// 跳转到关于页面
const goToAbout = () => {
  handleSelect('about')
}

// 初始化
onMounted(async () => {
  listen('tray-open-settings', () => {
    handleSelect('settings')
  })

  listen('tray-show-main', () => {
    handleSelect('home')
  })

  // 获取并打印系统信息
  try {
    const sysInfo = await getSystemInfo()
    console.log('[ACE Helper] 系统信息:', sysInfo)
    console.log(`[ACE Helper] 系统: ${sysInfo.version.displayName} (Build ${sysInfo.version.build})`)
    console.log(`[ACE Helper] 效率模式支持: ${sysInfo.efficiencyModeSupported ? '是' : '否'}`)
    if (sysInfo.efficiencyModeNote) {
      console.log(`[ACE Helper] 效率模式说明: ${sysInfo.efficiencyModeNote}`)
    }
  } catch (e) {
    console.error('[ACE Helper] 获取系统信息失败:', e)
  }

  loadSettings().then(async () => {
    await syncMinimizeToTray()
    await getCurrentWindow().show()
  })
  loadAutoMonitorSetting()
  initLog('UI 原型已加载：点击「一键优化」开始优化。')

  setTimeout(() => {
    // 根据持久化设置决定是否启动自动监听
    startAutoMonitorIfEnabled(() => {
      addLog('已开启自动监听，每 3 秒检测一次')
      runOptimize(true)
    })
  }, 1000)
})
</script>

<template>
  <div class="app">
    <Sidebar :active-tab="activeTab" @select="handleSelect" @go-to-about="goToAbout" />

    <main class="main">
      <!-- 首页 -->
      <HomePage
        v-if="activeTab === 'home'"
        :meta-text="metaText"
        :is-auto-monitoring="isAutoMonitoring"
        :logs="logs"
        :checked-count="checkedCount"
        :enabled-processes="enabledProcesses"
        :process-states="processStates"
        :process-details="processDetails"
        :detail-open-states="detailOpenStates"
        @optimize="runOptimize()"
        @get-info="getProcessInfo"
        @toggle-monitor="handleToggleMonitor"
        @toggle-card="toggleCard"
        @go-to-settings="goToSettings"
      />

      <!-- 进程优化页面 -->
      <OptimizationPage
        v-else-if="activeTab === 'settings'"
        @saved="loadSettings"
      />

      <!-- 通用设置页面 -->
      <SettingsPage
        v-else-if="activeTab === 'appsettings'"
      />

      <!-- 关于帮助页面 -->
      <AboutPage
        v-else-if="activeTab === 'about'"
      />
    </main>
  </div>
</template>

<style>
.app {
  display: flex;
  height: 100vh;
  overflow: hidden;
  font-family: 'Nunito', 'Microsoft YaHei', sans-serif;
  background: linear-gradient(135deg, #fdf2f8 0%, #f3e8ff 50%, #e0e7ff 100%);
  color: #4b5563;
}

.main {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow-y: auto;
  position: relative;
}
</style>
