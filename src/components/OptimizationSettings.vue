<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { PRIORITY_OPTIONS } from '../config/priority'
import { IO_PRIORITY_OPTIONS } from '../config/ioPriority'
import { ToolsIcon, RotateIcon, SaveIcon, CheckIcon, ShieldIcon, PawIcon, TrayIcon, AppIcon, ZapIcon, CpuIcon, HardDriveIcon } from './icons'
import { loadConfig, updateOptimizationSettings } from '../utils/configStorage'
import type { Component } from 'vue'

const emit = defineEmits<{
  (e: 'saved'): void
}>()

interface ProcessInfo {
  name: string
  icon: Component
  desc: string
}

const ALL_PROCESSES: ProcessInfo[] = [
  { name: 'SGuardSvc64.exe', icon: ShieldIcon, desc: '进程守护服务' },
  { name: 'SGuard64.exe', icon: PawIcon, desc: '进程守护客户端' },
  { name: 'ACE-Tray.exe', icon: TrayIcon, desc: 'ACE 系统托盘' },
  { name: 'ace-helper.exe', icon: AppIcon, desc: 'ACE 小助手本体' }
]

const cpuCoreCount = ref(navigator.hardwareConcurrency || 8)
const defaultAffinity = [(navigator.hardwareConcurrency || 8) - 1]

// 默认启用的进程
const DEFAULT_ENABLED_PROCESSES = ['SGuardSvc64.exe', 'SGuard64.exe', 'ACE-Tray.exe', 'ace-helper.exe']

interface Settings {
  enabledProcesses: string[]
  priority: string
  affinity: number[]
  ioPriority: string
}

const settings = ref<Settings>({
  enabledProcesses: [...DEFAULT_ENABLED_PROCESSES],
  priority: 'Idle',
  affinity: [...defaultAffinity],
  ioPriority: 'VeryLow'
})

const saveStatus = ref<'idle' | 'saving' | 'saved'>('idle')
const resetStatus = ref<'idle' | 'reset'>('idle')

const cpuCores = computed(() => {
  return Array.from({ length: cpuCoreCount.value }, (_, i) => ({
    id: i,
    label: `CPU ${i}`
  }))
})

const loadSettings = async () => {
  try {
    const config = await loadConfig()
    if (config.optimizationSettings) {
      settings.value.enabledProcesses = config.optimizationSettings.enabledProcesses || [...DEFAULT_ENABLED_PROCESSES]
      settings.value.priority = config.optimizationSettings.priority || 'Idle'
      // 处理空数组情况：如果 affinity 为空数组，使用默认值
      settings.value.affinity = (config.optimizationSettings.affinity && config.optimizationSettings.affinity.length > 0)
        ? config.optimizationSettings.affinity
        : [...defaultAffinity]
      settings.value.ioPriority = config.optimizationSettings.ioPriority || 'VeryLow'
    } else {
      settings.value.enabledProcesses = [...DEFAULT_ENABLED_PROCESSES]
      settings.value.priority = 'Idle'
      settings.value.affinity = [...defaultAffinity]
      settings.value.ioPriority = 'VeryLow'
    }
  } catch (e) {
    console.error('[ACE Helper] 加载设置失败:', e)
  }
}

const resetSettings = async () => {
  settings.value.enabledProcesses = [...DEFAULT_ENABLED_PROCESSES]
  settings.value.priority = 'Idle'
  settings.value.affinity = [...defaultAffinity]
  settings.value.ioPriority = 'VeryLow'
  try {
    await updateOptimizationSettings({
      enabledProcesses: [...DEFAULT_ENABLED_PROCESSES],
      priority: 'Idle',
      affinity: [...defaultAffinity],
      ioPriority: 'VeryLow'
    })
  } catch (e) {
    console.error('[ACE Helper] 重置设置失败:', e)
  }
  resetStatus.value = 'reset'
  setTimeout(() => {
    resetStatus.value = 'idle'
  }, 2000)
  console.log('[ACE Helper] 已恢复默认设置')
}

