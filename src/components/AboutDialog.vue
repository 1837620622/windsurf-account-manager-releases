<template>
  <el-dialog
    v-model="visible"
    title=""
    width="580px"
    class="about-dialog"
    @closed="handleClosed"
  >
    <div class="about-content">
      <!-- 背景装饰 -->
      <div class="bg-decoration">
        <div class="circle circle-1"></div>
        <div class="circle circle-2"></div>
        <div class="circle circle-3"></div>
      </div>
      
      <div class="logo-section">
        <div class="app-logo-wrapper">
          <div class="app-logo">
            <svg width="80" height="80" viewBox="0 0 100 100" class="logo-svg">
              <defs>
                <linearGradient id="logoGradient" x1="0%" y1="0%" x2="100%" y2="100%">
                  <stop offset="0%" style="stop-color:#667eea;stop-opacity:1" />
                  <stop offset="100%" style="stop-color:#764ba2;stop-opacity:1" />
                </linearGradient>
              </defs>
              <rect x="15" y="15" width="30" height="30" rx="8" fill="url(#logoGradient)" opacity="0.9" />
              <rect x="55" y="15" width="30" height="30" rx="8" fill="url(#logoGradient)" opacity="0.7" />
              <rect x="15" y="55" width="30" height="30" rx="8" fill="url(#logoGradient)" opacity="0.7" />
              <rect x="55" y="55" width="30" height="30" rx="8" fill="url(#logoGradient)" opacity="0.5" />
            </svg>
          </div>
          <div class="logo-glow"></div>
        </div>
        <h2 class="app-name">
          <span class="gradient-text">Windsurf </span>
          <span>Account Manager</span>
        </h2>
        <div class="version-badge">
          <span>v{{ appVersion }}</span>
        </div>
      </div>

      <div class="status-cards">
        <div class="status-card">
          <div class="card-icon">
            <el-icon :size="24">
              <Monitor />
            </el-icon>
          </div>
          <div class="card-info">
            <span class="card-label">Windsurf 版本</span>
            <span class="card-value">{{ windsurfVersion || '未检测到' }}</span>
          </div>
        </div>
        
        <div class="status-card">
          <div class="card-icon active">
            <el-icon :size="24">
              <UserFilled />
            </el-icon>
          </div>
          <div class="card-info">
            <span class="card-label">当前账号</span>
            <span class="card-value">{{ currentEmail || '未登录' }}</span>
          </div>
        </div>
      </div>

      <div class="tech-section">
        <div class="tech-item">
          <div class="tech-icon" style="background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);">
            <el-icon :size="20"><User /></el-icon>
          </div>
          <span>CK二次开发</span>
        </div>
        <div class="tech-divider"></div>
        <div class="tech-item">
          <div class="tech-icon" style="background: linear-gradient(135deg, #f093fb 0%, #f5576c 100%);">
            <el-icon :size="20"><Platform /></el-icon>
          </div>
          <span>Vue 3 + Tauri</span>
        </div>
        <div class="tech-divider"></div>
        <div class="tech-item">
          <div class="tech-icon" style="background: linear-gradient(135deg, #4facfe 0%, #00f2fe 100%);">
            <el-icon :size="20"><Promotion /></el-icon>
          </div>
          <span>Rust</span>
        </div>
      </div>

      <div class="features-section">
        <h3 class="section-title">
          <div class="title-icon-wrapper">
            <el-icon><Star /></el-icon>
          </div>
          功能特性
        </h3>
        <div class="feature-grid">
          <div class="feature-card">
            <div class="feature-icon-wrapper">
              <el-icon><Document /></el-icon>
            </div>
            <span class="feature-text">导出账号</span>
          </div>
          <div class="feature-card">
            <div class="feature-icon-wrapper">
              <el-icon><DataAnalysis /></el-icon>
            </div>
            <span class="feature-text">统计信息</span>
          </div>
          <div class="feature-card">
            <div class="feature-icon-wrapper">
              <el-icon><Refresh /></el-icon>
            </div>
            <span class="feature-text">刷新状态</span>
          </div>
          <div class="feature-card">
            <div class="feature-icon-wrapper">
              <el-icon><User /></el-icon>
            </div>
            <span class="feature-text">多账号管理</span>
          </div>
          <div class="feature-card">
            <div class="feature-icon-wrapper">
              <el-icon><Switch /></el-icon>
            </div>
            <span class="feature-text">切换账号</span>
          </div>
          <div class="feature-card">
            <div class="feature-icon-wrapper">
              <el-icon><Delete /></el-icon>
            </div>
            <span class="feature-text">删除账号</span>
          </div>
          <div class="feature-card">
            <div class="feature-icon-wrapper">
              <el-icon><Edit /></el-icon>
            </div>
            <span class="feature-text">编辑账号</span>
          </div>
          <div class="feature-card">
            <div class="feature-icon-wrapper">
              <el-icon><Upload /></el-icon>
            </div>
            <span class="feature-text">批量导入</span>
          </div>
          <div class="feature-card">
            <div class="feature-icon-wrapper">
              <el-icon><List /></el-icon>
            </div>
            <span class="feature-text">操作日志</span>
          </div>
          <div class="feature-card">
            <div class="feature-icon-wrapper">
              <el-icon><Setting /></el-icon>
            </div>
            <span class="feature-text">设置</span>
          </div>
          <div class="feature-card">
            <div class="feature-icon-wrapper">
              <el-icon><Link /></el-icon>
            </div>
            <span class="feature-text">获取链接</span>
          </div>
          <div class="feature-card">
            <div class="feature-icon-wrapper">
              <el-icon><Lightning /></el-icon>
            </div>
            <span class="feature-text">无感换号</span>
          </div>
        </div>
      </div>

      <div class="footer-section">
        <div class="footer-content">
          <p class="copyright">
            Made with <span class="heart">❤️</span> by <span class="author-name">传康Kk</span> (基于 chaogei666 二次开发)
          </p>
          <p class="open-source-link">
            <a href="https://github.com/1837620622/windsurf-account-manager" target="_blank" class="github-link">
              GitHub 开源地址
            </a>
          </p>
          <p class="contact-info">微信: 1837620622 | 咸鱼/B站: 万能程序员</p>
          <p class="year">© 2025 All rights reserved</p>
        </div>
        <p class="disclaimer">
          本软件仅供学习交流使用，请勿用于商业用途
        </p>
      </div>
    </div>
  </el-dialog>
