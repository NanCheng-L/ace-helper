import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { getPriorityLabel } from '../config/priority'
import type {
  BackendProcessState,
  FrontendProcessState,
  RealProcessStatus,
  ProcessData,
  ProcessDetail,
  OptimizationConfig
} from '../types'

// 后端状态转换为前端状态
export const backendToFrontendState = (state: BackendProcessState): FrontendProcessState => {
  switch (state) {
    case 1: return 'offline'
    case 2: return 'optimizing'
    case 3: return 'failed'
    case 4: return 'optimized'
    default: return 'offline'
  }
}

export function useProcess() {
  // 进程状态存储
  const processStates = ref<Record<string, ProcessData['state']>>({})
  const processDetails = ref<Record<string, ProcessDetail>>({})
  const detailOpenStates = ref<Record<string, boolean>>({})
  const checkedCount = ref(0)

  // 设置卡片状态
  const setCardState = (name: string, state: ProcessData['state'], lastUpdated?: string) => {
    processStates.value[name] = state
    if (!processDetails.value[name]) {
      processDetails.value[name] = { lastUpdated: lastUpdated || '' }
    }
    if (lastUpdated) {
      processDetails.value[name].lastUpdated = lastUpdated
    }
  }

  // 设置进程详情
  const setProcessDetail = (name: string, detail: Partial<ProcessDetail>) => {
    if (!processDetails.value[name]) {
      processDetails.value[name] = { lastUpdated: '' }
    }
    Object.assign(processDetails.value[name], detail)
  }

  // 切换卡片展开状态
  const toggleCard = (name: string) => {
    detailOpenStates.value[name] = !detailOpenStates.value[name]
  }

  // 执行优化
  const optimizeProcesses = async (config: OptimizationConfig): Promise<RealProcessStatus[]> => {
    return await invoke('optimize_processes', { config })
  }

  // 获取进程状态
  // 参数: processes 要检查的进程名列表，不传则使用默认列表
  // 参数: config 优化配置（可选），用于判断进程是否已优化
  const getProcessStatus = async (processes?: string[], config?: OptimizationConfig): Promise<RealProcessStatus[]> => {
    const defaultProcesses = ['SGuardSvc64.exe', 'SGuard64.exe', 'ACE-Tray.exe']
    return await invoke('get_process_status', { 
      processes: processes || defaultProcesses,
      config 
    })
  }

  // 获取系统信息
  // 返回系统版本和是否支持效率模式
  const getSystemInfo = async (): Promise<{
    platform: string
    version: { major: number; minor: number; build: number }
    efficiencyModeSupported: boolean
    efficiencyModeNote?: string
  }> => {
    return await invoke('get_system_info')
  }

  // 处理优化结果
  interface OptimizeResult {
    failedCount: number
    optimizedCount: number
    actuallyOptimizedCount: number
    offlineCount: number
    results: Array<{
      name: string
      state: FrontendProcessState
      logMessage: string
    }>
  }

  const processOptimizeResults = (
    realStatuses: RealProcessStatus[],
    enabledProcesses: Array<{ name: string; doodle: string }>,
    previousStates: Record<string, ProcessData['state']>
  ): OptimizeResult => {
    const result: OptimizeResult = {
      failedCount: 0,
      optimizedCount: 0,
      actuallyOptimizedCount: 0,
      offlineCount: 0,
      results: []
    }

    for (const p of enabledProcesses) {
      const status = realStatuses.find(s => s.name === p.name)
      if (!status) continue

      checkedCount.value++

      const frontendState = backendToFrontendState(status.state)
      processStates.value[p.name] = frontendState
      processDetails.value[p.name] = {
        lastUpdated: status.updatedAt.split(' ')[1] || '',
        priority: status.priority,
        affinity: status.affinity,
        coreCount: status.coreCount,
        ioPriority: status.ioPriority,
        efficiencyMode: status.efficiencyMode
      }

      let logMessage = ''

      if (status.state === 1) {
        result.offlineCount++
        logMessage = `${p.name}：当前离线`
      } else if (status.state === 4) {
        result.optimizedCount++
        const previousState = previousStates[p.name]

        if (previousState === 'optimized') {
          logMessage = `${p.name}：已优化（跳过）`
        } else {
          result.actuallyOptimizedCount++
          let statusText = '优化成功（已优化）'
          if (status.priority) statusText += `，优先级: ${getPriorityLabel(status.priority)}`
          if (status.affinity) statusText += `，CPU: ${status.affinity}`
          if (status.ioPriority) statusText += `，磁盘I/O: ${status.ioPriority}`
          if (status.efficiencyMode !== undefined) statusText += `，效率模式: ${status.efficiencyMode ? '已开启' : '未开启'}`
          logMessage = `${p.name}：${statusText}`
        }
      } else if (status.state === 3) {
        result.failedCount++
        logMessage = `${p.name}：优化失败${status.hint ? ` — ${status.hint}` : '（权限不足或被保护）'}`
      } else if (status.state === 2) {
        logMessage = `${p.name}：当前在线`
      }

      result.results.push({ name: p.name, state: frontendState, logMessage })
    }

    return result
  }

  // 处理进程信息查询结果
  const processInfoResults = (
    realStatuses: RealProcessStatus[]
  ): Array<{ name: string; info: string; isOnline: boolean }> => {
    const results: Array<{ name: string; info: string; isOnline: boolean }> = []

    for (const status of realStatuses) {
      checkedCount.value++

      processDetails.value[status.name] = {
        lastUpdated: status.updatedAt.split(' ')[1] || '',
        priority: status.priority,
        affinity: status.affinity,
        coreCount: status.coreCount,
        ioPriority: status.ioPriority,
        efficiencyMode: status.efficiencyMode
      }

      let frontendState: FrontendProcessState
      if (status.state === 1) {
        frontendState = 'offline'
      } else if (status.state === 3) {
        frontendState = 'failed'
      } else if (status.state === 4) {
        frontendState = 'optimized'
      } else if (status.state === 2) {
        // state === 2 是在线状态
        frontendState = 'online'
      } else {
        frontendState = 'offline'
      }

      processStates.value[status.name] = frontendState

      let info = `${status.name}：`
      const isOnline = frontendState === 'optimized' || frontendState === 'online'

      if (frontendState === 'optimized') info += '已优化'
      else if (frontendState === 'online') info += '在线'
      else if (frontendState === 'offline') info += '离线'
      else if (frontendState === 'failed') info += '优化失败'

      if (status.priority) info += `，优先级: ${getPriorityLabel(status.priority)}`
      if (status.affinity) info += `，CPU: ${status.affinity}`
      if (status.ioPriority) info += `，磁盘I/O: ${status.ioPriority}`
      if (status.efficiencyMode !== undefined) info += `，效率模式: ${status.efficiencyMode ? '已开启' : '未开启'}`
      if (status.pid) info += ` (PID: ${status.pid})`

      results.push({ name: status.name, info, isOnline })
    }

    return results
  }

  return {
    processStates,
    processDetails,
    detailOpenStates,
    checkedCount,
    setCardState,
    setProcessDetail,
    toggleCard,
    optimizeProcesses,
    getProcessStatus,
    getSystemInfo,
    processOptimizeResults,
    processInfoResults
  }
}
