<script setup lang="ts">
// ============================================================
// 欢迎门户页面
// 用户输入口令后进入主界面
// ============================================================

import { ref } from 'vue';
import { ElMessage } from 'element-plus';
import { useGateStore } from '@/store';

const emit = defineEmits<{
  (e: 'authenticated'): void;
}>();

const gateStore = useGateStore();
const accessCode = ref('');
const isShaking = ref(false);
const errorCount = ref(0);

// 处理口令验证
function handleSubmit() {
  if (!accessCode.value.trim()) {
    ElMessage.warning('请在下方输入框中输入 cknb');
    return;
  }

  const success = gateStore.authenticate(accessCode.value);
  if (success) {
    ElMessage.success('验证通过，欢迎使用！');
    emit('authenticated');
  } else {
    // 口令错误，触发抖动动画
    errorCount.value++;
    isShaking.value = true;
    if (errorCount.value >= 3) {
      ElMessage.error('口令错误！提示：请输入页面上高亮显示的口令 cknb');
    } else {
      ElMessage.error('口令错误，请仔细看页面提示');
    }
    accessCode.value = '';
    setTimeout(() => {
      isShaking.value = false;
    }, 600);
  }
}

// 回车提交
function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Enter') {
    handleSubmit();
  }
}
</script>

<template>
  <div class="gate-container">
    <!-- 背景粒子效果 -->
    <div class="bg-particles">
      <div v-for="i in 30" :key="i" class="particle" :style="{
        left: Math.random() * 100 + '%',
        top: Math.random() * 100 + '%',
        animationDelay: Math.random() * 5 + 's',
        animationDuration: (3 + Math.random() * 4) + 's'
      }" />
    </div>

    <!-- 主内容区域 -->
    <div class="gate-content" :class="{ 'shake': isShaking }">
      <!-- Logo 区域 -->
      <div class="logo-section">
        <div class="logo-icon">
          <svg viewBox="0 0 120 120" xmlns="http://www.w3.org/2000/svg">
            <defs>
              <linearGradient id="logoGrad" x1="0%" y1="0%" x2="100%" y2="100%">
                <stop offset="0%" style="stop-color:#667eea;stop-opacity:1" />
                <stop offset="100%" style="stop-color:#764ba2;stop-opacity:1" />
              </linearGradient>
            </defs>
            <circle cx="60" cy="60" r="55" fill="url(#logoGrad)" opacity="0.15"/>
            <circle cx="60" cy="60" r="40" fill="url(#logoGrad)" opacity="0.3"/>
            <text x="60" y="72" text-anchor="middle" fill="url(#logoGrad)" font-size="36" font-weight="bold" font-family="monospace">CK</text>
          </svg>
        </div>
        <h1 class="app-title">Windsurf Account Manager</h1>
        <p class="app-subtitle">CK 二次开发版 · 多账号管理 & 无感切号</p>
      </div>

      <!-- 醒目口令提示区域 -->
      <div class="code-banner">
        <p class="banner-label">请输入以下口令进入软件</p>
        <div class="code-display">
          <span class="code-char">c</span>
          <span class="code-char">k</span>
          <span class="code-char">n</span>
          <span class="code-char">b</span>
        </div>
        <div class="banner-arrow">
          <svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="2" opacity="0.6">
            <path d="M12 5v14M5 12l7 7 7-7"/>
          </svg>
        </div>
      </div>

      <!-- 输入区域 -->
      <div class="input-section">
        <div class="input-wrapper">
          <input
            v-model="accessCode"
            type="text"
            class="code-input"
            placeholder="在此输入口令 cknb"
            maxlength="20"
            autofocus
            autocomplete="off"
            spellcheck="false"
            @keydown="handleKeydown"
          />
          <button class="submit-btn" @click="handleSubmit">
            <span class="btn-text">进入</span>
            <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2.5">
              <path d="M5 12h14M12 5l7 7-7 7"/>
            </svg>
          </button>
        </div>
        <p v-if="errorCount >= 2" class="error-hint">
          提示：口令就是上方高亮显示的 <strong>cknb</strong>，直接输入即可
        </p>
      </div>

      <!-- 开源信息 -->
      <div class="info-section">
        <div class="info-links">
          <a href="https://github.com/1837620622/windsurf-account-manager" target="_blank" class="info-link">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor">
              <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/>
            </svg>
            GitHub 开源地址
          </a>
          <span class="info-divider">·</span>
          <span class="info-author">传康Kk · 万能程序员</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* ============================================================ */
/* 容器全屏背景 */
/* ============================================================ */
.gate-container {
  width: 100vw;
  height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, #0f0c29 0%, #302b63 50%, #24243e 100%);
  position: relative;
  overflow: hidden;
  user-select: none;
}

/* ============================================================ */
/* 背景粒子动画 */
/* ============================================================ */
.bg-particles {
  position: absolute;
  width: 100%;
  height: 100%;
  pointer-events: none;
}

.particle {
  position: absolute;
  width: 4px;
  height: 4px;
  background: rgba(102, 126, 234, 0.4);
  border-radius: 50%;
  animation: float linear infinite;
}

@keyframes float {
  0% { transform: translateY(0) scale(1); opacity: 0; }
  10% { opacity: 1; }
  90% { opacity: 1; }
  100% { transform: translateY(-100vh) scale(0.5); opacity: 0; }
}

