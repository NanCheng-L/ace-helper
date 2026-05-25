export interface IoPriorityOption {
  value: string
  label: string
  desc: string
}

export const IO_PRIORITY_OPTIONS: IoPriorityOption[] = [
  { value: 'Normal', label: '正常', desc: '标准磁盘 I/O 优先级' },
  { value: 'Low', label: '低', desc: '较低的磁盘 I/O 优先级' },
  { value: 'VeryLow', label: '非常低', desc: '最低磁盘 I/O 优先级' }
]

export const getIoPriorityLabel = (value: string): string => {
  const option = IO_PRIORITY_OPTIONS.find(p => p.value === value)
  return option ? option.label : value
}

export const getIoPriorityDesc = (value: string): string => {
  const option = IO_PRIORITY_OPTIONS.find(p => p.value === value)
  return option ? option.desc : ''
}
