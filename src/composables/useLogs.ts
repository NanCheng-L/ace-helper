import { ref } from 'vue'

export function useLogs() {
  const logs = ref<string[]>([])

  const nowStr = () => {
    const d = new Date()
    const pad = (n: number) => String(n).padStart(2, '0')
    return `${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`
  }

  // 添加日志，使用 push 把最新的添加到数组末尾，这样最新的在最下面（像终端一样）
  const addLog = (line: string) => {
    logs.value.push(`[${nowStr()}] ${line}`)
    if (logs.value.length > 30) {
      // 超过数量限制时，移除最旧的（数组开头）
      logs.value.shift()
    }
  }

  // 清空日志
  const clearLogs = () => {
    logs.value = []
  }

  // 初始化日志
  const initLog = (message: string) => {
    logs.value = [`[${nowStr()}] ${message}`]
  }

  return {
    logs,
    addLog,
    clearLogs,
    initLog
  }
}