/* ============================================================ */
/* 主内容卡片 */
/* ============================================================ */
.gate-content {
  position: relative;
  z-index: 10;
  text-align: center;
  padding: 40px 50px;
  background: rgba(255, 255, 255, 0.05);
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  border-radius: 24px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  box-shadow: 0 25px 50px rgba(0, 0, 0, 0.3);
  max-width: 500px;
  width: 92%;
  animation: fadeIn 0.8s ease-out;
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(30px); }
  to { opacity: 1; transform: translateY(0); }
}

/* 口令错误抖动效果 */
.shake {
  animation: shake 0.6s ease-in-out;
}

@keyframes shake {
  0%, 100% { transform: translateX(0); }
  10%, 30%, 50%, 70%, 90% { transform: translateX(-10px); }
  20%, 40%, 60%, 80% { transform: translateX(10px); }
}

/* ============================================================ */
/* Logo 区域 */
/* ============================================================ */
.logo-section {
  margin-bottom: 24px;
}

.logo-icon {
  width: 80px;
  height: 80px;
  margin: 0 auto 12px;
}

.logo-icon svg {
  width: 100%;
  height: 100%;
}

.app-title {
  font-size: 21px;
  font-weight: 700;
  color: #e5eaf3;
  letter-spacing: 0.5px;
  margin-bottom: 6px;
}

.app-subtitle {
  font-size: 12px;
  color: #94a3b8;
  letter-spacing: 1px;
}

/* ============================================================ */
/* 醒目口令提示横幅 */
/* ============================================================ */
.code-banner {
  margin: 24px 0 20px;
  padding: 20px;
  background: rgba(102, 126, 234, 0.08);
  border: 1px solid rgba(102, 126, 234, 0.2);
  border-radius: 16px;
  position: relative;
}

.banner-label {
  font-size: 14px;
  color: #94a3b8;
  margin-bottom: 14px;
  letter-spacing: 0.5px;
}

/* 口令字符大号展示 - 带脉冲光晕 */
.code-display {
  display: flex;
  justify-content: center;
  gap: 12px;
  margin-bottom: 10px;
}

.code-char {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 56px;
  height: 64px;
  font-size: 32px;
  font-weight: 800;
  font-family: 'Courier New', Consolas, monospace;
  color: #fff;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border-radius: 12px;
  box-shadow: 0 4px 20px rgba(102, 126, 234, 0.4);
  animation: pulse 2s ease-in-out infinite;
  text-transform: lowercase;
}

.code-char:nth-child(1) { animation-delay: 0s; }
.code-char:nth-child(2) { animation-delay: 0.15s; }
.code-char:nth-child(3) { animation-delay: 0.3s; }
.code-char:nth-child(4) { animation-delay: 0.45s; }

@keyframes pulse {
  0%, 100% {
    box-shadow: 0 4px 20px rgba(102, 126, 234, 0.4);
    transform: scale(1);
  }
  50% {
    box-shadow: 0 4px 30px rgba(102, 126, 234, 0.7), 0 0 40px rgba(118, 75, 162, 0.3);
    transform: scale(1.05);
  }
}

.banner-arrow {
  color: #667eea;
  animation: bounce 1.5s ease-in-out infinite;
}

@keyframes bounce {
  0%, 100% { transform: translateY(0); }
  50% { transform: translateY(6px); }
}

/* ============================================================ */
/* 输入区域 */
/* ============================================================ */
.input-section {
  margin-bottom: 20px;
}

.input-wrapper {
  display: flex;
  align-items: center;
  gap: 10px;
  max-width: 340px;
  margin: 0 auto;
}

.code-input {
  flex: 1;
  height: 50px;
  padding: 0 20px;
  font-size: 18px;
  font-weight: 600;
  letter-spacing: 4px;
  color: #e5eaf3;
  background: rgba(255, 255, 255, 0.08);
  border: 2px solid rgba(102, 126, 234, 0.3);
  border-radius: 14px;
  outline: none;
  transition: all 0.3s ease;
  text-align: center;
}

.code-input::placeholder {
  color: #64748b;
  letter-spacing: 1px;
  font-size: 13px;
  font-weight: 400;
}

.code-input:focus {
  border-color: #667eea;
  box-shadow: 0 0 0 3px rgba(102, 126, 234, 0.25);
  background: rgba(255, 255, 255, 0.12);
}

.submit-btn {
  height: 50px;
  padding: 0 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  background: linear-gradient(135deg, #667eea, #764ba2);
  border: none;
  border-radius: 14px;
  color: #fff;
  cursor: pointer;
  transition: all 0.3s ease;
  flex-shrink: 0;
  font-size: 15px;
  font-weight: 600;
}

.btn-text {
  white-space: nowrap;
}

.submit-btn:hover {
  transform: scale(1.03);
  box-shadow: 0 4px 20px rgba(102, 126, 234, 0.5);
}

.submit-btn:active {
  transform: scale(0.97);
}

/* 错误多次后的额外提示 */
.error-hint {
  margin-top: 12px;
  font-size: 13px;
  color: #f87171;
  animation: fadeIn 0.3s ease;
}

.error-hint strong {
  color: #fbbf24;
  font-size: 14px;
  letter-spacing: 1px;
}

/* ============================================================ */
/* 底部信息 */
/* ============================================================ */
.info-section {
  padding-top: 4px;
}

.info-links {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  font-size: 11px;
  color: #64748b;
}

.info-link {
  display: flex;
  align-items: center;
  gap: 4px;
  color: #94a3b8;
  text-decoration: none;
  transition: color 0.2s;
}

.info-link:hover {
  color: #667eea;
}

.info-divider {
  color: #475569;
}

.info-author {
  color: #64748b;
}
</style>
