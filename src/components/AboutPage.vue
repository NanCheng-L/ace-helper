<script setup lang="ts">
import { ref } from 'vue'
import { openUrl } from '@tauri-apps/plugin-opener'
import { check, type Update } from '@tauri-apps/plugin-updater'
import { getVersion } from '@tauri-apps/api/app'

const socialLinks = [
  { iconType: 'github', label: 'GitHub', url: 'https://github.com/NanCheng-L/ace-helper', desc: '查看源码，欢迎 Star' },
  { iconType: 'bilibili', label: 'B站', url: 'https://space.bilibili.com/yourid', desc: '视频教程与更新动态' },
  { iconType: 'douyin', label: '抖音', url: 'https://www.douyin.com/user/yourid', desc: '短视频日常分享' }
]

const appVersion = ref('0.1.0')
const updateStatus = ref<'idle' | 'checking' | 'available' | 'no-update' | 'error' | 'downloading'>('idle')
const updateMessage = ref('')
const downloadProgress = ref(0)
const availableUpdate = ref<Update | null>(null)

const openLink = (url: string) => {
  openUrl(url)
}

const checkUpdate = async () => {
  updateStatus.value = 'checking'
  updateMessage.value = '正在检查更新…'
  downloadProgress.value = 0
  availableUpdate.value = null

  try {
    const update = await check()
    if (update) {
      availableUpdate.value = update
      updateStatus.value = 'available'
      updateMessage.value = `发现新版本 ${update.version}，当前版本 ${update.currentVersion}`
    } else {
      updateStatus.value = 'no-update'
      updateMessage.value = '已是最新版本，棒棒哒 ✦'
    }
  } catch (e) {
    updateStatus.value = 'error'
    updateMessage.value = '检查更新失败，请稍后再试 (´•̥ ̯ •̥`)'
    console.error('[ACE Helper] 检查更新失败:', e)
  }
}

const doUpdate = async () => {
  if (updateStatus.value !== 'available' || !availableUpdate.value) return
  updateStatus.value = 'downloading'
  updateMessage.value = '正在下载更新…'

  try {
    const update = availableUpdate.value

    let downloaded = 0
    let contentLength = 0
    await update.download((event) => {
      switch (event.event) {
        case 'Started':
          contentLength = event.data.contentLength ?? 0
          break
        case 'Progress':
          downloaded += event.data.chunkLength
          if (contentLength > 0) {
            downloadProgress.value = Math.round((downloaded / contentLength) * 100)
          }
          break
        case 'Finished':
          break
      }
    })

    updateMessage.value = '下载完成，正在安装更新…'
    await update.install()
  } catch (e) {
    updateStatus.value = 'error'
    updateMessage.value = '下载更新失败 (´•̥ ̯ •̥`)'
    console.error('[ACE Helper] 下载更新失败:', e)
  }
}

getVersion().then(v => {
  appVersion.value = v
}).catch(() => {})
</script>

