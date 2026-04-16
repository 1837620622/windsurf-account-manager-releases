<script setup lang="ts">
// ============================================================
// 欢迎门户页面
// 用户输入口令后进入主界面
// ============================================================

import { ref, computed } from 'vue';
import { ElMessage } from 'element-plus';
import { useGateStore } from '@/store';

// ============================================================
// 预生成粒子位置数据，避免在模板中使用 Math.random()
// 防止响应式数据变化时粒子位置跳变
// ============================================================
const particles = Array.from({ length: 30 }, () => ({
  left: Math.random() * 100 + '%',
  top: Math.random() * 100 + '%',
  animationDelay: Math.random() * 5 + 's',
  animationDuration: (3 + Math.random() * 4) + 's'
}));

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
      <div v-for="(p, i) in particles" :key="i" class="particle" :style="p" />
    </div>

    <!-- 主内容区域 -->
    <div class="gate-content" :class="{ 'shake': isShaking }">
      <!-- Logo 区域 -->
      <div class="logo-section">
        <div class="logo-icon">
          <svg viewBox="0 0 120 120" xmlns="http://www.w3.org/2000/svg">
            <defs>
              <linearGradient id="logoGrad" x1="0%" y1="0%" x2="100%" y2="100%">
                <stop offset="0%" style="stop-color:#0ff;stop-opacity:1" />
                <stop offset="100%" style="stop-color:#0ea5e9;stop-opacity:1" />
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

      <!-- 验证交互区域 -->
      <div class="verification-card">
        <p class="banner-label">请输入以下口令进入软件</p>
        
        <div class="code-display">
          <span class="code-char">c</span>
          <span class="code-char">k</span>
          <span class="code-char">n</span>
          <span class="code-char">b</span>
        </div>

        <div class="banner-arrow">
          <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="2" opacity="0.6">
            <path d="M12 5v14M5 12l7 7 7-7"/>
          </svg>
        </div>

        <!-- 输入区域 -->
        <div class="input-wrapper">
          <input
            v-model="accessCode"
            type="text"
            class="code-input"
            placeholder="输入口令 cknb"
            maxlength="20"
            autofocus
            autocomplete="off"
            spellcheck="false"
            @keydown="handleKeydown"
          />
          <button class="submit-btn" @click="handleSubmit">
            <span class="btn-text">Enter ↵</span>
          </button>
        </div>
        
        <p v-if="errorCount >= 2" class="error-hint">
          提示：口令就是上方高亮显示的 <strong>cknb</strong>，直接输入即可
        </p>
      </div>

      <!-- 开源信息 -->
      <div class="info-section">
        <div class="info-links">
          <a href="https://github.com/1837620622/windsurf-account-manager" target="_blank" rel="noopener noreferrer" class="info-link">
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
/* 容器全屏背景 - 科技网格效果 */
/* ============================================================ */
.gate-container {
  width: 100vw;
  height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: #050a14;
  background-image: 
    linear-gradient(rgba(0, 255, 255, 0.03) 1px, transparent 1px),
    linear-gradient(90deg, rgba(0, 255, 255, 0.03) 1px, transparent 1px);
  background-size: 40px 40px;
  background-position: center center;
  position: relative;
  overflow: hidden;
  user-select: none;
}

/* 核心光晕 */
.gate-container::before {
  content: '';
  position: absolute;
  width: 800px;
  height: 800px;
  background: radial-gradient(circle, rgba(0, 255, 255, 0.08) 0%, transparent 60%);
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  pointer-events: none;
  animation: breathe 4s ease-in-out infinite alternate;
}

@keyframes breathe {
  from { opacity: 0.6; transform: translate(-50%, -50%) scale(0.9); }
  to { opacity: 1; transform: translate(-50%, -50%) scale(1.1); }
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
  width: 3px;
  height: 3px;
  background: rgba(0, 255, 255, 0.6);
  border-radius: 50%;
  box-shadow: 0 0 10px rgba(0, 255, 255, 0.8);
  animation: float linear infinite;
}

@keyframes float {
  0% { transform: translateY(0) scale(1); opacity: 0; }
  10% { opacity: 1; }
  90% { opacity: 1; }
  100% { transform: translateY(-100vh) scale(0.5); opacity: 0; }
}

/* ============================================================ */
/* 主内容卡片 - 磨砂玻璃科技面板 */
/* ============================================================ */
.gate-content {
  position: relative;
  z-index: 10;
  text-align: center;
  padding: 45px 50px;
  background: rgba(10, 16, 30, 0.7);
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  border-radius: 20px;
  border: 1px solid rgba(0, 255, 255, 0.15);
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.6), inset 0 0 30px rgba(0, 255, 255, 0.05);
  max-width: 480px;
  width: 92%;
  animation: fadeIn 0.8s cubic-bezier(0.16, 1, 0.3, 1);
}