const saveSettings = async () => {
  saveStatus.value = 'saving'
  try {
    await updateOptimizationSettings({
      enabledProcesses: settings.value.enabledProcesses,
      priority: settings.value.priority,
      affinity: settings.value.affinity,
      ioPriority: settings.value.ioPriority
    })
    console.log('[ACE Helper] 设置已保存:', settings.value)
    saveStatus.value = 'saved'
    emit('saved')
    setTimeout(() => {
      saveStatus.value = 'idle'
    }, 2000)
  } catch (e) {
    console.error('[ACE Helper] 保存设置失败:', e)
    saveStatus.value = 'idle'
  }
}

const toggleProcess = (processName: string) => {
  const idx = settings.value.enabledProcesses.indexOf(processName)
  if (idx === -1) {
    settings.value.enabledProcesses.push(processName)
  } else {
    settings.value.enabledProcesses.splice(idx, 1)
  }
}

const isProcessChecked = (processName: string) => {
  return settings.value.enabledProcesses.includes(processName)
}

const setPriority = (priority: string) => {
  settings.value.priority = priority
}

const setIoPriority = (ioPriority: string) => {
  settings.value.ioPriority = ioPriority
}

const toggleCore = (coreId: number) => {
  const idx = settings.value.affinity.indexOf(coreId)
  if (idx === -1) {
    settings.value.affinity.push(coreId)
  } else {
    settings.value.affinity.splice(idx, 1)
  }
}

const isCoreChecked = (coreId: number) => {
  return settings.value.affinity.includes(coreId)
}

const toggleAllCores = () => {
  if (settings.value.affinity.length === cpuCoreCount.value) {
    settings.value.affinity = []
  } else {
    settings.value.affinity = cpuCores.value.map(c => c.id)
  }
}

onMounted(() => {
  loadSettings()
})
</script>