<template>
  <div class="about-page">
    <div class="about-header">
      <div class="header-icon">❓</div>
      <div class="header-text">
        <h2>关于与帮助</h2>
        <p>有问题或建议？来找我吧！</p>
      </div>
    </div>

    <!-- 关于软件 -->
    <section class="about-section main-about">
      <div class="app-intro">
        <img src="/src/assets/app-icon.png" alt="ACE 小助手" class="app-icon" />
        <div class="app-info">
          <h3>ACE 小助手</h3>
          <p class="version">版本 {{ appVersion }}</p>
          <p class="desc">可爱又实用的 ACE 进程优化小工具</p>
          <p class="tagline">用蜡笔涂鸦风格，帮你管理 ACE 进程 ✦</p>
        </div>
      </div>
    </section>

    <!-- 社交链接 -->
    <section class="about-section">
      <div class="section-title">
        <span class="icon">🔗</span>
        <span>找到我</span>
      </div>
      <p class="section-desc">点击下面的链接找到我</p>
      
      <div class="link-list">
        <div
          v-for="link in socialLinks"
          :key="link.label"
          class="link-item"
          @click="openLink(link.url)"
        >
          <span class="link-icon">
            <svg v-if="link.iconType === 'github'" class="icon-svg" viewBox="0 0 16 16" width="24" height="24" fill="currentColor">
              <path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0016 8c0-4.42-3.58-8-8-8z"/>
            </svg>
            <svg v-else-if="link.iconType === 'bilibili'" class="icon-svg bilibili-icon" viewBox="0 0 18 18" width="24" height="24" fill="currentColor">
              <path fill-rule="evenodd" clip-rule="evenodd" d="M3.73252 2.67094C3.33229 2.28484 3.33229 1.64373 3.73252 1.25764C4.11291 0.890684 4.71552 0.890684 5.09591 1.25764L7.21723 3.30403C7.27749 3.36218 7.32869 3.4261 7.37081 3.49407H10.5789C10.6211 3.4261 10.6723 3.36218 10.7325 3.30403L12.8538 1.25764C13.2342 0.890684 13.8368 0.890684 14.2172 1.25764C14.6175 1.64373 14.6175 2.28484 14.2172 2.67094L13.364 3.49407H14C16.2091 3.49407 18 5.28493 18 7.49407V12.9996C18 15.2087 16.2091 16.9996 14 16.9996H4C1.79086 16.9996 0 15.2087 0 12.9996V7.49406C0 5.28492 1.79086 3.49407 4 3.49407H4.58579L3.73252 2.67094ZM4 5.42343C2.89543 5.42343 2 6.31886 2 7.42343V13.0702C2 14.1748 2.89543 15.0702 4 15.0702H14C15.1046 15.0702 16 14.1748 16 13.0702V7.42343C16 6.31886 15.1046 5.42343 14 5.42343H4ZM5 9.31747C5 8.76519 5.44772 8.31747 6 8.31747C6.55228 8.31747 7 8.76519 7 9.31747V10.2115C7 10.7638 6.55228 11.2115 6 11.2115C5.44772 11.2115 5 10.7638 5 10.2115V9.31747ZM12 8.31747C11.4477 8.31747 11 8.76519 11 9.31747V10.2115C11 10.7638 11.4477 11.2115 12 11.2115C12.5523 11.2115 13 10.7638 13 10.2115V9.31747C13 8.76519 12.5523 8.31747 12 8.31747Z"/>
            </svg>
            <svg v-else-if="link.iconType === 'douyin'" class="icon-svg douyin-icon" viewBox="0 0 24 24" width="24" height="24" fill="currentColor">
              <path d="M19.59 6.69a4.83 4.83 0 01-3.77-4.25V2h-3.45v13.89a2.89 2.89 0 01-2.88 2.59 2.89 2.89 0 01-2.89-2.89 2.89 2.89 0 012.89-2.89c.3 0 .59.05.86.13V9.36a6.34 6.34 0 00-.86-.06 6.35 6.35 0 100 12.7 6.35 6.35 0 006.34-6.35V8.33a8.27 8.27 0 004.76 1.52V6.45a4.97 4.97 0 01-.73-.05v.29z"/>
            </svg>
          </span>
          <div class="link-info">
            <span class="link-label">{{ link.label }}</span>
            <span class="link-desc">{{ link.desc }}</span>
          </div>
          <span class="link-arrow">→</span>
        </div>
      </div>
    </section>

    <!-- 检查更新 -->
    <section class="about-section update-section">
      <div class="section-title">
        <span class="icon">🔄</span>
        <span>检查更新</span>
      </div>
      <p class="section-desc">保持软件最新，获得更好的体验</p>
      
      <div class="update-area">
        <div class="update-message" :class="updateStatus">
          <span v-if="updateStatus === 'checking'">⏳</span>
          <span v-else-if="updateStatus === 'available'">🎉</span>
          <span v-else-if="updateStatus === 'no-update'">✅</span>
          <span v-else-if="updateStatus === 'error'">😿</span>
          <span v-else-if="updateStatus === 'downloading'">📥</span>
          <span v-else>📦</span>
          {{ updateMessage }}
        </div>
        <div v-if="updateStatus === 'downloading'" class="progress-bar-wrap">
          <div class="progress-bar" :style="{ width: downloadProgress + '%' }"></div>
          <span class="progress-text">{{ downloadProgress }}%</span>
        </div>
        <div class="update-buttons">
          <button
            v-if="updateStatus === 'available'"
            class="update-btn"
            @click="doUpdate"
          >
            📥 立即更新
          </button>
          <button
            v-if="updateStatus !== 'downloading'"
            class="update-btn check-btn"
            :disabled="updateStatus === 'checking'"
            @click="checkUpdate"
          >
            {{ updateStatus === 'checking' ? '⏳ 检查中…' : '🔍 检查更新' }}
          </button>
        </div>
      </div>
    </section>

    <!-- 使用帮助 -->
    <section class="about-section">
      <div class="section-title">
        <span class="icon">💡</span>
        <span>使用帮助</span>
      </div>
      
      <div class="help-list">
        <div class="help-item">
          <div class="help-q">Q: 这个软件有什么用？</div>
          <div class="help-a">A: ACE 小助手可以自动检测和优化 ACE 相关进程的优先级和 CPU 亲和性，让它们更听话~</div>
        </div>
        <div class="help-item">
          <div class="help-q">Q: 开机自动启动怎么设置？</div>
          <div class="help-a">A: 点击侧边栏的「软件设置」，打开「开机自动启动」开关即可~</div>
        </div>
        <div class="help-item">
          <div class="help-q">Q: 优化后进程还是没反应？</div>
          <div class="help-a">A: 试试用管理员权限运行软件，部分进程需要更高权限才能修改哦。</div>
        </div>
      </div>
    </section>

    <!-- 底部信息 -->
    <div class="about-footer">
      <p>Made with 💖 and lots of ☕</p>
      <p class="copyright">© 2026 ACE 小助手 · 蜡笔涂鸦风格</p>
    </div>
  </div>
