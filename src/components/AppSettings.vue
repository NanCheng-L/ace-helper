<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { isPermissionGranted, requestPermission } from '@tauri-apps/plugin-notification'
import { SettingsIcon, SaveIcon, CheckIcon, RocketIcon, WindowIcon, SearchIcon, BellIcon } from './icons'
import { loadConfig, updateAppSettings } from '../utils/configStorage'

interface AppSettings {
  autoStart: boolean
  minimizeToTray: boolean
  autoMonitor: boolean
  enableNotifications: boolean
}

const settings = ref<AppSettings>({
  autoStart: true,
  minimizeToTray: true,
  autoMonitor: true,
  enableNotifications: true
})

const saveStatus = ref<'idle' | 'saved'>('idle')

const loadSettings = async () => {
  try {
    const config = await loadConfig()
    if (config.appSettings) {
      settings.value = {
        autoStart: config.appSettings.autoStart ?? true,
        minimizeToTray: config.appSettings.minimizeToTray ?? true,
        autoMonitor: config.appSettings.autoMonitor ?? true,
        enableNotifications: config.appSettings.enableNotifications ?? true
      }
    }
  } catch (e) {
    console.error('[ACE Helper] 加载软件设置失败:', e)
  }
}

const saveSettings = async () => {
  try {
    await updateAppSettings({
      autoStart: settings.value.autoStart,
      minimizeToTray: settings.value.minimizeToTray,
      autoMonitor: settings.value.autoMonitor,
      enableNotifications: settings.value.enableNotifications
    })

    if (settings.value.enableNotifications) {
      try {
        const granted = await isPermissionGranted()
        if (!granted) {
          await requestPermission()
        }
      } catch {}
    }

    // 设置开机自启动
    try {
      const { isEnabled, enable, disable } = await import('@tauri-apps/plugin-autostart')
      const enabled = await isEnabled()

      if (settings.value.autoStart && !enabled) {
        await enable()
      } else if (!settings.value.autoStart && enabled) {
        await disable()
      }
    } catch (e) {
      console.error('[ACE Helper] 设置开机自启动失败:', e)
    }

    invoke('set_minimize_to_tray', { value: settings.value.minimizeToTray }).catch(() => {})

    saveStatus.value = 'saved'
    setTimeout(() => {
      saveStatus.value = 'idle'
    }, 2000)
  } catch (e) {
    console.error('[ACE Helper] 保存软件设置失败:', e)
    saveStatus.value = 'idle'
  }
}

onMounted(async () => {
  await loadSettings()
  invoke('set_minimize_to_tray', { value: settings.value.minimizeToTray }).catch(() => {})
})
</script>

