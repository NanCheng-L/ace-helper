<script setup lang="ts">
import { PlayIcon, PauseIcon, SearchIcon, ZapIcon } from './icons'

defineProps<{
  metaText: string
  isAutoMonitoring?: boolean
}>()

defineEmits<{
  (e: 'optimize'): void
  (e: 'getinfo'): void
  (e: 'toggleMonitor'): void
}>()
</script>

<template>
  <header class="topbar">
    <img class="topicon" src="/src/assets/app-icon-32.png" alt="" aria-hidden="true" />
    <div class="headline">
      <div class="h">
        ACE 进程状态检测
        <span v-if="isAutoMonitoring" class="auto-badge">自动监听中</span>
      </div>
      <div class="meta">{{ metaText }}</div>
    </div>
    <div class="spacer"></div>
    <button class="btn secondary" type="button" @click="$emit('toggleMonitor')">
      <span class="mini">
        <component :is="isAutoMonitoring ? PauseIcon : PlayIcon" :size="12" />
      </span>
      {{ isAutoMonitoring ? '停止监听' : '开始监听' }}
    </button>
    <button v-if="true" class="btn secondary" type="button" @click="$emit('getinfo')">
      <span class="mini">
        <SearchIcon :size="12" />
      </span>
      获取进程
    </button>
    <button class="btn" type="button" @click="$emit('optimize')">
      <span class="mini">
        <ZapIcon :size="12" />
      </span>
      一键优化
    </button>
  </header>
</template>

<style scoped>
.auto-badge {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  margin-left: 8px;
  padding: 2px 8px;
  border: 2px solid var(--line);
  border-radius: 999px;
  background: rgba(185, 251, 192, 0.8);
  font-size: 11px;
  font-weight: 700;
  color: var(--ink);
}
</style>