</template>

<script setup lang="ts">
import { computed, ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { 
  Monitor, 
  UserFilled, 
  User,
  Platform,
  Promotion,
  Document,
  DataAnalysis,
  Refresh,
  Switch,
  Delete,
  Edit,
  Upload,
  List,
  Setting,
  Link,
  Lightning,
  Star
} from '@element-plus/icons-vue';

const props = defineProps<{
  modelValue: boolean;
  currentEmail?: string;
  windsurfVersion?: string;
}>();

const emit = defineEmits<{
  'update:modelValue': [value: boolean];
}>();

const visible = computed({
  get: () => props.modelValue,
  set: (val) => emit('update:modelValue', val)
});

// 应用版本号
const appVersion = ref<string>('');

// 获取版本号
onMounted(async () => {
  try {
    const versionInfo = await invoke<any>('get_app_version');
    appVersion.value = versionInfo.version || '';
  } catch (error) {
    console.error('Failed to get app version:', error);
  }
});

function handleClosed() {
  emit('update:modelValue', false);
}
</script>

<style scoped>
.about-dialog :deep(.el-dialog__header) {
  display: none;
}

.about-dialog :deep(.el-dialog__body) {
  padding: 0;
}

.about-content {
  position: relative;
  padding: 30px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  overflow: hidden;
}

/* 背景装饰 */
.bg-decoration {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  pointer-events: none;
  overflow: hidden;
}

.circle {
  position: absolute;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.05);
}

.circle-1 {
  width: 200px;
  height: 200px;
  top: -100px;
  right: -100px;
  animation: float 20s infinite ease-in-out;
}

.circle-2 {
  width: 300px;
  height: 300px;
  bottom: -150px;
  left: -150px;
  animation: float 25s infinite ease-in-out reverse;
}

.circle-3 {
  width: 150px;
  height: 150px;
  top: 50%;
  right: 10%;
  animation: float 30s infinite ease-in-out;
}