<template>
  <div class="settings-page">
    <div class="settings-header">
      <div class="header-icon">
        <ToolsIcon :size="24" />
      </div>
      <div class="header-text">
        <h2>优化设置</h2>
        <p>配置进程监控和优化参数</p>
      </div>
      <div class="spacer"></div>
      <button
        class="reset-btn"
        :class="{ reset: resetStatus === 'reset' }"
        @click="resetSettings"
      >
        <span v-if="resetStatus === 'idle'" class="btn-content">
          <RotateIcon :size="16" />
          <span>恢复默认</span>
        </span>
        <span v-else class="btn-content">
          <CheckIcon :size="16" />
          <span>已恢复</span>
        </span>
      </button>
      <button
        class="save-btn"
        :class="{ saving: saveStatus === 'saving', saved: saveStatus === 'saved' }"
        :disabled="saveStatus === 'saving'"
        @click="saveSettings"
      >
        <span v-if="saveStatus === 'idle'" class="btn-content">
          <SaveIcon :size="16" />
          <span>保存设置</span>
        </span>
        <span v-else-if="saveStatus === 'saving'">保存中…</span>
        <span v-else class="btn-content">
          <CheckIcon :size="16" />
          <span>已保存</span>
        </span>
      </button>
    </div>

    <!-- 进程配置 -->
    <section class="settings-section">
      <div class="section-title">
        <ShieldIcon :size="18" />
        <span>进程配置</span>
      </div>
      <p class="section-desc">勾选要在首页显示的进程</p>
      <div class="process-list">
        <label
          v-for="proc in ALL_PROCESSES"
          :key="proc.name"
          class="process-item"
          :class="{ checked: isProcessChecked(proc.name) }"
        >
          <input
            type="checkbox"
            :checked="isProcessChecked(proc.name)"
            @change="toggleProcess(proc.name)"
          />
          <span class="checkbox-custom"></span>
          <span class="proc-icon">
            <component :is="proc.icon" :size="20" />
          </span>
          <span class="proc-info">
            <span class="proc-name">{{ proc.name }}</span>
            <span class="proc-desc">{{ proc.desc }}</span>
          </span>
        </label>
      </div>
    </section>

    <!-- 优先级设置 -->
    <section class="settings-section">
      <div class="section-title">
        <ZapIcon :size="18" />
        <span>优先级设置</span>
      </div>
      <p class="section-desc">选择进程优化后的优先级</p>
      <div class="priority-list">
        <label
          v-for="p in PRIORITY_OPTIONS"
          :key="p.value"
          class="priority-item"
          :class="{ selected: settings.priority === p.value }"
        >
          <input
            type="radio"
            name="priority"
            :value="p.value"
            :checked="settings.priority === p.value"
            @change="setPriority(p.value)"
          />
          <span class="radio-custom"></span>
          <span class="priority-info">
            <span class="priority-label">{{ p.label }}</span>
            <span class="priority-desc">{{ p.desc }}</span>
          </span>
        </label>
      </div>
    </section>

    <!-- 磁盘 I/O 优先级设置 -->
    <section class="settings-section">
      <div class="section-title">
        <HardDriveIcon :size="18" />
        <span>磁盘 I/O 优先级</span>
      </div>
      <p class="section-desc">设置进程的磁盘 I/O 优先级（默认最低）</p>
      <div class="priority-list">
        <label
          v-for="p in IO_PRIORITY_OPTIONS"
          :key="p.value"
          class="priority-item"
          :class="{ selected: settings.ioPriority === p.value }"
        >
          <input
            type="radio"
            name="ioPriority"
            :value="p.value"
            :checked="settings.ioPriority === p.value"
            @change="setIoPriority(p.value)"
          />
          <span class="radio-custom"></span>
          <span class="priority-info">
            <span class="priority-label">{{ p.label }}</span>
            <span class="priority-desc">{{ p.desc }}</span>
          </span>
        </label>
      </div>
    </section>

    <!-- CPU相关性设置 -->
    <section class="settings-section">
      <div class="section-title">
        <CpuIcon :size="18" />
        <span>CPU 相关性</span>
      </div>
      <p class="section-desc">设置进程可使用的 CPU 核心（留空表示使用所有核心）</p>
      <div class="affinity-controls">
        <button class="btn small" @click="toggleAllCores">
          {{ settings.affinity.length === cpuCoreCount ? '取消全选' : '全选' }}
        </button>
        <span class="core-count">共 {{ cpuCoreCount }} 个核心，已选 {{ settings.affinity.length }} 个</span>
      </div>
      <div class="core-grid">
        <label
          v-for="core in cpuCores"
          :key="core.id"
          class="core-item"
          :class="{ selected: isCoreChecked(core.id) }"
        >
          <input
            type="checkbox"
            :checked="isCoreChecked(core.id)"
            @change="toggleCore(core.id)"
          />
          <span class="checkbox-custom"></span>
          <span class="core-label">{{ core.label }}</span>
        </label>
      </div>
    </section>

    <div class="settings-footer">
      <div class="hint-box">
        修改设置后请点击「保存设置」按钮生效
      </div>
    </div>
  </div>
</template>

<style scoped>
.settings-page {
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding: 4px;
}

.settings-header {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 14px 16px;
  border: 3px solid var(--line);
  border-radius: 20px 16px 22px 14px;
  background: linear-gradient(180deg, rgba(255,230,109,.4), rgba(255,255,255,.5));
}

.header-icon {
  width: 48px;
  height: 48px;
  border: 3px solid var(--line);
  border-radius: 14px 18px 12px 20px;
  background: rgba(255,255,255,.6);
  display: grid;
  place-items: center;
  box-shadow: 0 8px 0 rgba(0,0,0,.06);
  color: var(--ink);
}

.header-text h2 {
  margin: 0;
  font-size: 20px;
  font-weight: 900;
  text-shadow: 0 2px 0 rgba(0,0,0,.12);
}

.header-text p {
  margin: 4px 0 0;
  font-size: 13px;
  color: var(--muted);
  font-weight: 700;
}

.spacer {
  flex: 1;
}

.btn-content {
  display: flex;
  align-items: center;
  gap: 6px;
}

