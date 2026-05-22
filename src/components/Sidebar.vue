<script setup lang="ts">
defineProps<{
  activeTab: string
}>()

const emit = defineEmits<{
  (e: 'select', id: string): void
}>()

const tabs = [
  { id: 'home', icon: '🏠', label: '首页', sub: 'NOW' },
  { id: 'settings', icon: '🧰', label: '优化设置', sub: 'Soon' },
  { id: 'appsettings', icon: '⚙️', label: '软件设置', sub: 'Soon' },
  { id: 'about', icon: '❓', label: '关于帮助', sub: 'Soon' }
]
</script>

<template>
  <aside class="sidebar">
    <div class="brand">
      <div class="logo">
        <img src="/src/assets/app-icon.png" alt="应用图标" />
      </div>
      <div class="title">
        <b>ACE 小助手</b>
        <small>蜡笔涂鸦 · 可爱但混乱</small>
      </div>
    </div>

    <nav class="tabs" aria-label="菜单">
      <div
        v-for="tab in tabs"
        :key="tab.id"
        class="tab"
        :class="{ active: activeTab === tab.id }"
        :style="{ '--rot': tab.id === 'home' ? '-1.6deg' : tab.id === 'settings' ? '1.2deg' : tab.id === 'appsettings' ? '-0.8deg' : '0.6deg' }"
        @click="emit('select', tab.id)"
      >
        <div class="ico">{{ tab.icon }}</div>
        <div class="txt">{{ tab.label }}</div>
        <div class="sub">{{ activeTab === tab.id ? 'NOW' : 'Soon' }}</div>
      </div>
    </nav>

    <div class="doodles" aria-hidden="true">
      <div class="doodle three">✦✦</div>
      <div class="doodle one">♥ ☆ ☁</div>
      <div class="doodle two">🎮 claw!!</div>
    </div>
    <div class="char-decorations" aria-hidden="true">
      <img class="char-img char-pink" src="/src/assets/char-pink.png" alt="" />
    </div>
  </aside>
</template>