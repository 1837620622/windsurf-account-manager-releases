<template>
  <el-dialog
    v-model="visible"
    title="批量导入账号"
    width="700px"
    :close-on-click-modal="false"
    @close="handleClose"
  >
    <div class="import-container">
      <!-- 导入模式切换 -->
      <div class="mode-section">
        <span class="mode-label">导入模式：</span>
        <el-radio-group v-model="importMode" @change="handleModeChange">
          <el-radio value="password">邮箱密码</el-radio>
          <el-radio value="refresh_token">Refresh Token</el-radio>
        </el-radio-group>
      </div>

      <!-- 格式说明 -->
      <el-alert
        type="info"
        :closable="false"
        show-icon
        style="margin-bottom: 16px;"
      >
        <template #title>
          <span v-if="importMode === 'password'">每行一个账号，格式：<code>邮箱 密码 备注(可选)</code></span>
          <span v-else>每行一个 Token，格式：<code>refresh_token 备注(可选)</code></span>
        </template>
      </el-alert>

      <!-- 输入区域 -->
      <div class="input-section">
        <div class="section-header">
          <span class="section-title">{{ importMode === 'password' ? '账号数据' : 'Refresh Token 列表' }}</span>
          <el-button type="primary" link @click="handleFileImport">
            <el-icon><Upload /></el-icon>
            从文件导入
          </el-button>
        </div>
        <el-input
          v-model="inputText"
          type="textarea"
          :rows="12"
          :placeholder="importMode === 'password' 
            ? 'user1@example.com password123 测试账号1\nuser2@example.com password456\nuser3@example.com password789 备注信息'
            : 'AMf-vBx...长token... 测试账号1\nAMf-vBy...长token...\nAMf-vBz...长token... 备注信息'"
          @input="parseAccounts"
        />
        <input
          ref="fileInputRef"
          type="file"
          accept=".txt,.csv"
          style="display: none;"
          @change="handleFileSelected"
        />
      </div>

      <!-- 解析预览 -->
      <div class="preview-section" v-if="inputText.trim()">
        <div class="section-header">
          <span class="section-title">解析预览</span>
          <div class="stats">
            <el-tag type="success" size="small">有效: {{ validAccounts.length }}</el-tag>
            <el-tag v-if="invalidLines.length > 0" type="danger" size="small">
              无效: {{ invalidLines.length }}
            </el-tag>
          </div>
        </div>
        
        <!-- 有效账号表格 -->
        <el-table
          v-if="validAccounts.length > 0"
          :data="validAccounts.slice(0, 10)"
          size="small"
          max-height="200"
          stripe
        >
          <el-table-column prop="email" label="邮箱" min-width="180" />
          <el-table-column prop="password" label="密码" width="120">
            <template #default="{ row }">
              <span class="password-mask">{{ maskPassword(row.password) }}</span>
            </template>
          </el-table-column>
          <el-table-column prop="remark" label="备注" min-width="100">
            <template #default="{ row }">
              <span class="remark-text">{{ row.remark || '-' }}</span>
            </template>
          </el-table-column>
        </el-table>
        <div v-if="validAccounts.length > 10" class="more-hint">
          ... 还有 {{ validAccounts.length - 10 }} 个账号
        </div>

        <!-- 无效行提示 -->
        <el-alert
          v-if="invalidLines.length > 0"
          type="warning"
          :closable="false"
          style="margin-top: 12px;"
        >
          <template #title>
            格式错误的行: {{ invalidLines.slice(0, 5).join(', ') }}
            <span v-if="invalidLines.length > 5">... 等 {{ invalidLines.length }} 行</span>
          </template>
        </el-alert>
      </div>

      <!-- 导入设置 -->
      <div class="settings-section">
        <div class="section-header">
          <span class="section-title">导入设置</span>
        </div>
        <div class="settings-content">
          <!-- 分组选择 -->
          <div class="setting-item">
            <span class="setting-label">分组:</span>
            <el-select
              v-model="selectedGroup"
              placeholder="选择分组（可选）"
              clearable
              style="width: 180px;"
            >
              <el-option
                v-for="group in settingsStore.groups"
                :key="group"
                :label="group"
                :value="group"
              />
            </el-select>
            <span class="setting-hint">留空则使用默认分组</span>
          </div>
          
          <!-- 标签选择 -->
          <div class="setting-item">
            <span class="setting-label">标签:</span>
            <el-select
              v-model="selectedTags"
              multiple
              collapse-tags
              collapse-tags-tooltip
              placeholder="选择标签（可选）"
              clearable
              style="width: 180px;"
            >
              <el-option
                v-for="tag in settingsStore.tags"
                :key="tag.name"
                :label="tag.name"
                :value="tag.name"
              >
                <span :style="{ color: tag.color }">{{ tag.name }}</span>
              </el-option>
            </el-select>
            <span class="setting-hint">留空则不添加标签</span>
          </div>
          
          <div class="setting-item">
            <span class="setting-label">并发模式:</span>
            <el-tag :type="unlimitedConcurrent ? 'danger' : 'primary'" size="small">
              {{ unlimitedConcurrent ? '全量并发' : `限制并发 (${concurrencyLimit})` }}
            </el-tag>
            <span class="setting-hint">可在设置中修改</span>
          </div>
          <div class="setting-item">
            <el-checkbox v-model="autoLogin">导入后自动登录</el-checkbox>
          </div>
        </div>
      </div>
    </div>

    <template #footer>
      <div class="dialog-footer">
        <el-button @click="handleClose">取消</el-button>
        <el-button
          type="primary"
          :disabled="validAccounts.length === 0"
          :loading="importing"
          @click="handleImport"
        >
          {{ importing ? '导入中...' : `导入 ${validAccounts.length} 个账号` }}
        </el-button>
      </div>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { Upload } from '@element-plus/icons-vue';