</template>

<style scoped>
.about-page {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.about-header {
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

.about-section {
  border: 3px solid var(--line);
  border-radius: 20px 16px 22px 14px;
  background: rgba(255,255,255,.65);
  padding: 14px 16px;
  box-shadow: var(--shadow);
}

.main-about {
  background: linear-gradient(135deg, rgba(255,230,109,.3), rgba(255,255,255,.7));
}

.app-intro {
  display: flex;
  align-items: center;
  gap: 16px;
}

.app-icon {
  width: 80px;
  height: 80px;
  border: 4px solid var(--line);
  border-radius: 20px;
  background: rgba(255,255,255,.8);
  box-shadow: 0 6px 0 rgba(0,0,0,.08);
}

.app-info h3 {
  margin: 0;
  font-size: 22px;
  font-weight: 900;
}

.app-info .version {
  margin: 4px 0;
  font-size: 12px;
  color: var(--muted);
  font-weight: 700;
}

.app-info .desc {
  margin: 0;
  font-size: 14px;
  font-weight: 700;
}

.app-info .tagline {
  margin: 8px 0 0;
  font-size: 13px;
  color: var(--ink);
  font-weight: 700;
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

.link-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.link-item {
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

.link-item:hover {
  transform: translateX(4px);
  background: rgba(255,255,255,.9);
}

.link-item:active {
  transform: translateX(2px);
}

.link-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
}

.icon-svg {
  display: block;
}

.bilibili-icon {
  filter: drop-shadow(0 1px 2px rgba(251,114,153,.25));
}

.douyin-icon {
  filter: drop-shadow(0 1px 2px rgba(0,0,0,.15));
}

.link-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.link-label {
  font-weight: 900;
  font-size: 15px;
}

.link-desc {
  font-size: 12px;
  color: var(--muted);
  font-weight: 700;
}

.link-arrow {
  font-size: 20px;
  font-weight: 900;
  color: var(--muted);
}

/* 检查更新 */
.update-section {
  background: linear-gradient(135deg, rgba(200,220,255,.3), rgba(255,255,255,.7));
}

.update-area {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.update-message {
  padding: 10px 14px;
  border: 2px dashed var(--line);
  border-radius: 12px;
  font-weight: 700;
  font-size: 13px;
  background: rgba(255,255,255,.4);
}

.update-message.checking { border-color: rgba(255,200,100,.5); }
.update-message.available { border-color: rgba(160,230,130,.6); background: rgba(185,251,192,.15); }
.update-message.downloading { border-color: rgba(130,180,255,.6); background: rgba(180,210,255,.15); }
.update-message.no-update { border-color: rgba(160,230,130,.4); }
.update-message.error { border-color: rgba(255,150,150,.5); background: rgba(255,200,200,.12); }

.progress-bar-wrap {
  position: relative;
  height: 22px;
  border: 3px solid var(--line);
  border-radius: 999px;
  background: rgba(255,255,255,.5);
  overflow: hidden;
}

.progress-bar {
  height: 100%;
  background: linear-gradient(90deg, rgba(185,251,192,.8), rgba(130,200,255,.8));
  border-radius: 999px;
  transition: width .3s ease;
}

.progress-text {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 900;
  font-size: 11px;
}

.update-buttons {
  display: flex;
  gap: 10px;
}

.update-btn {
  border: 3px solid var(--line);
  border-radius: 14px 12px 16px 14px;
  padding: 10px 20px;
  background: rgba(185,251,192,.8);
  box-shadow: 0 6px 0 rgba(0,0,0,.06);
  font-weight: 900;
  font-size: 14px;
  cursor: pointer;
  transition: transform .12s ease, box-shadow .12s ease;
  user-select: none;
}

.update-btn:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 0 rgba(0,0,0,.07);
}

.update-btn:active {
  transform: translateY(2px);
  box-shadow: 0 4px 0 rgba(0,0,0,.06);
}

.update-btn:disabled {
  opacity: .6;
  cursor: not-allowed;
  transform: none;
}

.check-btn {
  background: rgba(200,220,255,.8);
}

.help-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.help-item {
  padding: 10px 12px;
  border: 2px dashed rgba(43,43,43,.3);
  border-radius: 12px;
  background: rgba(255,255,255,.4);
}

.help-q {
  font-weight: 900;
  font-size: 13px;
  margin-bottom: 6px;
  color: var(--ink);
}

.help-a {
  font-size: 12px;
  color: var(--muted);
  font-weight: 700;
  line-height: 1.5;
}

.about-footer {
  text-align: center;
  padding: 16px;
  color: var(--muted);
  font-weight: 700;
  font-size: 13px;
}

.about-footer .copyright {
  font-size: 11px;
  margin-top: 6px;
  opacity: 0.7;
}
</style>
