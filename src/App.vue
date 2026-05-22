<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { getPriorityLabel } from './config/priority'
import Sidebar from './components/Sidebar.vue'
import TopBar from './components/TopBar.vue'
import ProcessCard, { type ProcessData } from './components/ProcessCard.vue'
import LogSection from './components/LogSection.vue'
import OptimizationSettings from './components/OptimizationSettings.vue'
import AppSettings from './components/AppSettings.vue'
import AboutPage from './components/AboutPage.vue'

const activeTab = ref('home')
const metaText = ref('正在等你来点一下一键优化…')
const logs = ref<string[]>([])
const checkedCount = ref(0)

// 设置状态
const settings = ref({
  enabledProcesses: ['SGuardSvc64.exe', 'SGuard64.exe', 'ACE-Tray.exe'],
  priority: 'Idle',
  affinity: [(navigator.hardwareConcurrency || 8) - 1] as number[]
})

const PROC_LIST: { name: string; doodle: string }[] = [
  { name: 'SGuardSvc64.exe', doodle: '🧸' },
  { name: 'SGuard64.exe', doodle: '🐾' },
  { name: 'ACE-Tray.exe', doodle: '💫' }
]

// 根据设置过滤启用的进程
const enabledProcesses = computed(() => {
  return PROC_LIST.filter(p => settings.value.enabledProcesses.includes(p.name))
})

// 进程状态存储（用于保留每个进程的状态）
const processStates = ref<Record<string, ProcessData['state']>>({})
const processDetails = ref<Record<string, { lastUpdated: string; priority?: string; affinity?: string; coreCount?: number }>>({})
const detailOpenStates = ref<Record<string, boolean>>({})

// 根据设置和状态计算当前显示的进程
const processes = computed<ProcessData[]>(() => {
  return enabledProcesses.value.map(p => {
    const name = p.name
    return {
      name,
      doodle: p.doodle,
      state: processStates.value[name] || 'offline',
      lastUpdated: processDetails.value[name]?.lastUpdated || '',
      foundThisRun: processStates.value[name] === 'optimized',
      detailOpen: detailOpenStates.value[name] || false,
      priority: processDetails.value[name]?.priority,
      affinity: processDetails.value[name]?.affinity,
      coreCount: processDetails.value[name]?.coreCount
    }
  })
})

// 加载设置
const loadSettings = () => {
  try {
    const saved = localStorage.getItem('ace-helper-settings')
    if (saved) {
      const parsed = JSON.parse(saved)
      settings.value = {
        enabledProcesses: parsed.enabledProcesses || [...PROC_LIST.map(p => p.name)],
        priority: parsed.priority || 'Idle',
        affinity: parsed.affinity || []
      }
    }
  } catch (e) {
    console.error('[ACE Helper] 加载设置失败:', e)
  }
}

const nowStr = () => {
  const d = new Date()
  const pad = (n: number) => String(n).padStart(2, '0')
  return `${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`
}

// 添加日志函数，使用 push 把最新的添加到数组末尾，这样最新的在最下面（像终端一样）
const addLog = (line: string) => {
  logs.value.push(`[${nowStr()}] ${line}`)
  if (logs.value.length > 30) {
    // 超过数量限制时，移除最旧的（数组开头）
    logs.value.shift()
  }
}

const setMeta = (text: string) => {
  metaText.value = text
}

const setCardState = (name: string, state: ProcessData['state']) => {
  processStates.value[name] = state
  if (!processDetails.value[name]) {
    processDetails.value[name] = { lastUpdated: nowStr() }
  }
  processDetails.value[name].lastUpdated = nowStr()
}

// 后端返回的进程状态枚举
// 1: 离线, 2: 在线, 3: 优化失败, 4: 已优化
type BackendProcessState = 1 | 2 | 3 | 4

// 前端使用的进程状态类型
type FrontendProcessState = 'offline' | 'scanning' | 'optimizing' | 'optimized' | 'failed'

