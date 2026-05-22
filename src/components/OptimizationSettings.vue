<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { PRIORITY_OPTIONS } from '../config/priority'

const emit = defineEmits<{
  (e: 'saved'): void
}>()

interface Settings {
  enabledProcesses: string[]
  priority: string
  affinity: number[]
}

const ALL_PROCESSES = [
  { name: 'SGuardSvc64.exe', doodle: '🧸', desc: '进程守护服务' },
  { name: 'SGuard64.exe', doodle: '🐾', desc: '进程守护客户端' },
  { name: 'ACE-Tray.exe', doodle: '💫', desc: 'ACE 系统托盘' }
]

const cpuCoreCount = ref(navigator.hardwareConcurrency || 8)
const defaultAffinity = [(navigator.hardwareConcurrency || 8) - 1]
const settings = ref<Settings>({
  enabledProcesses: ['SGuardSvc64.exe', 'SGuard64.exe', 'ACE-Tray.exe'],
  priority: 'Idle',
  affinity: [...defaultAffinity]
})

const saveStatus = ref<'idle' | 'saving' | 'saved'>('idle')
const resetStatus = ref<'idle' | 'reset'>('idle')

const cpuCores = computed(() => {
  return Array.from({ length: cpuCoreCount.value }, (_, i) => ({
    id: i,
    label: `CPU ${i}`
  }))
})

const loadSettings = () => {
  try {
    const saved = localStorage.getItem('ace-helper-settings')
    if (saved) {
      const parsed = JSON.parse(saved)
      settings.value.enabledProcesses = parsed.enabledProcesses || [...ALL_PROCESSES.map(p => p.name)]
      settings.value.priority = parsed.priority || 'Idle'
      settings.value.affinity = parsed.affinity || [...defaultAffinity]
    } else {
      settings.value.enabledProcesses = [...ALL_PROCESSES.map(p => p.name)]
      settings.value.priority = 'Idle'
      settings.value.affinity = [...defaultAffinity]
    }
  } catch (e) {
    console.error('[ACE Helper] 加载设置失败:', e)
  }
}

const resetSettings = () => {
  settings.value.enabledProcesses = [...ALL_PROCESSES.map(p => p.name)]
  settings.value.priority = 'Idle'
  settings.value.affinity = [...defaultAffinity]
  localStorage.removeItem('ace-helper-settings')
  resetStatus.value = 'reset'
  setTimeout(() => {
    resetStatus.value = 'idle'
  }, 2000)
  console.log('[ACE Helper] 已恢复默认设置')
}

const saveSettings = async () => {
  saveStatus.value = 'saving'
  try {
    localStorage.setItem('ace-helper-settings', JSON.stringify(settings.value))
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
      <div class="header-icon">🧰</div>
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
        <span v-if="resetStatus === 'idle'">🔄 恢复默认</span>
        <span v-else>✅ 已恢复</span>
      </button>
      <button
        class="save-btn"
        :class="{ saving: saveStatus === 'saving', saved: saveStatus === 'saved' }"
        :disabled="saveStatus === 'saving'"
        @click="saveSettings"
      >
        <span v-if="saveStatus === 'idle'">💾 保存设置</span>
        <span v-else-if="saveStatus === 'saving'">⏳ 保存中…</span>
        <span v-else>✅ 已保存</span>
      </button>
    </div>

    <!-- 进程配置 -->
    <section class="settings-section">
      <div class="section-title">
        <span class="icon">📋</span>
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
          <span class="proc-icon">{{ proc.doodle }}</span>
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
        <span class="icon">⚡</span>
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

    <!-- CPU相关性设置 -->
    <section class="settings-section">
      <div class="section-title">
        <span class="icon">🔧</span>
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
        💡 修改设置后请点击「保存设置」按钮生效
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
  font-size: 24px;
  box-shadow: 0 8px 0 rgba(0,0,0,.06);
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
}

.section-title .icon {
  font-size: 18px;
}

.section-desc {
  margin: 0 0 12px;
  font-size: 12px;
  color: var(--muted);
  font-weight: 700;
}

/* 进程列表 */
.process-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.process-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  border: 3px solid var(--line);
  border-radius: 14px 12px 16px 14px;
  background: rgba(255,255,255,.5);
  cursor: pointer;
  transition: transform .1s ease, background .1s ease;
}