@keyframes float {
  0%, 100% { transform: translate(0, 0) scale(1); }
  50% { transform: translate(30px, -30px) scale(1.1); }
}

/* Logo 部分 */
.logo-section {
  text-align: center;
  padding: 30px 0;
  position: relative;
}

.app-logo-wrapper {
  position: relative;
  display: inline-block;
  margin-bottom: 20px;
}

.app-logo {
  position: relative;
  z-index: 2;
  animation: logoFloat 3s infinite ease-in-out;
}

@keyframes logoFloat {
  0%, 100% { transform: translateY(0); }
  50% { transform: translateY(-5px); }
}

.logo-svg {
  filter: drop-shadow(0 10px 20px rgba(0, 0, 0, 0.15));
}

.logo-glow {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: 100px;
  height: 100px;
  background: radial-gradient(circle, rgba(103, 126, 234, 0.3) 0%, transparent 70%);
  filter: blur(20px);
  animation: pulse 2s infinite;
}

@keyframes pulse {
  0%, 100% { transform: translate(-50%, -50%) scale(1); opacity: 0.5; }
  50% { transform: translate(-50%, -50%) scale(1.2); opacity: 0.8; }
}

.app-name {
  margin: 20px 0;
  font-size: 28px;
  font-weight: 700;
  color: white;
  text-shadow: 0 2px 10px rgba(0, 0, 0, 0.2);
  letter-spacing: 0.5px;
}

.gradient-text {
  background: linear-gradient(90deg, #fff 0%, rgba(255, 255, 255, 0.8) 100%);
  -webkit-background-clip: text;
  background-clip: text;
  -webkit-text-fill-color: transparent;
}

.version-badge {
  display: inline-block;
  padding: 6px 16px;
  background: rgba(255, 255, 255, 0.2);
  border-radius: 20px;
  color: white;
  font-size: 14px;
  font-weight: 500;
  backdrop-filter: blur(10px);
  border: 1px solid rgba(255, 255, 255, 0.3);
}

/* 状态卡片 */
.status-cards {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 15px;
  margin: 25px 0;
}

.status-card {
  background: rgba(255, 255, 255, 0.95);
  border-radius: 15px;
  padding: 20px;
  display: flex;
  align-items: center;
  gap: 15px;
  transition: all 0.3s ease;
  box-shadow: 0 5px 15px rgba(0, 0, 0, 0.1);
}

.status-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 20px rgba(0, 0, 0, 0.15);
}

.card-icon {
  width: 50px;
  height: 50px;
  border-radius: 12px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
}

.card-icon.active {
  background: linear-gradient(135deg, #f093fb 0%, #f5576c 100%);
}

.card-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.card-label {
  font-size: 12px;
  color: #666;
  font-weight: 500;
}

.card-value {
  font-size: 14px;
  color: #333;
  font-weight: 600;
  word-break: break-all;
}

/* 技术栈 */
.tech-section {
  background: rgba(255, 255, 255, 0.95);
  border-radius: 15px;
  padding: 15px;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 15px;
  margin: 25px 0;
  box-shadow: 0 5px 15px rgba(0, 0, 0, 0.1);
}

.tech-item {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 14px;
  color: #333;
  font-weight: 500;
}

.tech-icon {
  width: 36px;
  height: 36px;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
}

.tech-divider {
  width: 1px;
  height: 30px;
  background: #e0e0e0;
}

/* 功能特性 */
.features-section {
  background: rgba(255, 255, 255, 0.95);
  border-radius: 15px;
  padding: 25px;
  margin: 25px 0;
  box-shadow: 0 5px 15px rgba(0, 0, 0, 0.1);
}

.section-title {
  font-size: 18px;
  font-weight: 600;
  color: #333;
  margin-bottom: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
}

.title-icon-wrapper {
  width: 28px;
  height: 28px;
  border-radius: 8px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  font-size: 16px;
  animation: sparkle 2s infinite;
}

@keyframes sparkle {
  0%, 100% { opacity: 1; transform: scale(1); }
  50% { opacity: 0.9; transform: scale(1.05); }
}

.feature-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 14px;
}

.feature-card {
  background: #f8f9fa;
  border-radius: 12px;
  padding: 14px 16px;
  display: flex;
  align-items: center;
  gap: 12px;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  cursor: default;
  border: 1px solid transparent;
  min-width: 0;
}

