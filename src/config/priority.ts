export interface PriorityOption {
  value: string
  label: string
  desc: string
}

export const PRIORITY_OPTIONS: PriorityOption[] = [
  { value: 'Normal', label: '正常', desc: '标准优先级' },
  { value: 'BelowNormal', label: '低于正常', desc: '稍低的优先级' },
  { value: 'Idle', label: '低', desc: '最低优先级' }
]

export const getPriorityLabel = (value: string): string => {
  const option = PRIORITY_OPTIONS.find(p => p.value === value)
  return option ? option.label : value
}

export const getPriorityDesc = (value: string): string => {
  const option = PRIORITY_OPTIONS.find(p => p.value === value)
  return option ? option.desc : ''
}