<template>
  <div class="app-settings-page">
    <div class="settings-header">
      <div class="header-icon">
        <SettingsIcon :size="24" />
      </div>
      <div class="header-text">
        <h2>软件设置</h2>
        <p>配置软件的运行方式</p>
      </div>
      <div class="spacer"></div>
      <button
        class="save-btn"
        :class="{ saved: saveStatus === 'saved' }"
        @click="saveSettings"
      >
        <span v-if="saveStatus === 'idle'" class="btn-content">
          <SaveIcon :size="16" />
          <span>保存</span>
        </span>
        <span v-else class="btn-content">
          <CheckIcon :size="16" />
          <span>已保存</span>
        </span>
      </button>
    </div>

    <!-- 启动设置 -->
    <section class="settings-section">
      <div class="section-title">
        <RocketIcon :size="18" />
        <span>启动设置</span>
      </div>
      <p class="section-desc">软件启动时的行为</p>
      
      <div class="setting-list">
        <label class="setting-item">
          <div class="setting-info">
            <span class="setting-label">开机自动启动</span>
            <span class="setting-desc">电脑开机时自动运行软件（静默进入系统托盘）</span>
          </div>
          <div class="toggle-switch" :class="{ active: settings.autoStart }">
            <input 
              type="checkbox" 
              v-model="settings.autoStart"
            />
            <span class="toggle-slider"></span>
          </div>
        </label>
      </div>
    </section>

    <!-- 窗口设置 -->
    <section class="settings-section">
      <div class="section-title">
        <WindowIcon :size="18" />
        <span>窗口设置</span>
      </div>
      <p class="section-desc">窗口关闭和显示行为</p>
      
      <div class="setting-list">
        <label class="setting-item">
          <div class="setting-info">
            <span class="setting-label">关闭窗口时最小化</span>
            <span class="setting-desc">点击关闭按钮时最小化到托盘而不是退出</span>
          </div>
          <div class="toggle-switch" :class="{ active: settings.minimizeToTray }">
            <input 
              type="checkbox" 
              v-model="settings.minimizeToTray"
            />
            <span class="toggle-slider"></span>
          </div>
        </label>
      </div>
    </section>

    <!-- 进程监控 -->
    <section class="settings-section">
      <div class="section-title">
        <SearchIcon :size="18" />
        <span>进程监控</span>
      </div>
      <p class="section-desc">软件启动时的自动监听行为</p>

      <div class="setting-list">
        <label class="setting-item">
          <div class="setting-info">
            <span class="setting-label">启动时自动监听</span>
            <span class="setting-desc">软件打开后自动开始监听进程（首页可临时关闭）</span>
          </div>
          <div class="toggle-switch" :class="{ active: settings.autoMonitor }">
            <input
              type="checkbox"
              v-model="settings.autoMonitor"
            />
            <span class="toggle-slider"></span>
          </div>
        </label>
      </div>
    </section>

    <!-- 通知设置 -->
    <section class="settings-section">
      <div class="section-title">
        <BellIcon :size="18" />
        <span>通知设置</span>
      </div>
      <p class="section-desc">优化完成后的系统通知</p>
      
      <div class="setting-list">
        <label class="setting-item">
          <div class="setting-info">
            <span class="setting-label">系统通知</span>
            <span class="setting-desc">进程优化成功时在右下角弹出通知提醒</span>
          </div>
          <div class="toggle-switch" :class="{ active: settings.enableNotifications }">
            <input 
              type="checkbox" 
              v-model="settings.enableNotifications"
            />
            <span class="toggle-slider"></span>
          </div>
        </label>
      </div>
    </section>

  </div>
</template>

<style scoped>
.app-settings-page {
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

.save-btn {
  border: 3px solid var(--line);
  border-radius: 16px 18px 14px 20px;
  padding: 10px 20px;
  background: rgba(185,251,192,.8);
  box-shadow: 0 8px 0 rgba(0,0,0,.06);
  font-weight: 900;
  font-size: 14px;
  cursor: pointer;
  transition: transform .12s ease, box-shadow .12s ease, background .2s ease;
  user-select: none;
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

.save-btn.saved {
  background: rgba(185,251,192,1);
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

.setting-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.setting-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 14px;
  border: 3px solid var(--line);
  border-radius: 14px 12px 16px 14px;
  background: rgba(255,255,255,.5);
  cursor: pointer;
  transition: transform .1s ease, background .1s ease;
}

.setting-item:hover {
  background: rgba(255,255,255,.8);
}

.setting-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.setting-label {
  font-weight: 900;
  font-size: 14px;
}

.setting-desc {
  font-size: 12px;
  color: var(--muted);
  font-weight: 700;
}

/* Toggle Switch */
.toggle-switch {
  position: relative;
  width: 52px;
  height: 28px;
  flex-shrink: 0;
}

.toggle-switch input {
  opacity: 0;
  width: 0;
  height: 0;
}

.toggle-slider {
  position: absolute;
  cursor: pointer;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(255,200,200,.8);
  border: 3px solid var(--line);
  border-radius: 999px;
  transition: .2s;
}

.toggle-slider:before {
  position: absolute;
  content: "";
  height: 18px;
  width: 18px;
  left: 2px;
  bottom: 2px;
  background-color: white;
  border: 2px solid var(--line);
  border-radius: 999px;
  transition: .2s;
}

.toggle-switch.active .toggle-slider {
  background-color: rgba(185,251,192,.8);
}

.toggle-switch.active .toggle-slider:before {
  transform: translateX(24px);
}
</style>
