// 后端返回的进程状态枚举
// 1: 离线, 2: 在线, 3: 优化失败, 4: 已优化
export type BackendProcessState = 1 | 2 | 3 | 4

// 前端使用的进程状态类型
export type FrontendProcessState = 'offline' | 'online' | 'scanning' | 'optimizing' | 'optimized' | 'failed'

// 定义从 Rust 后端接收的数据结构
export interface RealProcessStatus {
  name: string
  state: BackendProcessState
  updatedAt: string
  hint: string
  pid?: number
  priority?: string
  priorityKey?: string
  affinity?: string
  coreCount?: number
  ioPriority?: string
  ioPriorityKey?: string
  efficiencyMode?: boolean
}

// 进程数据（用于 ProcessCard 组件）
export interface ProcessData {
  name: string
  doodle: string
  state: FrontendProcessState
  lastUpdated: string
  foundThisRun: boolean
  detailOpen: boolean
  priority?: string
  affinity?: string
  coreCount?: number
  ioPriority?: string
  efficiencyMode?: boolean
}

// 进程列表项
export interface ProcessListItem {
  name: string
  doodle: string
}

// 优化配置
export interface OptimizationConfig {
  processes: string[]
  priority: string
  affinity: number[]
  ioPriority: string
  efficiencyMode: boolean
}

// 设置状态
export interface SettingsState {
  enabledProcesses: string[]
  priority: string
  affinity: number[]
  ioPriority: string
  efficiencyMode: boolean
}

// 进程详情
export interface ProcessDetail {
  lastUpdated: string
  priority?: string
  affinity?: string
  coreCount?: number
  ioPriority?: string
  efficiencyMode?: boolean
}