// 定义从 Rust 后端接收的数据结构
interface RealProcessStatus {
  name: string
  state: BackendProcessState
  updatedAt: string
  hint: string
  pid?: number
  priority?: string
  priorityKey?: string
  affinity?: string
  coreCount?: number
}

// 后端状态转换为前端状态
// 状态枚举: 1=离线, 2=在线, 3=优化失败, 4=已优化
const backendToFrontendState = (state: BackendProcessState): FrontendProcessState => {
  switch (state) {
    case 1: return 'offline'
    case 2: return 'optimizing'  // 在线状态表示进程已发现，准备优化或正在优化
    case 3: return 'failed'
    case 4: return 'optimized'
    default: return 'offline'
  }
}

// 自动监听定时器 ID
let autoMonitorTimer: ReturnType<typeof setTimeout> | null = null
const isAutoMonitoring = ref(true)

const loadAutoMonitorSetting = () => {
  try {
    const saved = localStorage.getItem('ace-app-settings')
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

const saveAutoMonitorSetting = (val: boolean) => {
  try {
    const saved = localStorage.getItem('ace-app-settings')
    const parsed = saved ? JSON.parse(saved) : {}
    parsed.autoMonitor = val
    localStorage.setItem('ace-app-settings', JSON.stringify(parsed))
  } catch (e) {
    // 忽略
  }
}  // 默认开启自动监听

// 切换自动监听模式
const toggleAutoMonitor = () => {
  if (isAutoMonitoring.value) {
    if (autoMonitorTimer) {
      clearTimeout(autoMonitorTimer)
      autoMonitorTimer = null
    }
    isAutoMonitoring.value = false
    saveAutoMonitorSetting(false)
    addLog('已关闭自动监听')
  } else {
    isAutoMonitoring.value = true
    saveAutoMonitorSetting(true)
    addLog('已开启自动监听，每 3 秒检测一次')
    runOptimize(true)
  }
}

const sleep = (ms: number) => new Promise(r => setTimeout(r, ms))

// 运行优化流程的函数
// 参数 isAuto: 是否为自动监听触发的调用
const runOptimize = async (isAuto = false) => {
  const currentProcesses = processes.value
  const startTime = Date.now()
  
  // 只有非自动检测时才显示"正在优化进程…"提示
  if (!isAuto) {
    setMeta('正在优化进程…')
  }
  
  addLog(`开始优化：${currentProcesses.map(p => p.name).join(' / ')}`)
  checkedCount.value = currentProcesses.length

  try {
    // 关键步骤：调用 Tauri 命令，访问 Rust 后端执行优化
    // 使用用户在设置页面配置的优先级和 CPU 亲和性
    const realStatuses: RealProcessStatus[] = await invoke('optimize_processes', {
      config: {
        processes: settings.value.enabledProcesses,
        priority: settings.value.priority,
        affinity: settings.value.affinity
      }
    })

    console.log('[ACE Helper] 真实进程状态:', JSON.stringify(realStatuses, null, 2))

    for (const status of realStatuses) {
      console.log(`[ACE Helper] ${status.name}: ${status.state} (PID: ${status.pid || 'N/A'}) - ${status.hint}`)
    }

    // 处理结果
    let failedCount = 0
    let optimizedCount = 0
    let actuallyOptimizedCount = 0
    let offlineCount = 0

    for (const p of currentProcesses) {
      const status = realStatuses.find(s => s.name === p.name)
      if (!status) continue

      const frontendState = backendToFrontendState(status.state)
      processStates.value[p.name] = frontendState
      processDetails.value[p.name] = {
        lastUpdated: status.updatedAt.split(' ')[1] || nowStr(),
        priority: status.priority,
        affinity: status.affinity,
        coreCount: status.coreCount
      }

      // 添加日志
      if (status.state === 1) {
        // 离线
        offlineCount++
        addLog(`${p.name}：当前离线`)
      } else if (status.state === 4) {
        // 已优化
        optimizedCount++
        
        // 检查之前的状态，如果之前已经是已优化，说明跳过了优化
        const previousState = p.state
        let statusText = ''
        
        if (previousState === 'optimized') {
          statusText = '已优化（跳过）'
        } else {
          // 实际执行了优化
          actuallyOptimizedCount++
          statusText = '优化成功（已优化）'
        }
        
        if (status.priority) statusText += `，优先级: ${getPriorityLabel(status.priority)}`
        if (status.affinity) statusText += `，CPU: ${status.affinity}`
        addLog(`${p.name}：${statusText}`)
      } else if (status.state === 3) {
        // 优化失败
        failedCount++
        addLog(`${p.name}：优化失败（请看日志）`)
      } else if (status.state === 2) {
        // 在线（这个状态在优化逻辑中应该不会返回，因为在线需要优化成已优化或失败）
        addLog(`${p.name}：当前在线`)
      }
    }

    // 确保"正在优化进程..."提示至少显示1秒（仅手动触发时）
    if (!isAuto) {
      const elapsed = Date.now() - startTime
      if (elapsed < 1000) {
        await sleep(1000 - elapsed)
      }
    }

    // 底部提示文字
    if (failedCount > 0) {
      setMeta(`有 ${failedCount} 项优化失败：点卡片看提示，或者再刷新一次试试。`)
    } else if (optimizedCount > 0) {
      setMeta('当前一切正常，像贴纸一样乖乖的 ✦')
      // 只有实际执行了优化才显示优化完成日志
      if (actuallyOptimizedCount > 0) {
        addLog(`✅ 优化完成：${actuallyOptimizedCount} 个进程已优化`)
      }
    } else if (offlineCount === currentProcesses.length) {
      addLog('没有发现目标进程在运行哦 ( ˘︹˘ )')
      setMeta('当前离线：没有发现目标进程，等待进程出现…')
    }

    // 自动监听模式：本次检测完成后，设置下一个定时器
    if (isAutoMonitoring.value) {
      console.log(`[ACE Helper] 自动监听：将在 3 秒后进行下一次检测...`)
      if (autoMonitorTimer) {
        clearTimeout(autoMonitorTimer)
      }
      autoMonitorTimer = setTimeout(() => {
        runOptimize(true)
      }, 3000)
    }

  } catch (error) {
    console.error('[ACE Helper] 获取进程状态失败:', error)
    const errMsg = error instanceof Error ? error.message : String(error)
    addLog('获取进程状态失败：' + errMsg)
    setMeta('获取进程状态失败，请检查日志')

    // 自动监听模式下即使出错也继续监听
    if (isAutoMonitoring.value) {
      console.log(`[ACE Helper] 自动监听：出错后将在 5 秒后重试...`)
      if (autoMonitorTimer) {
        clearTimeout(autoMonitorTimer)
      }
      autoMonitorTimer = setTimeout(() => {
        runOptimize(true)
      }, 5000)
    }
  }
}

const toggleCard = (index: number) => {
  const processName = processes.value[index].name
  detailOpenStates.value[processName] = !detailOpenStates.value[processName]
}

// 检查是否有启用任何进程
const hasEnabledProcesses = computed(() => {
  return settings.value.enabledProcesses.length > 0
})

const handleSelect = (id: string) => {
  activeTab.value = id
  if (id === 'settings') {
    loadSettings()
  } else if (id === 'home') {
    loadSettings()
    loadAutoMonitorSetting()
    if (isAutoMonitoring.value && !autoMonitorTimer) {
      runOptimize(true)
    }
  }
}

// 获取进程信息用于调试（同时更新首页显示）
const getProcessInfo = async () => {
  try {
    const realStatuses = await invoke<RealProcessStatus[]>('get_process_status')
    console.log('[ACE Helper] 当前进程状态:', JSON.stringify(realStatuses, null, 2))
    
    addLog('===== 获取进程信息 =====')
    
    let onlineCount = 0
    for (const status of realStatuses) {
      // 更新进程详情
      processDetails.value[status.name] = {
        lastUpdated: status.updatedAt.split(' ')[1] || nowStr(),
        priority: status.priority,
        affinity: status.affinity,
        coreCount: status.coreCount
      }
      
      // 根据进程信息判断状态
      // 如果有优先级信息且状态是在线，说明进程已经被检测过，可以认为是已优化状态
      let frontendState: FrontendProcessState
      if (status.state === 1) {
        frontendState = 'offline'
      } else if (status.state === 3) {
        frontendState = 'failed'
      } else if (status.state === 4) {
        frontendState = 'optimized'
      } else if (status.state === 2) {
        // 在线状态：如果有优先级信息，显示已优化；否则显示在线（待优化）
        if (status.priority) {
          frontendState = 'optimized'
        } else {
          frontendState = 'optimizing'
        }
      } else {
        frontendState = 'offline'
      }
      
      processStates.value[status.name] = frontendState
      
      if (frontendState === 'optimized' || frontendState === 'optimizing') {
        // 在线或已优化
        onlineCount++
        let info = `${status.name}：`
        if (frontendState === 'optimized') info += '已优化'
        else info += '在线（待优化）'
        
        if (status.priority) info += `，优先级: ${getPriorityLabel(status.priority)}`
        if (status.affinity) info += `，CPU: ${status.affinity}`
        if (status.pid) info += ` (PID: ${status.pid})`
        
        addLog(info)
        console.log(`[ACE Helper] ${status.name}: state=${status.state} (PID: ${status.pid || 'N/A'})`)
      } else if (frontendState === 'offline') {
        // 离线
        addLog(`${status.name}：离线`)
      } else if (frontendState === 'failed') {
        // 优化失败
        addLog(`${status.name}：优化失败`)
      }
    }
    
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

// 初始日志
addLog('UI 原型已加载：点击「一键优化」开始优化。')

// 页面加载时读取设置并自动开始监听
onMounted(() => {
  loadSettings()
  loadAutoMonitorSetting()
  setTimeout(() => {
    if (isAutoMonitoring.value) {
      addLog('已开启自动监听，每 3 秒检测一次')
      runOptimize(true)
    }
  }, 1000)
})
</script>

<template>
  <div class="app">
    <Sidebar :active-tab="activeTab" @select="handleSelect" />

    <main class="main">
      <!-- 首页 -->
      <template v-if="activeTab === 'home'">
        <div class="floaties" aria-hidden="true">
          <div class="floaty b">A/B!!</div>
          <div class="floaty a">像贴纸一样乖乖的</div>
          <div class="floaty c">小爪机彩蛋</div>
        </div>
        <TopBar :meta-text="metaText" :is-auto-monitoring="isAutoMonitoring" @optimize="runOptimize()" @getinfo="getProcessInfo()" @toggle-monitor="toggleAutoMonitor()" />

        <section class="grid" aria-label="进程卡片">
          <!-- 没有启用任何进程时的提示 -->
          <div v-if="!hasEnabledProcesses" class="empty-tip" @click="handleSelect('settings')">
            <div class="empty-tip-icon">⚙️</div>
            <div class="empty-tip-text">还没有勾选要优化的进程哦</div>
            <div class="empty-tip-sub">点击这里 → 去设置里勾选进程</div>
          </div>
          <ProcessCard
            v-for="(process, index) in processes"
            :key="process.name"
            :process="process"
            :index="index"
            @toggle="toggleCard(index)"
          />
        </section>

        <LogSection :logs="logs" :checked-count="checkedCount" />
      </template>

      <!-- 优化设置页面 -->
      <template v-else-if="activeTab === 'settings'">
        <OptimizationSettings @saved="loadSettings" />
      </template>

      <!-- 软件设置页面 -->
      <template v-else-if="activeTab === 'appsettings'">
        <AppSettings />
      </template>

      <!-- 关于帮助页面 -->
      <template v-else-if="activeTab === 'about'">
        <AboutPage />
      </template>
    </main>
  </div>
</template>