.reset-btn {
  border: 3px solid var(--line);
  border-radius: 16px 18px 14px 20px;
  padding: 10px 16px;
  background: rgba(255,200,200,.8);
  box-shadow: 0 8px 0 rgba(0,0,0,.06);
  font-weight: 900;
  font-size: 14px;
  cursor: pointer;
  transition: transform .12s ease, box-shadow .12s ease, background .2s ease;
  user-select: none;
  color: var(--ink);
}

.reset-btn:hover {
  transform: translateY(-2px);
  box-shadow: 0 10px 0 rgba(0,0,0,.07);
  background: rgba(255,180,180,.8);
}

.reset-btn:active {
  transform: translateY(2px);
  box-shadow: 0 6px 0 rgba(0,0,0,.06);
}

.reset-btn.reset {
  background: rgba(185,251,192,1);
}

.save-btn {
  border: 3px solid var(--line);
  border-radius: 16px 18px 14px 20px;
  padding: 10px 16px;
  background: rgba(185,251,192,.8);
  box-shadow: 0 8px 0 rgba(0,0,0,.06);
  font-weight: 900;
  font-size: 14px;
  cursor: pointer;
  transition: transform .12s ease, box-shadow .12s ease, background .2s ease;
  user-select: none;
  margin-left: 8px;
  color: var(--ink);
}

.save-btn:hover {
  transform: translateY(-2px);
  box-shadow: 0 10px 0 rgba(0,0,0,.07);
}

.save-btn:active {
  transform: translateY(2px);
  box-shadow: 0 6px 0 rgba(0,0,0,.06);
}

.save-btn.saving {
  background: rgba(169,214,255,.8);
  cursor: wait;
}

.save-btn.saved {
  background: rgba(185,251,192,1);
}

.save-btn:disabled {
  cursor: not-allowed;
}

.settings-section {
  border: 3px solid var(--line);
  border-radius: 20px 16px 22px 14px;
  background: rgba(255,255,255,.65);
  padding: 14px 16px;
  box-shadow: var(--shadow);
}

.section-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 16px;
  font-weight: 900;
  margin-bottom: 6px;
  color: var(--ink);
}

.section-desc {
  margin: 0 0 12px;
  font-size: 12px;
  color: var(--muted);
  font-weight: 700;
}

.process-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.process-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 14px;
  border: 3px solid var(--line);
  border-radius: 14px 12px 16px 14px;
  background: rgba(255,255,255,.5);
  cursor: pointer;
  transition: transform .1s ease, background .1s ease;
}

.process-item:hover {
  background: rgba(255,255,255,.8);
}

.process-item.checked {
  background: rgba(185,251,192,.4);
}

.process-item input[type="checkbox"] {
  display: none;
}

.checkbox-custom {
  width: 22px;
  height: 22px;
  border: 3px solid var(--line);
  border-radius: 8px;
  background: rgba(255,255,255,.8);
  display: grid;
  place-items: center;
  flex-shrink: 0;
  transition: background .2s ease;
}

.process-item.checked .checkbox-custom {
  background: rgba(185,251,192,1);
}

.checkbox-custom::after {
  content: '✓';
  font-size: 14px;
  font-weight: 900;
  color: var(--ink);
  opacity: 0;
  transition: opacity .2s ease;
  display: flex;
  align-items: center;
  justify-content: center;
  line-height: 1;
}

.process-item.checked .checkbox-custom::after {
  opacity: 1;
}

.proc-icon {
  width: 46px;
  height: 46px;
  border: 3px solid var(--line);
  border-radius: 14px 18px 12px 20px;
  display: grid;
  place-items: center;
  flex-shrink: 0;
  color: var(--ink);
  box-shadow: 0 8px 0 rgba(0,0,0,.06);
  transform: rotate(-2deg);
}

