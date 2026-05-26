<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { check } from '@tauri-apps/plugin-updater'
import { getVersion } from '@tauri-apps/api/app'
import { BellIcon, SparklesIcon } from './icons'

const showNotification = ref(false)
const hasNewVersion = ref(false)
const newVersion = ref('')
const currentVersion = ref('')

const emit = defineEmits<{
  (e: 'goToAbout'): void
}>()

const checkForUpdate = async () => {
  try {
    const version = await getVersion()
    currentVersion.value = version
    
    const update = await check()
    if (update) {
      hasNewVersion.value = true
      newVersion.value = update.version
      showNotification.value = true
    }
  } catch (e) {
    console.error('[ACE Helper] 检查更新失败:', e)
  }
}

const handleUpdateClick = () => {
  emit('goToAbout')
}

onMounted(() => {
  setTimeout(() => {
    checkForUpdate()
  }, 3000)
})
</script>

<template>
  <Transition name="fade">
    <div v-if="showNotification && hasNewVersion" class="sidebar-update-tag" @click="handleUpdateClick">
      <div class="tag-glow"></div>
      <SparklesIcon :size="12" class="tag-icon" />
      <span class="tag-text">新版本 v{{ newVersion }}</span>
      <BellIcon :size="10" class="tag-bell" />
    </div>
  </Transition>
</template>

<style scoped>
.sidebar-update-tag {
  position: absolute;
  bottom: 16px;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 14px;
  background: linear-gradient(135deg, #f59e0b 0%, #d97706 100%);
  color: white;
  border-radius: 20px;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  box-shadow: 0 4px 12px rgba(245, 158, 11, 0.4);
  z-index: 100;
  transition: all 0.2s ease;
  white-space: nowrap;
}

.sidebar-update-tag:hover {
  transform: translateX(-50%) translateY(-2px);
  box-shadow: 0 6px 20px rgba(245, 158, 11, 0.5);
}

.tag-glow {
  position: absolute;
  inset: -2px;
  background: linear-gradient(135deg, #fbbf24, #f59e0b, #d97706);
  border-radius: 22px;
  z-index: -1;
  opacity: 0.6;
  animation: glow-pulse 2s ease-in-out infinite;
}

@keyframes glow-pulse {
  0%, 100% { 
    opacity: 0.4;
    transform: scale(1);
  }
  50% { 
    opacity: 0.8;
    transform: scale(1.05);
  }
}

.tag-icon {
  animation: sparkle 2s ease-in-out infinite;
}

@keyframes sparkle {
  0%, 100% { transform: scale(1) rotate(0deg); }
  25% { transform: scale(1.2) rotate(10deg); }
  50% { transform: scale(1) rotate(0deg); }
  75% { transform: scale(1.2) rotate(-10deg); }
}

.tag-text {
  position: relative;
  z-index: 1;
}

.tag-bell {
  animation: bell-ring 2s ease-in-out infinite;
  opacity: 0.9;
}

@keyframes bell-ring {
  0%, 100% { transform: rotate(0deg); }
  10%, 30%, 50%, 70%, 90% { transform: rotate(15deg); }
  20%, 40%, 60%, 80% { transform: rotate(-15deg); }
}

/* 动画效果 */
.fade-enter-active,
.fade-leave-active {
  transition: all 0.3s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
  transform: translateX(-50%) translateY(10px);
}

.fade-enter-to,
.fade-leave-from {
  opacity: 1;
  transform: translateX(-50%) translateY(0);
}
</style>