/* 卡片边角科技装饰 */
.gate-content::before, .gate-content::after {
  content: '';
  position: absolute;
  width: 20px;
  height: 20px;
  border: 2px solid rgba(0, 255, 255, 0.5);
  pointer-events: none;
}

.gate-content::before {
  top: -1px; left: -1px;
  border-right: none; border-bottom: none;
  border-top-left-radius: 20px;
}

.gate-content::after {
  bottom: -1px; right: -1px;
  border-left: none; border-top: none;
  border-bottom-right-radius: 20px;
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(30px) scale(0.95); }
  to { opacity: 1; transform: translateY(0) scale(1); }
}

/* 口令错误抖动效果 */
.shake {
  animation: shake 0.5s cubic-bezier(.36,.07,.19,.97) both;
  border-color: rgba(255, 0, 85, 0.5);
  box-shadow: 0 0 30px rgba(255, 0, 85, 0.2);
}

@keyframes shake {
  10%, 90% { transform: translate3d(-2px, 0, 0); }
  20%, 80% { transform: translate3d(4px, 0, 0); }
  30%, 50%, 70% { transform: translate3d(-8px, 0, 0); }
  40%, 60% { transform: translate3d(8px, 0, 0); }
}

/* ============================================================ */
/* Logo 区域 */
/* ============================================================ */
.logo-section {
  margin-bottom: 24px;
}

.logo-icon {
  width: 76px;
  height: 76px;
  margin: 0 auto 16px;
  filter: drop-shadow(0 0 10px rgba(0, 255, 255, 0.4));
}

.logo-icon svg {
  width: 100%;
  height: 100%;
}

.app-title {
  font-size: 22px;
  font-weight: 800;
  color: #fff;
  letter-spacing: 1px;
  margin-bottom: 8px;
  text-shadow: 0 0 10px rgba(0, 255, 255, 0.4);
}

.app-subtitle {
  font-size: 12px;
  color: #64d1ff;
  opacity: 0.8;
  letter-spacing: 2px;
  text-transform: uppercase;
}

/* ============================================================ */
/* 验证交互区域 (Verification Card) */
/* ============================================================ */
.verification-card {
  margin: 30px 0;
  padding: 30px 20px;
  background: rgba(0, 0, 0, 0.4);
  border: 1px solid rgba(0, 255, 255, 0.1);
  border-radius: 16px;
  position: relative;
  overflow: hidden;
  box-shadow: inset 0 0 20px rgba(0, 0, 0, 0.8);
}

/* 扫描线效果 */
.verification-card::before {
  content: '';
  position: absolute;
  top: 0; left: 0; right: 0;
  height: 100%;
  background: linear-gradient(to bottom, transparent, rgba(0, 255, 255, 0.05), transparent);
  animation: scan 3s linear infinite;
  pointer-events: none;
  border-radius: 16px;
}

@keyframes scan {
  0% { transform: translateY(-100%); }
  100% { transform: translateY(100%); }
}

.banner-label {
  font-size: 13px;
  color: #8bb4e7;
  margin-bottom: 16px;
  letter-spacing: 1px;
  text-transform: uppercase;
}

/* ============================================================ */
/* 口令字符大号展示 - 霓虹键盘效果 */
/* ============================================================ */
.code-display {
  display: flex;
  justify-content: center;
  gap: 14px;
  margin-bottom: 16px;
}

.code-char {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 50px;
  height: 56px;
  font-size: 28px;
  font-weight: 700;
  font-family: 'Courier New', Courier, monospace;
  color: #0ff;
  background: rgba(0, 255, 255, 0.05);
  border: 1px solid rgba(0, 255, 255, 0.3);
  border-radius: 8px;
  box-shadow: 0 0 15px rgba(0, 255, 255, 0.2), inset 0 0 10px rgba(0, 255, 255, 0.1);
  text-shadow: 0 0 8px rgba(0, 255, 255, 0.8);
  animation: neonPulse 2s infinite alternate;
  text-transform: lowercase;
}

.code-char:nth-child(1) { animation-delay: 0.1s; }
.code-char:nth-child(2) { animation-delay: 0.3s; }
.code-char:nth-child(3) { animation-delay: 0.5s; }
.code-char:nth-child(4) { animation-delay: 0.7s; }