/* 不同进程的图标背景色 */
.process-item:nth-child(1) .proc-icon {
  background: radial-gradient(circle at 30% 30%, #fff, rgba(255,200,100,.3));
}

.process-item:nth-child(2) .proc-icon {
  background: radial-gradient(circle at 30% 30%, #fff, rgba(255,150,150,.3));
}

.process-item:nth-child(3) .proc-icon {
  background: radial-gradient(circle at 30% 30%, #fff, rgba(150,200,255,.3));
}

.process-item:nth-child(4) .proc-icon {
  background: radial-gradient(circle at 30% 30%, #fff, rgba(185,251,192,.3));
}

.proc-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.proc-name {
  font-weight: 900;
  font-size: 14px;
}

.proc-desc {
  font-size: 12px;
  color: var(--muted);
  font-weight: 700;
}

.priority-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.priority-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 14px;
  border: 3px solid var(--line);
  border-radius: 14px 12px 16px 14px;
  background: rgba(255,255,255,.5);
  cursor: pointer;
  transition: transform .1s ease, background .1s ease;
}

.priority-item:hover {
  background: rgba(255,255,255,.8);
}

.priority-item.selected {
  background: rgba(255,230,109,.4);
}

.priority-item input[type="radio"] {
  display: none;
}

.radio-custom {
  width: 22px;
  height: 22px;
  border: 3px solid var(--line);
  border-radius: 50%;
  background: rgba(255,255,255,.8);
  display: grid;
  place-items: center;
  flex-shrink: 0;
  transition: background .2s ease;
}

.priority-item.selected .radio-custom {
  background: rgba(255,230,109,1);
}

.radio-custom::after {
  content: '';
  width: 10px;
  height: 10px;
  background: var(--ink);
  border-radius: 50%;
  opacity: 0;
  transition: opacity .2s ease;
}

.priority-item.selected .radio-custom::after {
  opacity: 1;
}

.priority-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.priority-label {
  font-weight: 900;
  font-size: 14px;
}

.priority-desc {
  font-size: 12px;
  color: var(--muted);
  font-weight: 700;
}

.affinity-controls {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 12px;
}

.btn.small {
  border: 3px solid var(--line);
  border-radius: 12px;
  padding: 6px 12px;
  background: rgba(169,214,255,.6);
  font-weight: 700;
  font-size: 12px;
  cursor: pointer;
  transition: transform .1s ease, background .1s ease;
}

.btn.small:hover {
  background: rgba(169,214,255,.8);
  transform: translateY(-1px);
}

.core-count {
  font-size: 12px;
  color: var(--muted);
  font-weight: 700;
}

.core-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(100px, 1fr));
  gap: 8px;
}

.core-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 10px;
  border: 3px solid var(--line);
  border-radius: 10px;
  background: rgba(255,255,255,.5);
  cursor: pointer;
  transition: transform .1s ease, background .1s ease;
  white-space: nowrap;
  overflow: hidden;
}

.core-item:hover {
  background: rgba(255,255,255,.8);
}

.core-item.selected {
  background: rgba(185,251,192,.4);
}

.core-item input[type="checkbox"] {
  display: none;
}

.core-item .checkbox-custom {
  width: 18px;
  height: 18px;
  border: 2px solid var(--line);
  border-radius: 4px;
  background: rgba(255,255,255,.8);
  display: grid;
  place-items: center;
  flex-shrink: 0;
  transition: background .2s ease;
}

.core-item.selected .checkbox-custom {
  background: rgba(185,251,192,1);
}

.core-item .checkbox-custom::after {
  content: '✓';
  font-size: 12px;
  font-weight: 900;
  color: var(--ink);
  opacity: 0;
  transition: opacity .2s ease;
  display: flex;
  align-items: center;
  justify-content: center;
  line-height: 1;
}

.core-item.selected .checkbox-custom::after {
  opacity: 1;
}

.core-label {
  font-weight: 700;
  font-size: 12px;
}

.settings-footer {
  margin-top: 8px;
}

.hint-box {
  border: 3px dashed var(--line);
  border-radius: 16px;
  padding: 12px 16px;
  background: rgba(255,230,109,.2);
  font-size: 13px;
  color: var(--muted);
  font-weight: 700;
  text-align: center;
}
</style>
