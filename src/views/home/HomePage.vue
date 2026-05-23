<script setup lang="ts">
import { computed } from 'vue'
import TopBar from '../../components/TopBar.vue'
import ProcessCard from '../../components/ProcessCard.vue'
import LogSection from '../../components/LogSection.vue'
import { SettingsIcon } from '../../components/icons'
import type { ProcessData } from '../../components/ProcessCard.vue'
import type { ProcessListItem } from '../../types'

const props = defineProps<{
  metaText: string
  isAutoMonitoring: boolean
  logs: string[]
  checkedCount: number
  enabledProcesses: ProcessListItem[]
  processStates: Record<string, ProcessData['state']>
  processDetails: Record<string, { lastUpdated: string; priority?: string; affinity?: string; coreCount?: number }>
  detailOpenStates: Record<string, boolean>
}>()

const emit = defineEmits<{
  (e: 'optimize'): void
  (e: 'getInfo'): void
  (e: 'toggleMonitor'): void
  (e: 'toggleCard', name: string): void
  (e: 'goToSettings'): void
}>()

// 根据设置和状态计算当前显示的进程
const processes = computed<ProcessData[]>(() => {
  return props.enabledProcesses.map(p => {
    const name = p.name
    return {
      name,
      doodle: p.doodle,
      state: props.processStates[name] || 'offline',
      lastUpdated: props.processDetails[name]?.lastUpdated || '',
      foundThisRun: props.processStates[name] === 'optimized',
      detailOpen: props.detailOpenStates[name] || false,
      priority: props.processDetails[name]?.priority,
      affinity: props.processDetails[name]?.affinity,
      coreCount: props.processDetails[name]?.coreCount
    }
  })
})

// 检查是否有启用任何进程
const hasEnabledProcesses = computed(() => {
  return props.enabledProcesses.length > 0
})

const handleToggleCard = (index: number) => {
  const processName = processes.value[index]?.name
  if (processName) {
    emit('toggleCard', processName)
  }
}
</script>

<template>
  <div class="home-page">
    <div class="floaties" aria-hidden="true">
      <div class="floaty b">A/B!!</div>
      <div class="floaty a">像贴纸一样乖乖的</div>
      <div class="floaty c">小爪机彩蛋</div>
    </div>

    <TopBar
      :meta-text="metaText"
      :is-auto-monitoring="isAutoMonitoring"
      @optimize="emit('optimize')"
      @getinfo="emit('getInfo')"
      @toggle-monitor="emit('toggleMonitor')"
    />

    <section class="grid" aria-label="进程卡片">
      <!-- 没有启用任何进程时的提示 -->
      <div v-if="!hasEnabledProcesses" class="empty-tip" @click="emit('goToSettings')">
        <div class="empty-tip-icon">
          <SettingsIcon :size="48" />
        </div>
        <div class="empty-tip-text">还没有勾选要优化的进程哦</div>
        <div class="empty-tip-sub">点击这里 → 去设置里勾选进程</div>
      </div>

      <ProcessCard
        v-for="(process, index) in processes"
        :key="process.name"
        :process="process"
        :index="index"
        @toggle="handleToggleCard(index)"
      />
    </section>

    <LogSection :logs="logs" :checked-count="checkedCount" />
  </div>
</template>

<style scoped>
.home-page {
  position: relative;
}

.floaties {
  position: fixed;
  inset: 0;
  pointer-events: none;
  z-index: 0;
}

.floaty {
  position: absolute;
  font-family: 'Caveat', cursive;
  color: #a78bfa;
  opacity: 0.35;
  font-size: 18px;
  transform: rotate(-6deg);
}

.floaty.a {
  top: 18%;
  right: 6%;
}

.floaty.b {
  top: 10%;
  right: 12%;
  font-size: 22px;
  transform: rotate(8deg);
  color: #f472b6;
}

.floaty.c {
  top: 28%;
  right: 4%;
  font-size: 14px;
  transform: rotate(-12deg);
}

/* 给 grid 添加上边距，与 TopBar 保持间距 */
.grid {
  margin-top: 14px;
}

.empty-tip {
  grid-column: 1 / -1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 60px 20px;
  background: linear-gradient(135deg, #fefce8 0%, #fef3c7 100%);
  border: 3px dashed #fbbf24;
  border-radius: 20px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.empty-tip:hover {
  transform: scale(1.02);
  border-color: #f59e0b;
  background: linear-gradient(135deg, #fef9c3 0%, #fde68a 100%);
}

.empty-tip-icon {
  margin-bottom: 12px;
  color: #f59e0b;
}

.empty-tip-text {
  font-size: 18px;
  font-weight: 600;
  color: #92400e;
  margin-bottom: 8px;
}

.empty-tip-sub {
  font-size: 14px;
  color: #a16207;
}
</style>