@keyframes neonPulse {
  from {
    box-shadow: 0 0 10px rgba(0, 255, 255, 0.1), inset 0 0 5px rgba(0, 255, 255, 0.05);
    border-color: rgba(0, 255, 255, 0.2);
  }
  to {
    box-shadow: 0 0 20px rgba(0, 255, 255, 0.4), inset 0 0 15px rgba(0, 255, 255, 0.2);
    border-color: rgba(0, 255, 255, 0.6);
  }
}

.banner-arrow {
  color: rgba(0, 255, 255, 0.5);
  margin-bottom: 16px;
  animation: arrowBounce 1.5s ease-in-out infinite;
}

@keyframes arrowBounce {
  0%, 100% { transform: translateY(0); opacity: 0.3; }
  50% { transform: translateY(5px); opacity: 0.8; }
}

/* ============================================================ */
/* 输入区域 - 科技感输入条 */
/* ============================================================ */
.input-wrapper {
  display: flex;
  align-items: center;
  max-width: 320px;
  margin: 0 auto;
  height: 52px;
  background: rgba(0, 15, 30, 0.6);
  border: 1px solid rgba(0, 255, 255, 0.3);
  border-radius: 26px; /* 药丸形状 */
  padding: 4px;
  box-shadow: 0 0 15px rgba(0, 255, 255, 0.1);
  transition: all 0.3s ease;
  position: relative;
  z-index: 2;
}

.input-wrapper:focus-within {
  border-color: #0ff;
  box-shadow: 0 0 20px rgba(0, 255, 255, 0.3), inset 0 0 10px rgba(0, 255, 255, 0.1);
}

.code-input {
  flex: 1;
  height: 100%;
  background: transparent;
  border: none;
  padding: 0 16px 0 20px;
  font-size: 16px;
  font-weight: 600;
  font-family: 'Courier New', Courier, monospace;
  letter-spacing: 3px;
  color: #0ff;
  text-shadow: 0 0 5px rgba(0, 255, 255, 0.5);
  outline: none;
  text-align: center;
}

.code-input::placeholder {
  color: rgba(0, 255, 255, 0.3);
  letter-spacing: 1px;
  font-size: 13px;
  font-family: sans-serif;
  text-shadow: none;
}

.submit-btn {
  height: 100%;
  padding: 0 22px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 255, 255, 0.12);
  border: 1px solid rgba(0, 255, 255, 0.4);
  border-radius: 22px;
  color: #0ff;
  cursor: pointer;
  transition: all 0.3s ease;
  font-size: 13px;
  font-weight: bold;
  letter-spacing: 2px;
  font-family: 'Courier New', Courier, monospace;
  white-space: nowrap;
}

.submit-btn:hover {
  background: rgba(0, 255, 255, 0.25);
  box-shadow: 0 0 15px rgba(0, 255, 255, 0.5);
  transform: translateY(-1px);
}

.submit-btn:active {
  transform: translateY(1px);
  background: rgba(0, 255, 255, 0.15);
}

.btn-text {
  white-space: nowrap;
}

.error-hint {
  margin-top: 16px;
  font-size: 12px;
  color: #ff3366;
  text-shadow: 0 0 5px rgba(255, 51, 102, 0.4);
  animation: fadeIn 0.3s ease;
}

.error-hint strong {
  color: #0ff;
  font-family: monospace;
  font-size: 14px;
  text-shadow: 0 0 8px #0ff;
}

/* ============================================================ */
/* 底部信息 */
/* ============================================================ */
.info-section {
  padding-top: 10px;
}

.info-links {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
  font-size: 12px;
  color: rgba(255, 255, 255, 0.4);
}

.info-link {
  display: flex;
  align-items: center;
  gap: 6px;
  color: rgba(0, 255, 255, 0.6);
  text-decoration: none;
  transition: all 0.3s ease;
}

.info-link:hover {
  color: #0ff;
  text-shadow: 0 0 8px rgba(0, 255, 255, 0.6);
}

.info-divider {
  color: rgba(255, 255, 255, 0.2);
}

.info-author {
  color: rgba(255, 255, 255, 0.5);
}

/* ============================================================ */
/* 响应式适配 */
/* ============================================================ */
@media (max-width: 520px) {
  .gate-content {
    padding: 30px 24px;
    border-radius: 16px;
  }
  .gate-content::before { border-top-left-radius: 16px; }
  .gate-content::after { border-bottom-right-radius: 16px; }
  .app-title { font-size: 18px; }
  .code-char { width: 42px; height: 48px; font-size: 24px; }
  .code-display { gap: 10px; }
  .input-wrapper { max-width: 100%; height: 48px; }
  .submit-btn { padding: 0 18px; font-size: 13px; }
  .verification-card { padding: 24px 16px; }
}
</style>
