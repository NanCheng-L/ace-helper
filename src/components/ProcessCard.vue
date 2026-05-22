<script setup lang="ts">
import { computed } from 'vue'
import { getPriorityLabel } from '../config/priority'

export interface ProcessData {
  name: string
  doodle: string
  state: 'offline' | 'scanning' | 'optimizing' | 'optimized' | 'failed'
  lastUpdated: string
  foundThisRun: boolean
  detailOpen: boolean
  priority?: string
  affinity?: string
  coreCount?: number
}

const props = defineProps<{
  process: ProcessData
  index: number
}>()

const emit = defineEmits<{
  (e: 'toggle'): void
}>()

const stateText: Record<string, string> = {
  offline: '离线',
  scanning: '检测中',
  optimizing: '优化中',
  optimized: '已优化',
  failed: '优化失败'
}

const moods: Record<string, string> = {
  offline: '( ˘︹˘ )',
  scanning: '( •̀ ω •́ )✧',
  optimizing: '(ง •̀_•́)ง',
  optimized: '(ﾉ◕ヮ◕)ﾉ*:･ﾟ✧',
  failed: '(；´д｀)ゞ'
}

const hints: Record<string, string> = {
  offline: '没有发现它在运行哦',
  scanning: '我在努力找它……',
  optimizing: '正在整理中（像把桌面贴纸摆整齐）',
  optimized: '已经帮你整理好啦！',
  failed: '哎呀，出错了。去日志看看，或者再试一次？'
}

const descriptions: Record<string, string> = {
  offline: '小声：它今天会不会上线呢…',
  scanning: '我去找找看！',
  optimizing: '整理整理整理——',
  optimized: '通关！奖励你一颗星星 ✦',
  failed: '这个进程有点顽固…'
}

const cardClasses = computed(() => ({
  'card': true,
  'expanded': props.process.detailOpen,
  'optimized-state': props.process.state === 'optimized'
}))

const handleClick = () => {
  emit('toggle')
}
</script>

<template>
  <article :class="cardClasses" @click="handleClick">
    <div class="tape" aria-hidden="true"></div>
    <div class="cardhead">
      <div class="proc-icon" aria-hidden="true">{{ process.doodle }}</div>
      <div class="proc-title">
        <div class="name">{{ process.name }}</div>
        <div class="desc">{{ descriptions[process.state] }}</div>
      </div>
      <div class="tag" :class="process.state">
        <span class="dot" aria-hidden="true"></span>
        <span class="spin" aria-hidden="true"></span>
        <span class="label">{{ stateText[process.state] }}</span>
      </div>
    </div>
    <div class="cardbody">
      <div class="minirow">最近更新时间：<b>{{ process.lastUpdated || '—' }}</b></div>
      <div class="minirow">小表情：<b>{{ moods[process.state] }}</b></div>
    </div>
    <div class="expand">
      <div class="kv"><div>检测策略</div><span>自动扫描 → 发现即整理</span></div>
      <div class="kv"><div>提示</div><span>{{ hints[process.state] }}</span></div>
      <div v-if="process.priority" class="kv"><div>优先级</div><span>{{ getPriorityLabel(process.priority) }}</span></div>
      <div v-if="process.affinity" class="kv"><div>CPU 亲和性</div><span>CPU {{ process.affinity }}</span></div>
      <div v-if="process.coreCount" class="kv"><div>使用核心数</div><span>{{ process.coreCount }} 个核心</span></div>
    </div>
    <svg class="sparkle" viewBox="0 0 64 64" aria-hidden="true">
      <path d="M32 5l4.5 14.8L52 24l-15.5 4.2L32 43 27.5 28.2 12 24l15.5-4.2L32 5z" fill="rgba(255,230,109,.95)" stroke="rgba(43,43,43,.75)" stroke-width="3" stroke-linejoin="round"/>
      <path v-if="index === 0" d="M51 36l2.2 7.2L61 45l-7.8 2.1L51 54l-2.2-6.9L41 45l7.8-1.8L51 36z" fill="rgba(185,251,192,.95)" stroke="rgba(43,43,43,.75)" stroke-width="3" stroke-linejoin="round"/>
      <path v-else d="M12 37l2.2 7.2L22 46l-7.8 2.1L12 55l-2.2-6.9L2 46l7.8-1.8L12 37z" fill="rgba(169,214,255,.95)" stroke="rgba(43,43,43,.75)" stroke-width="3" stroke-linejoin="round"/>
    </svg>
  </article>
</template>