import { useSettingsStore } from '@/store';

const props = defineProps<{
  modelValue: boolean;
}>();

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void;
  (e: 'import', accounts: Array<{ email: string; password: string; remark: string; refreshToken?: string }>, autoLogin: boolean, group: string, tags: string[], mode: 'password' | 'refresh_token'): void;
}>();

const settingsStore = useSettingsStore();

const visible = computed({
  get: () => props.modelValue,
  set: (val) => emit('update:modelValue', val)
});

const inputText = ref('');
const validAccounts = ref<Array<{ email: string; password: string; remark: string; refreshToken?: string }>>([]);
const invalidLines = ref<number[]>([]);
const autoLogin = ref(true);
const importing = ref(false);
const fileInputRef = ref<HTMLInputElement | null>(null);
const selectedGroup = ref<string>('');
const selectedTags = ref<string[]>([]);
const importMode = ref<'password' | 'refresh_token'>('password');

const unlimitedConcurrent = computed(() => settingsStore.settings?.unlimitedConcurrentRefresh || false);
const concurrencyLimit = computed(() => settingsStore.settings?.concurrent_limit || 5);

// 切换模式时重置
function handleModeChange() {
  inputText.value = '';
  validAccounts.value = [];
  invalidLines.value = [];
}

// 解析账号数据
function parseAccounts() {
  const lines = inputText.value.split('\n').filter(line => line.trim());
  validAccounts.value = [];
  invalidLines.value = [];

  if (importMode.value === 'password') {
    // 邮箱密码模式
    lines.forEach((line, index) => {
      const parts = line.trim().split(/\s+/);
      if (parts.length >= 2) {
        const [email, password, ...remarkParts] = parts;
        if (email.includes('@')) {
          validAccounts.value.push({
            email,
            password,
            remark: remarkParts.join(' ') || ''
          });
        } else {
          invalidLines.value.push(index + 1);
        }
      } else {
        invalidLines.value.push(index + 1);
      }
    });
  } else {
    // Refresh Token 模式
    lines.forEach((line, index) => {
      const parts = line.trim().split(/\s+/);
      if (parts.length >= 1 && parts[0].length >= 10) {
        const [token, ...remarkParts] = parts;
        validAccounts.value.push({
          email: `Token #${index + 1}`,
          password: '',
          remark: remarkParts.join(' ') || '',
          refreshToken: token
        });
      } else {
        invalidLines.value.push(index + 1);
      }
    });
  }
}

// 遮蔽密码显示
function maskPassword(password: string): string {
  if (password.length <= 4) {
    return '*'.repeat(password.length);
  }
  return password.slice(0, 2) + '*'.repeat(password.length - 4) + password.slice(-2);
}

// 从文件导入
function handleFileImport() {
  fileInputRef.value?.click();
}

function handleFileSelected(event: Event) {
  const input = event.target as HTMLInputElement;
  const file = input.files?.[0];
  if (!file) return;

  const reader = new FileReader();
  reader.onload = (e) => {
    const content = e.target?.result as string;
    inputText.value = content;
    parseAccounts();
  };
  reader.readAsText(file);
  
  // 重置input，允许再次选择同一文件
  input.value = '';
}

// 执行导入
function handleImport() {
  if (validAccounts.value.length === 0) return;
  importing.value = true;
  emit('import', [...validAccounts.value], autoLogin.value, selectedGroup.value || '默认分组', [...selectedTags.value], importMode.value);
}

// 关闭对话框
function handleClose() {
  if (!importing.value) {
    inputText.value = '';
    validAccounts.value = [];
    invalidLines.value = [];
    selectedGroup.value = '';
    selectedTags.value = [];
    importMode.value = 'password';
    visible.value = false;
  }
}

// 导入完成后重置状态
function resetImporting() {
  importing.value = false;
}

// 监听对话框关闭
watch(visible, (val) => {
  if (!val) {
    inputText.value = '';
    validAccounts.value = [];
    invalidLines.value = [];
    selectedGroup.value = '';
    selectedTags.value = [];
    importing.value = false;
    importMode.value = 'password';
  }
});

defineExpose({
  resetImporting
});
</script>

<style scoped>
.import-container {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.mode-section {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 12px;
  background: #f0f9eb;
  border-radius: 6px;
}

.mode-label {
  font-size: 13px;
  color: #606266;
  font-weight: 500;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.section-title {
  font-weight: 600;
  color: var(--el-text-color-primary);
}

.input-section :deep(.el-textarea__inner) {
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 13px;
  line-height: 1.6;
}

.preview-section {
  background: var(--el-fill-color-light);
  border-radius: 8px;
  padding: 12px;
}

.stats {
  display: flex;
  gap: 8px;
}

.password-mask {
  font-family: monospace;
  color: var(--el-text-color-secondary);
}

.remark-text {
  color: var(--el-text-color-secondary);
  font-size: 12px;
}

.more-hint {
  text-align: center;
  color: var(--el-text-color-secondary);
  font-size: 12px;
  padding: 8px 0;
}

.settings-section {
  background: var(--el-fill-color-lighter);
  border-radius: 8px;
  padding: 12px;
}

.settings-content {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.setting-item {
  display: flex;
  align-items: center;
  gap: 8px;
}

.setting-label {
  color: var(--el-text-color-regular);
  font-size: 13px;
}

.setting-hint {
  color: var(--el-text-color-placeholder);
  font-size: 12px;
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
}

code {
  background: var(--el-fill-color);
  padding: 2px 6px;
  border-radius: 4px;
  font-family: monospace;
}
</style>
