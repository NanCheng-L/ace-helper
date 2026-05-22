<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch, nextTick } from 'vue'

// 定义组件接收的属性
const props = defineProps<{
  logs: string[]
  checkedCount: number
}>()

const clock = ref('--:--:--')
// 用于引用日志 DOM 元素，实现滚动
const logsRef = ref<HTMLElement | null>(null)
let clockInterval: number | null = null

const updateClock = () => {
  const now = new Date()
  const pad = (n: number) => String(n).padStart(2, '0')
  clock.value = `${pad(now.getHours())}:${pad(now.getMinutes())}:${pad(now.getSeconds())}`
}

// 自动滚动到最底部的函数
const scrollToBottom = () => {
  if (logsRef.value) {
    nextTick(() => {
      logsRef.value!.scrollTop = logsRef.value!.scrollHeight
    })
  }
}

// 监听 logs 变化，当有新日志时自动滚动到底部
watch(() => props.logs.length, () => {
  scrollToBottom()
})

onMounted(() => {
  updateClock()
  clockInterval = window.setInterval(updateClock, 1000)
  // 初始加载时也滚动到底部
  nextTick(() => {
    scrollToBottom()
  })
})

onUnmounted(() => {
  if (clockInterval) {
    clearInterval(clockInterval)
  }
})
</script>

<template>
  <section class="bottom" aria-label="底部信息">
    <div class="logbox">
      <h3>进程小日志</h3>
      <!-- 使用 ref 引用日志容器，并直接循环显示（顺序为原顺序，最新的在最下面） -->
      <div ref="logsRef" class="logs">
        <div v-for="(log, index) in logs" :key="index">{{ log }}</div>
      </div>
    </div>
    <div class="logbox">
      <h3>今日状态</h3>
      <div style="display:flex;gap:10px;flex-wrap:wrap;align-items:center">
        <div class="badge"><span class="tiny" aria-hidden="true"></span> 已检查 <span>{{ checkedCount }}</span> 项目标</div>
        <div class="badge" style="background:rgba(205,180,255,.45)">⏱️ 当前时间：<span>{{ clock }}</span></div>
      </div>
      <div style="margin-top:10px" class="hint" id="footerHint">
        小提示：这是前端 UI 原型（含状态切换演示），后续接入真实进程检测时，只要把状态更新接口对上就行啦。
      </div>
    </div>
  </section>
</template>