.process-item:hover {
  transform: translateX(3px);
  background: rgba(255,255,255,.8);
}

.process-item.checked {
  background: rgba(185,251,192,.5);
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
  position: relative;
  flex-shrink: 0;
}

.process-item.checked .checkbox-custom::after {
  content: '✓';
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  font-size: 14px;
  font-weight: 900;
  color: var(--ink);
}

.proc-icon {
  font-size: 22px;
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

/* 优先级列表 */
.priority-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.priority-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  border: 3px solid var(--line);
  border-radius: 14px 12px 16px 14px;
  background: rgba(255,255,255,.5);
  cursor: pointer;
  transition: transform .1s ease, background .1s ease;
}

.priority-item:hover {
  transform: translateX(3px);
  background: rgba(255,255,255,.8);
}

.priority-item.selected {
  background: rgba(169,214,255,.5);
  border-color: var(--line);
  box-shadow: 0 6px 0 rgba(0,0,0,.05);
}

.priority-item input[type="radio"] {
  display: none;
}

.radio-custom {
  width: 22px;
  height: 22px;
  border: 3px solid var(--line);
  border-radius: 999px;
  background: rgba(255,255,255,.8);
  position: relative;
  flex-shrink: 0;
}

.priority-item.selected .radio-custom::after {
  content: '';
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: 10px;
  height: 10px;
  border-radius: 999px;
  background: var(--ink);
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

/* CPU 核心网格 */
.affinity-controls {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 12px;
}

.btn.small {
  padding: 6px 12px;
  font-size: 12px;
  border: 2px solid var(--line);
  border-radius: 12px;
  background: rgba(185,251,192,.7);
  box-shadow: 0 4px 0 rgba(0,0,0,.05);
  font-weight: 900;
  cursor: pointer;
  transition: transform .1s ease;
}

.btn.small:hover {
  transform: translateY(-1px);
}

.core-count {
  font-size: 12px;
  color: var(--muted);
  font-weight: 700;
}

.core-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
  gap: 8px;
  max-height: 220px;
  overflow-y: auto;
  padding-right: 4px;
}

.core-grid::-webkit-scrollbar {
  width: 10px;
}

.core-grid::-webkit-scrollbar-track {
  background: rgba(255,255,255,.3);
  border-radius: 10px;
}

.core-grid::-webkit-scrollbar-thumb {
  background: rgba(185,251,192,.8);
  border: 3px solid var(--line);
  border-radius: 10px;
}

.core-grid::-webkit-scrollbar-thumb:hover {
  background: rgba(185,251,192,1);
}

.core-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  border: 3px solid var(--line);
  border-radius: 14px 12px 16px 14px;
  background: rgba(255,255,255,.5);
  cursor: pointer;
  transition: transform .1s ease, background .1s ease;
}

.core-item:hover {
  transform: translateX(3px);
  background: rgba(255,255,255,.8);
}

.core-item.selected {
  background: rgba(185,251,192,.5);
}

.core-item input[type="checkbox"] {
  display: none;
}

.core-item .checkbox-custom {
  width: 22px;
  height: 22px;
  border: 3px solid var(--line);
  border-radius: 8px;
  background: rgba(255,255,255,.8);
  position: relative;
  flex-shrink: 0;
}

.core-item.selected .checkbox-custom::after {
  content: '✓';
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  font-size: 14px;
  font-weight: 900;
  color: var(--ink);
}

.core-label {
  font-weight: 900;
  font-size: 14px;
}

/* 底部提示 */
.settings-footer {
  margin-top: 4px;
}

.hint-box {
  padding: 10px 14px;
  border: 3px dashed rgba(43,43,43,.4);
  border-radius: 14px 12px 16px 14px;
  background: rgba(255,255,255,.4);
  font-size: 12px;
  font-weight: 800;
  color: var(--muted);
}
</style>