.feature-card:hover {
  background: white;
  transform: translateY(-2px);
  box-shadow: 0 8px 20px rgba(102, 126, 234, 0.15);
  border-color: rgba(102, 126, 234, 0.3);
}

.feature-card:hover .feature-icon-wrapper {
  transform: scale(1.1) rotate(5deg);
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
}

.feature-card:hover .feature-text {
  color: #667eea;
  font-weight: 600;
}

.feature-icon-wrapper {
  width: 36px;
  height: 36px;
  min-width: 36px;
  border-radius: 10px;
  background: white;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 18px;
  color: #606266;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.05);
  transition: all 0.3s ease;
}

.feature-text {
  flex: 1;
  font-size: 14px;
  color: #606266;
  font-weight: 500;
  transition: all 0.3s ease;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* 页脚 */
.footer-section {
  text-align: center;
  padding-top: 20px;
  position: relative;
}

.footer-section::before {
  content: '';
  position: absolute;
  top: 0;
  left: 50%;
  transform: translateX(-50%);
  width: 80px;
  height: 1px;
  background: linear-gradient(90deg, transparent, rgba(255, 255, 255, 0.5), transparent);
}

.footer-content {
  margin-bottom: 10px;
}

.copyright {
  margin: 8px 0;
  font-size: 14px;
  color: rgba(255, 255, 255, 0.95);
  font-weight: 500;
}

.heart {
  display: inline-block;
  animation: heartbeat 1.5s ease-in-out infinite;
  color: #ff4757;
}

@keyframes heartbeat {
  0%, 100% { transform: scale(1); }
  10%, 30% { transform: scale(1.1); }
  20% { transform: scale(1.15); }
}

.year {
  margin: 5px 0;
  font-size: 13px;
  color: rgba(255, 255, 255, 0.8);
}

.author-name {
  font-weight: 700;
  background: linear-gradient(90deg, #ffd700, #ffaa00);
  -webkit-background-clip: text;
  background-clip: text;
  -webkit-text-fill-color: transparent;
}

.open-source-link {
  margin: 6px 0;
}

.github-link {
  color: rgba(255, 255, 255, 0.95);
  text-decoration: none;
  font-size: 13px;
  font-weight: 500;
  padding: 4px 12px;
  background: rgba(255, 255, 255, 0.15);
  border-radius: 12px;
  border: 1px solid rgba(255, 255, 255, 0.25);
  transition: all 0.3s ease;
  display: inline-block;
}

.github-link:hover {
  background: rgba(255, 255, 255, 0.25);
  border-color: rgba(255, 255, 255, 0.4);
  transform: translateY(-1px);
}

.contact-info {
  margin: 6px 0;
  font-size: 11px;
  color: rgba(255, 255, 255, 0.7);
}

.disclaimer {
  margin: 8px 0;
  font-size: 11px;
  color: rgba(255, 255, 255, 0.6);
  font-style: italic;
}

/* 暗色主题适配 */
:root.dark .about-content {
  background: linear-gradient(135deg, #2d3561 0%, #1e1e2e 100%);
}

:root.dark .status-card,
:root.dark .tech-section,
:root.dark .features-section {
  background: rgba(30, 30, 46, 0.95);
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.2);
}

:root.dark .card-label,
:root.dark .tech-item,
:root.dark .section-title,
:root.dark .feature-text {
  color: #e4e4e7;
}

:root.dark .card-value {
  color: #fafafa;
}

:root.dark .tech-divider {
  background: #4a4a5a;
}

:root.dark .feature-card {
  background: rgba(255, 255, 255, 0.05);
}

:root.dark .feature-card:hover {
  background: rgba(255, 255, 255, 0.1);
  border-color: rgba(118, 75, 162, 0.5);
  box-shadow: 0 8px 20px rgba(0, 0, 0, 0.2);
}

:root.dark .feature-icon-wrapper {
  background: rgba(255, 255, 255, 0.1);
  color: #e4e4e7;
}

:root.dark .feature-card:hover .feature-icon-wrapper {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
}

:root.dark .feature-card:hover .feature-text {
  color: #a0c4ff;
}
</style>
